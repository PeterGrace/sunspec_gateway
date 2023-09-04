use crate::ipc::{IPCMessage, Payload, PublishMessage, ValueType};
use crate::monitored_point::MonitoredPoint;
use crate::sunspec_unit::SunSpecUnit;
use crate::SETTINGS;
use sunspec_rs::sunspec_models::ResponseType;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration, Instant};

const MINIMUM_POLL_INTERVAL: u16 = 15_u16;

const DEFAULT_DISPLAY_PRECISION: u8 = 4_u8;

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
                warn!(%sn, "Can't find any model points defined for models.{id} in config: {e}");
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
            let precision = match map.get("precision") {
                None => None,
                Some(v) => Some(v.clone().into_uint().unwrap() as u8),
            };
            match MonitoredPoint::new(
                id.to_string(),
                point.clone(),
                Duration::from_secs(interval),
                device_class,
                state_class,
                precision,
            ) {
                Ok(p) => points.push(p),
                Err(e) => {
                    warn!(%sn, "unable to create MonitoredPoint for {id}/{point}: {e}");
                    continue;
                }
            };
        }
        info!("Points: {:#?}", points);
    }

    loop {
        for p in points.iter() {
            let model = p.model.clone();
            let point_name = p.name.clone();
            let md = unit
                .conn
                .models
                .get(&p.model.parse::<u16>().unwrap())
                .unwrap();
            match unit.conn.clone().get_point(md.clone(), &p.name).await {
                None => {}
                Some(v) => match v.value {
                    None => {}
                    Some(val) => {
                        let topic = format!("homeassistant/sensor/{sn}/{model}_{point_name}");
                        let mut payload: Payload = Payload::default();
                        payload.device_class = p.device_class.clone();
                        payload.state_class = p.state_class.clone();
                        payload.unique_id = format!("{sn}.{model}.{point_name}");
                        match val {
                            ResponseType::String(str) => {
                                info!("Response for {model}/{point_name}: {str}");
                                payload.native_value = ValueType::string(str)
                            }
                            ResponseType::Integer(int) => {
                                info!("Response for {model}/{point_name}: {int}");
                                payload.native_uom = v.units.clone();
                                payload.native_value = ValueType::int(int as i64)
                            }
                            ResponseType::Float(float) => {
                                info!("Response for {model}/{point_name}: {0:.1}", float);
                                if let Some(precision) = p.precision {
                                    payload.suggested_display_precision = precision;
                                } else {
                                    payload.suggested_display_precision = DEFAULT_DISPLAY_PRECISION;
                                };
                                payload.native_uom = v.units.clone();
                                payload.native_value = ValueType::float(float as f64)
                            }
                            ResponseType::Boolean(boolean) => {
                                info!("Response for {model}/{point_name}: {boolean}");
                                payload.native_value = ValueType::string(boolean.to_string());
                            }
                            ResponseType::Array(vec) => {
                                info!("Response for {model}/{point_name}: {:#?}", vec);
                                let concat = vec.join(",");
                                payload.native_value = ValueType::string(concat)
                            }
                        }
                        payload.device = unit.device_info.clone();
                        let _ = tx
                            .send(IPCMessage::Outbound(PublishMessage { topic, payload }))
                            .await;
                    }
                },
            }
        }

        info!(%sn, "Device tick");
        let _ = sleep(Duration::from_secs(MINIMUM_POLL_INTERVAL.into())).await;
    }
    //tx.send(IPCMessage::Error(IPCError{rcpn:rcpn.to_owned(), msg: "Exited loop".to_string()}));
}
