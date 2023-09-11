use crate::ipc::{
    HAConfigPayload, IPCMessage, Payload, PayloadValueType, PublishMessage, StatePayload,
};
use crate::monitored_point::MonitoredPoint;
use crate::sunspec_unit::SunSpecUnit;
use crate::SETTINGS;
use chrono::Utc;
use rand::{thread_rng, Rng};
use sunspec_rs::sunspec_models::ValueType;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration, Instant};

const MINIMUM_POLL_INTERVAL_SECS: u16 = 5_u16;

const DEFAULT_DISPLAY_PRECISION: Option<u8> = Some(4_u8);

pub async fn poll_loop(unit: &SunSpecUnit, tx: Sender<IPCMessage>) -> anyhow::Result<()> {
    let sn = &unit.serial_number;
    let config = SETTINGS.read().await;
    let mut points: Vec<MonitoredPoint> = vec![];
    let mut point: String = String::default();
    let mut interval: u64 = 0;
    for (id, _) in unit.conn.models.iter() {
        let mname = format!("{id}");
        let models_table = config.get_table("models").unwrap();

        let modelpoints = match models_table.get(&mname) {
            Some(m) => match m.clone().into_array() {
                Ok(pointarray) => pointarray,
                Err(e) => {
                    error!("configuration issue with models array in config.yaml");
                    continue;
                }
            },
            None => {
                info!(%sn, "Can't find any model points defined for models.{id} in config.");
                continue;
            }
        };
        for pointname in modelpoints.iter() {
            let map = pointname.clone().into_table().unwrap();
            let point = match map.get("point") {
                Some(p) => p.clone().into_string().unwrap(),
                None => {
                    warn!(%sn, "model {id} missing point def in config, skipping point");
                    continue;
                }
            };
            let interval = match map.get("interval") {
                Some(i) => i.clone().into_uint().unwrap(),
                None => {
                    warn!(%sn, "model {id} missing interval def in config, skipping point");
                    continue;
                }
            };
            let device_class = match map.get("device_class") {
                None => None,
                Some(v) => Some(v.clone().into_string().unwrap()),
            };
            let state_class = match map.get("state_class") {
                None => None,
                Some(v) => Some(v.clone().into_string().unwrap()),
            };
            let uom = match map.get("uom") {
                None => None,
                Some(v) => Some(v.clone().into_string().unwrap()),
            };
            let precision = match map.get("precision") {
                None => None,
                Some(v) => Some(v.clone().into_uint().unwrap() as u8),
            };
            let homeassistant = match map.get("homeassistant") {
                None => true,
                Some(v) => match v.clone().into_bool() {
                    Ok(b) => b,
                    Err(e) => {
                        error!("homeassistant value in config file isn't true or false, assuming false for safety");
                        false
                    }
                },
            };
            match MonitoredPoint::new(
                id.to_string(),
                point.clone(),
                interval,
                device_class,
                state_class,
                precision,
                uom,
                homeassistant,
            ) {
                Ok(p) => points.push(p),
                Err(e) => {
                    warn!(%sn, "unable to create MonitoredPoint for {id}/{point}: {e}");
                    continue;
                }
            };
        }
    }
    // since we're done reading config file variables, lets drop the RwLock.
    drop(config);

    loop {
        for p in points.iter_mut() {
            let interval = p.interval as i64;
            let time_pad = thread_rng().gen_range(0..interval) as i64;
            if Utc::now().timestamp() - p.last_report.timestamp() < (interval + time_pad) {
                trace!("interval has not elapsed for {}", p.name.clone());
                continue;
            }
            p.last_report = Utc::now();
            let model = p.model.clone();
            let point_name = p.name.clone();
            debug!("Checking point {model}/{point_name}");

            let md = unit
                .conn
                .models
                .get(&p.model.parse::<u16>().unwrap())
                .unwrap();
            match unit.conn.clone().get_point(md.clone(), &p.name).await {
                None => {
                    debug!("No value for {model}/{point_name} returned from get_point()");
                }
                Some(v) => match v.value {
                    None => {}
                    Some(val) => {
                        let state_topic = format!("sunspec_gateway/{sn}/{model}/{point_name}");
                        let config_topic =
                            format!("homeassistant/sensor/{sn}/{model}_{point_name}/config");
                        let mut config_payload: HAConfigPayload = HAConfigPayload::default();
                        let mut state_payload: StatePayload = StatePayload::default();
                        config_payload.name = format!("{sn}: {model}-{point_name}");
                        config_payload.state_topic = state_topic.clone();
                        config_payload.device_class = p.device_class.clone();
                        config_payload.state_class = p.state_class.clone();
                        config_payload.expires_after = 300;
                        config_payload.value_template = Some("{{ value_json.value }}".to_string());
                        config_payload.unique_id = format!("{sn}.{}.{point_name}", p.model);
                        config_payload.entity_id = format!("sensor.{sn}_{}_{point_name}", p.model);
                        match val {
                            ValueType::String(str) => {
                                debug!("Response for {model}/{point_name}: {str}");
                                state_payload.value = PayloadValueType::String(str)
                            }
                            ValueType::Integer(int) => {
                                debug!("Response for {model}/{point_name}: {int}");
                                if p.uom.is_some() {
                                    // we are overriding the default uom from config
                                    config_payload.native_uom = p.uom.clone();
                                } else {
                                    config_payload.native_uom = v.units.clone();
                                }
                                state_payload.value = PayloadValueType::Int(int as i64)
                            }
                            ValueType::Float(float) => {
                                debug!("Response for {model}/{point_name}: {0:.1}", float);
                                if p.precision.is_some() {
                                    config_payload.suggested_display_precision = p.precision;
                                } else {
                                    config_payload.suggested_display_precision =
                                        DEFAULT_DISPLAY_PRECISION;
                                };
                                if p.uom.is_some() {
                                    // we are overriding the default uom from config
                                    config_payload.native_uom = p.uom.clone();
                                } else {
                                    config_payload.native_uom = v.units.clone();
                                }
                                state_payload.value = PayloadValueType::Float(float as f64);
                                if let Some(literal) = v.literal {
                                    if literal.label.is_some() {
                                        config_payload.name = literal.label.clone().unwrap();
                                    }
                                    state_payload.label = literal.label;
                                    state_payload.description = literal.description;
                                    state_payload.notes = literal.notes;
                                }
                            }
                            ValueType::Boolean(boolean) => {
                                debug!("Response for {model}/{point_name}: {boolean}");
                                state_payload.value = PayloadValueType::String(boolean.to_string());
                            }
                            ValueType::Array(vec) => {
                                debug!("Response for {model}/{point_name}: {:#?}", vec);
                                let concat = vec.join(",");
                                state_payload.value = PayloadValueType::String(concat)
                            }
                        }
                        config_payload.device = unit.device_info.clone();
                        if p.homeassistant_discovery {
                            let _ = tx
                                .send(IPCMessage::Outbound(PublishMessage {
                                    topic: config_topic,
                                    payload: Payload::Config(config_payload),
                                }))
                                .await;
                        }
                        let _ = tx
                            .send(IPCMessage::Outbound(PublishMessage {
                                topic: state_topic,
                                payload: Payload::CurrentState(state_payload),
                            }))
                            .await;
                    }
                },
            }
        }

        info!(%sn, "Device tick");
        let _ = sleep(Duration::from_secs(MINIMUM_POLL_INTERVAL_SECS.into())).await;
    }
    error!("Aiee, leaving poll_loop");
}
