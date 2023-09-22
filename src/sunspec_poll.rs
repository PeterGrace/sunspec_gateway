use crate::ipc::{
    HAConfigPayload, IPCMessage, Payload, PayloadValueType, PublishMessage, StatePayload,
};
use crate::monitored_point::MonitoredPoint;
use crate::payload::generate_payloads;
use crate::sunspec_unit::SunSpecUnit;
use crate::{GatewayError, SETTINGS};
use chrono::Utc;
use rand::{thread_rng, Rng};
use sunspec_rs::sunspec_connection::SunSpecPointError;
use sunspec_rs::sunspec_models::{Access, ValueType};
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration, Instant};

const MINIMUM_POLL_INTERVAL_SECS: u16 = 5_u16;

pub async fn poll_loop(unit: &SunSpecUnit, tx: Sender<IPCMessage>) -> Result<(), GatewayError> {
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
                debug!(%sn, "Can't find any model points defined for models.{id} in config.");
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
            let readwrite = match map.get("readwrite") {
                None => Access::ReadOnly,
                Some(v) => match v.clone().into_bool() {
                    Ok(b) => {
                        if b {
                            Access::ReadWrite
                        } else {
                            Access::ReadOnly
                        }
                    }
                    Err(e) => {
                        error!("readwrite value in config file isn't true or false, assuming Read-Only for safety");
                        Access::ReadOnly
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
                readwrite,
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
            let model = p.model.clone();
            let point_name = p.name.clone();
            if Utc::now().timestamp() - p.last_report.timestamp() < (interval + time_pad) {
                continue;
            } else {
                if Utc::now().timestamp() - p.last_report.timestamp() > 1800 {
                    warn!("point {model}/{point_name} hasn't been updated in over 30 minutes.");
                }
            }

            debug!("Checking point {model}/{point_name}");

            let md = unit
                .conn
                .models
                .get(&p.model.parse::<u16>().unwrap())
                .unwrap();
            match unit.conn.clone().get_point(md.clone(), &p.name).await {
                Err(e) => {
                    match e {
                        SunSpecPointError::GeneralError(e) => {
                            error!("{model}/{point_name}: General error reading point: {e}");
                            continue;
                        }
                        SunSpecPointError::DoesNotExist(e) => {
                            error!("{model}/{point_name} Point specified does not exist: {e}");
                            continue;
                        }
                        SunSpecPointError::UndefinedError => {
                            warn!("{model}/{point_name} Undefined error returned: {e}");
                            continue;
                        }
                        SunSpecPointError::CommError(e) => {
                            let _ = tx
                                .send(IPCMessage::PleaseReconnect(
                                    unit.addr.clone(),
                                    unit.slave_id,
                                ))
                                .await;
                            // lets wait two seconds for the ipc to process.
                            let _ = sleep(Duration::from_secs(2)).await;
                            return Err(GatewayError::CommunicationError(e.to_string()));
                        }
                    }
                }
                Ok(recvd_point) => match recvd_point.clone().value {
                    None => {}
                    Some(val) => {
                        let state_topic = format!("sunspec_gateway/{sn}/{model}/{point_name}");
                        let config_topic =
                            format!("homeassistant/sensor/{sn}/{model}_{point_name}/config");
                        let v = recvd_point.clone();
                        let (config_payload, state_payload) =
                            generate_payloads(unit, &recvd_point, &p, &val);
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
                        p.last_report = Utc::now();
                    }
                },
            }
        }

        debug!(%sn, "Device tick");
        let _ = sleep(Duration::from_secs(MINIMUM_POLL_INTERVAL_SECS.into())).await;
    }
}
