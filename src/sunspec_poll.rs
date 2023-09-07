use crate::ipc::{HAConfigPayload, IPCMessage, Payload, PublishMessage, StatePayload, ValueType};
use crate::monitored_point::MonitoredPoint;
use crate::sunspec_unit::SunSpecUnit;
use crate::SETTINGS;
use chrono::Utc;
use rand::{thread_rng, Rng};
use sunspec_rs::sunspec_models::ResponseType;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration, Instant};

const MINIMUM_POLL_INTERVAL: u16 = 1_u16;

const DEFAULT_DISPLAY_PRECISION: Option<u8> = Some(4_u8);

pub async fn poll_loop(unit: &SunSpecUnit, tx: Sender<IPCMessage>) {
    let sn = &unit.serial_number;
    let config = SETTINGS.read().await;
    let mut points: Vec<MonitoredPoint> = vec![];
    let mut point: String = String::default();
    let mut interval: u64 = 0;
    for (id, _) in unit.conn.models.iter() {
        let mname = format!("models.{id}");
        let modelpoints = match config.get_array(&mname) {
            Ok(m) => m,
            Err(e) => {
                info!(%sn, "Can't find any model points defined for models.{id} in config: {e}");
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
            let time_pad = thread_rng().gen_range(0..6);
            if Utc::now().timestamp() - p.last_report.timestamp() < (p.interval as i64 + time_pad) {
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
                            ResponseType::String(str) => {
                                info!("Response for {model}/{point_name}: {str}");
                                state_payload.value = ValueType::String(str)
                            }
                            ResponseType::Integer(int) => {
                                info!("Response for {model}/{point_name}: {int}");
                                if p.uom.is_some() {
                                    // we are overriding the default uom from config
                                    config_payload.native_uom = p.uom.clone();
                                } else {
                                    config_payload.native_uom = v.units.clone();
                                }
                                state_payload.value = ValueType::Int(int as i64)
                            }
                            ResponseType::Float(float) => {
                                info!("Response for {model}/{point_name}: {0:.1}", float);
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
                                state_payload.value = ValueType::Float(float as f64);
                                if let Some(literal) = v.literal {
                                    if literal.label.is_some() {
                                        config_payload.name = literal.label.clone().unwrap();
                                    }
                                    state_payload.label = literal.label;
                                    state_payload.description = literal.description;
                                    state_payload.notes = literal.notes;
                                }
                            }
                            ResponseType::Boolean(boolean) => {
                                info!("Response for {model}/{point_name}: {boolean}");
                                state_payload.value = ValueType::String(boolean.to_string());
                            }
                            ResponseType::Array(vec) => {
                                info!("Response for {model}/{point_name}: {:#?}", vec);
                                let concat = vec.join(",");
                                state_payload.value = ValueType::String(concat)
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

        trace!(%sn, "Device tick");
        let _ = sleep(Duration::from_secs(MINIMUM_POLL_INTERVAL.into())).await;
    }
}
