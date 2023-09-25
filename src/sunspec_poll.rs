use crate::ipc::{IPCMessage, PublishMessage};
use crate::monitored_point::MonitoredPoint;
use crate::payload::generate_payloads;
use crate::payload::{HAConfigPayload, Payload, PayloadValueType, StatePayload};
use crate::state_mgmt::{cull_records_to, write_payload_history};
use crate::sunspec_unit::SunSpecUnit;
use crate::{GatewayError, SETTINGS};
use chrono::Utc;
use rand::{thread_rng, Rng};
use sunspec_rs::model_data::ModelData;
use sunspec_rs::sunspec_connection::SunSpecPointError;
use sunspec_rs::sunspec_models::{Access, ValueType};
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration, Instant};

const MINIMUM_POLL_INTERVAL_SECS: u16 = 5_u16;
const CULL_HISTORY_ROWS: u8 = 50_u8;

pub async fn poll_loop(unit: &SunSpecUnit, tx: Sender<IPCMessage>) -> Result<(), GatewayError> {
    let sn = &unit.serial_number;
    let config = SETTINGS.read().await;
    let mut points: Vec<MonitoredPoint> = vec![];
    for (id, _) in unit.conn.models.iter() {
        let mname = format!("{id}");
        for (model, config_points) in config.models.iter() {
            for point in config_points {
                match MonitoredPoint::new(model.clone(), point.clone()) {
                    Ok(p) => points.push(p),
                    Err(e) => {
                        warn!(%sn, "unable to create MonitoredPoint for {id}/{}: {e}", point.point);
                        continue;
                    }
                };
            }
        }
    } // at this point, `points` should contain all points we've been asked to check.

    // since we're done reading config file variables, lets drop the RwLock.
    drop(config);

    loop {
        // p is each point we've been asked to check.
        for requested_point_to_check in points.iter_mut() {
            let md = match unit
                .conn
                .models
                .get(&requested_point_to_check.model.parse::<u16>().unwrap())
            {
                Some(model) => Some(model),
                None => {
                    // the unit we're watching doesn't have this point in its system table
                    continue;
                }
            };
            let interval = requested_point_to_check.interval as i64;
            let time_pad = thread_rng().gen_range(0..interval) as i64;
            let model = requested_point_to_check.model.clone();
            let point_name = requested_point_to_check.name.clone();
            if Utc::now().timestamp() - requested_point_to_check.last_report.timestamp()
                < (interval + time_pad)
            {
                continue;
            } else {
                if Utc::now().timestamp() - requested_point_to_check.last_report.timestamp() > 1800
                {
                    warn!("point {model}/{point_name} hasn't been updated in over 30 minutes.");
                }
            }
            debug!(%sn, "Checking point {model}/{point_name}");

            match unit
                .conn
                .clone()
                .get_point(md.unwrap().clone(), &requested_point_to_check.name)
                .await
            {
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
                        let v = recvd_point.clone();
                        let payloads =
                            generate_payloads(unit, &recvd_point, &requested_point_to_check, &val)
                                .await;

                        for payload in payloads {
                            if requested_point_to_check.homeassistant_discovery {
                                let _ = tx
                                    .send(IPCMessage::Outbound(PublishMessage {
                                        topic: payload.config_topic,
                                        payload: Payload::Config(payload.config.clone()),
                                    }))
                                    .await;
                            }

                            let _ = tx
                                .send(IPCMessage::Outbound(PublishMessage {
                                    topic: payload.state_topic,
                                    payload: Payload::CurrentState(payload.state.clone()),
                                }))
                                .await;
                            if let Err(e) =
                                cull_records_to(payload.config.clone().unique_id, CULL_HISTORY_ROWS)
                                    .await
                            {
                                warn!("Couldn't cull history for this point: {e}");
                            };
                            if let Err(e) =
                                write_payload_history(payload.config, payload.state).await
                            {
                                warn!("Unable to store value in db: {e}");
                            }
                        }
                        requested_point_to_check.last_report = Utc::now();
                    }
                },
            }
        }

        debug!(%sn, "Device tick");
        let _ = sleep(Duration::from_secs(MINIMUM_POLL_INTERVAL_SECS.into())).await;
    }
}
