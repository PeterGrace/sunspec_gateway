use crate::config_structs::InputType;
use crate::consts::*;
use crate::ipc::{IPCMessage, InboundMessage, PublishMessage};
use crate::monitored_point::MonitoredPoint;
use crate::payload::generate_payloads;
use crate::payload::{HAConfigPayload, Payload, PayloadValueType, StatePayload};
use crate::state_mgmt::{cull_records_to, write_payload_history};
use crate::sunspec_unit::SunSpecUnit;
use crate::{GatewayError, SETTINGS};
use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use sunspec_rs::model_data::ModelData;
use sunspec_rs::sunspec_connection::SunSpecPointError;
use sunspec_rs::sunspec_models::{Access, ValueType};
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration, Instant};

pub async fn poll_loop(
    unit: &SunSpecUnit,
    tx: Sender<IPCMessage>,
    mut broadcast_rx: Receiver<IPCMessage>,
) -> Result<(), GatewayError> {
    let genesis_moment = Utc::now();
    let sn = &unit.serial_number;
    let config = SETTINGS.read().await;
    let mut points: Vec<MonitoredPoint> = vec![];
    let mut last_report: HashMap<String, DateTime<Utc>> = HashMap::new();
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
        for requested_point_to_check in points.iter_mut() {
            //region assign variables
            let interval = requested_point_to_check.interval as i64;
            let time_pad = thread_rng().gen_range(0..interval) as i64;
            let model = requested_point_to_check.model.clone();
            let point_name = requested_point_to_check.name.clone();
            let uniqueid = format!("{sn}.{model}.{point_name}");
            let log_prefix = format!(
                "[{}:{} {sn} {model}/{point_name}]",
                unit.addr, unit.slave_id
            );
            //endregion

            //region handle any incoming write requests asap
            match broadcast_rx.try_recv() {
                Ok(msg) => {
                    match msg {
                        IPCMessage::Shutdown => {
                            info!("{log_prefix}: Received shutdown message, exiting thread.");
                            // TODO: when I implement disconnect on SunSpecConnection, should call that here
                            return Err(GatewayError::ExitingThread);
                        }
                        IPCMessage::Inbound(inmsg) => {
                            if inmsg.serial_number == *sn {
                                info!("{log_prefix}: message was destined for me");
                                let mn = unit.device_info.manufacturer.clone();
                                let ssd = unit.data.clone();
                                let mid = inmsg.model.parse::<u16>().unwrap();

                                if let Some(symbols) = ssd.get_symbols_for_point(
                                    mid,
                                    inmsg.point_name.clone(),
                                    Some(mn),
                                ) {
                                    for symbol in symbols {
                                        if symbol.id == inmsg.payload {
                                            info!("Found symbol {}->{}", symbol.id, symbol.symbol);
                                            let md = unit.conn.models.get(&mid).unwrap();
                                            match unit
                                                .conn
                                                .clone()
                                                .set_point(
                                                    md.clone(),
                                                    &inmsg.point_name,
                                                    ValueType::Integer(
                                                        symbol.symbol.parse::<i32>().unwrap(),
                                                    ),
                                                )
                                                .await
                                            {
                                                Ok(_) => {
                                                    // TODO maybe I can immediately reschedule a check of the point to get it refreshed?
                                                    info!(
                                                        "Value successfully sent {}:{}",
                                                        inmsg.point_name, symbol.id
                                                    );
                                                }
                                                Err(e) => {
                                                    error!("Couldn't set point: {e}");
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    debug!("This inbound message has no symbol.  Need to check for number");
                                    let config = SETTINGS.read().await;
                                    let points = config.models.get(&inmsg.model).unwrap().clone();
                                    drop(config);
                                    for point in points {
                                        if point.point == inmsg.point_name {
                                            debug!("Found point config, now validating number");
                                            if let Some(input) = point.inputs {
                                                match input {
                                                    InputType::Select(_) => {
                                                        warn!("{log_prefix}: We can't find a symbol for this selection.  Received {}", inmsg.payload);
                                                    }
                                                    InputType::Switch(sw) => {
                                                        if let Ok(on_val) = sw.on.parse::<u32>() {
                                                            if let Ok(off_val) =
                                                                sw.off.parse::<u32>()
                                                            {
                                                                if let Ok(payload_val) =
                                                                    inmsg.payload.parse::<u32>()
                                                                {
                                                                    // phew, we have all the ducks in a row
                                                                    if payload_val == on_val
                                                                        || payload_val == off_val
                                                                    {
                                                                        let md = unit
                                                                            .conn
                                                                            .models
                                                                            .get(&mid)
                                                                            .unwrap();
                                                                        match unit
                                                                            .conn
                                                                            .clone()
                                                                            .set_point(
                                                                                md.clone(),
                                                                                &inmsg.point_name,
                                                                                ValueType::Integer(
                                                                                    payload_val
                                                                                        .try_into()
                                                                                        .unwrap(),
                                                                                ),
                                                                            )
                                                                            .await
                                                                        {
                                                                            Ok(_) => {
                                                                                // TODO maybe I can immediately reschedule a check of the point to get it refreshed?
                                                                                info!(
                                                        "Value successfully sent {}:{}",
                                                        inmsg.point_name, payload_val
                                                    );
                                                                            }
                                                                            Err(e) => {
                                                                                error!("Couldn't set point: {e}");
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        //warn!("{log_prefix}: We can't find a symbol for this switch value.  Received {}", inmsg.payload);
                                                    }
                                                    InputType::Button(val) => {
                                                        if let Ok(off_val) = val.parse::<u32>() {
                                                            if let Ok(payload_val) =
                                                                inmsg.payload.parse::<u32>()
                                                            {
                                                                // phew, we have all the ducks in a row
                                                                if payload_val == off_val {
                                                                    let md = unit
                                                                        .conn
                                                                        .models
                                                                        .get(&mid)
                                                                        .unwrap();
                                                                    match unit
                                                                        .conn
                                                                        .clone()
                                                                        .set_point(
                                                                            md.clone(),
                                                                            &inmsg.point_name,
                                                                            ValueType::Integer(
                                                                                payload_val
                                                                                    .try_into()
                                                                                    .unwrap(),
                                                                            ),
                                                                        )
                                                                        .await
                                                                    {
                                                                        Ok(_) => {
                                                                            // TODO maybe I can immediately reschedule a check of the point to get it refreshed?
                                                                            info!(
                                                        "Value successfully sent {}:{}",
                                                        inmsg.point_name, payload_val
                                                    );
                                                                        }
                                                                        Err(e) => {
                                                                            error!("Couldn't set point: {e}");
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        //warn!("{log_prefix}: We can't find a symbol for this button payload.  Received {}", inmsg.payload);
                                                    }
                                                    InputType::Number(val) => {
                                                        debug!("Inbound payload is a number!");
                                                        match inmsg.payload.parse::<i32>() {
                                                            Ok(parsed) => {
                                                                if (parsed >= val.min
                                                                    && parsed <= val.max)
                                                                {
                                                                    let md = unit
                                                                        .conn
                                                                        .models
                                                                        .get(&mid)
                                                                        .unwrap();
                                                                    match unit
                                                                        .conn
                                                                        .clone()
                                                                        .set_point(
                                                                            md.clone(),
                                                                            &inmsg.point_name,
                                                                            ValueType::Integer(
                                                                                parsed,
                                                                            ),
                                                                        )
                                                                        .await
                                                                    {
                                                                        Ok(_) => {
                                                                            // TODO maybe I can immediately reschedule a check of the point to get it refreshed?
                                                                            info!(
                                                        "Value successfully sent {}:{}",
                                                        inmsg.point_name, parsed
                                                    );
                                                                        }
                                                                        Err(e) => {
                                                                            error!("Couldn't set point: {e}");
                                                                        }
                                                                    }
                                                                } else {
                                                                    error!("{log_prefix}: Sanity check failed: {parsed} is outside of {}<->{}",val.min, val.max);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                error!("{log_prefix}: Inbound number was unparseable: {e}");
                                                            }
                                                        }
                                                    }
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        IPCMessage::Outbound(_) => {
                            todo!()
                        }
                        IPCMessage::PleaseReconnect(_, _) => {
                            todo!()
                        }
                        IPCMessage::Error(_) => {
                            todo!()
                        }
                    }
                }
                Err(e) => match e {
                    TryRecvError::Empty => {}
                    TryRecvError::Closed => {
                        panic!("Broadcast channel closed?")
                    }
                    TryRecvError::Lagged(lag) => {
                        warn!("broadcast channel is lagged: {e}")
                    }
                },
            }
            //endregion

            // region instantiate modeldata for unit
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
            //endregion
            let now_stamp = Utc::now().timestamp();
            let last_stamp = last_report
                .get(&uniqueid)
                .unwrap_or(&genesis_moment)
                .timestamp();
            //region check if it is time to send a datapoint / report on point being stale
            if (now_stamp - last_stamp) < (interval + time_pad) {
                continue;
            } else {
                if (now_stamp - last_stamp) > 1800 {
                    warn!(
                        "{log_prefix}: point hasn't been updated in over 30 minutes {} | {} .",
                        now_stamp, last_stamp
                    );
                }
            }
            //endregion

            debug!("{log_prefix}: Checking point {model}/{point_name}");
            let input_only = requested_point_to_check.input_only.unwrap_or(false);
            //region actually get the point and generate payload
            if input_only {
                debug!("{log_prefix}: this point is input-only; skipping point get and just sending config payload.");
                let payloads = generate_payloads(unit, None, &requested_point_to_check, None).await;

                last_report.insert(uniqueid, Utc::now());
                if requested_point_to_check.homeassistant_discovery {
                    let _ = tx
                        .send(IPCMessage::Outbound(PublishMessage {
                            topic: payloads[0].config_topic.clone(),
                            payload: Payload::Config(payloads[0].config.clone()),
                        }))
                        .await;
                }
            } else {
                match unit
                    .conn
                    .clone()
                    .get_point(md.unwrap().clone(), &requested_point_to_check.name, None)
                    .await
                {
                    Err(e) => {
                        match e {
                            SunSpecPointError::GeneralError(e) => {
                                error!("{log_prefix}: General error reading point: {e}");
                                continue;
                            }
                            SunSpecPointError::DoesNotExist(e) => {
                                error!("{log_prefix}: Point specified does not exist: {e}");
                                continue;
                            }
                            SunSpecPointError::UndefinedError => {
                                warn!("{log_prefix}: Undefined error returned: {e}");
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
                                let _ =
                                    sleep(Duration::from_millis(MQTT_PROCESSING_PAD_MILLIS)).await;
                                return Err(GatewayError::CommunicationError(e.to_string()));
                            }
                        }
                    }
                    Ok(recvd_point) => {
                        match recvd_point.clone().value {
                            None => {
                                info!("{log_prefix}: Received none on match recvd_point.clone().value -- is this ok?")
                            }
                            Some(val) => {
                                let v = recvd_point.clone();
                                let payloads = generate_payloads(
                                    unit,
                                    Some(&recvd_point),
                                    &requested_point_to_check,
                                    Some(&val),
                                )
                                .await;

                                last_report.insert(uniqueid, Utc::now());
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
                                    if let Err(e) = cull_records_to(
                                        payload.config.clone().unique_id,
                                        CULL_HISTORY_ROWS,
                                    )
                                    .await
                                    {
                                        warn!("{log_prefix}: Couldn't cull history for this point: {e}");
                                    };
                                    if let Err(e) =
                                        write_payload_history(payload.config, payload.state).await
                                    {
                                        warn!("{log_prefix}: Unable to store value in db: {e}");
                                    }
                                }
                            }
                        }
                    }
                }
            }
            //endregion
        }

        debug!(%sn, "Device tick");
        let _ = sleep(Duration::from_secs(MINIMUM_QUERY_INTERVAL_SECS.into())).await;
    }
}
