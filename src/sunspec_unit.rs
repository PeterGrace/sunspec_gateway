use crate::consts::*;
use crate::monitored_point::MonitoredPoint;
use crate::payload::DeviceInfo;
use crate::{GatewayError, SHUTDOWN};
use anyhow::bail;

use std::time::Duration;

use sunspec_rs::sunspec_connection::SunSpecConnection;
use sunspec_rs::sunspec_data::SunSpecData;
use sunspec_rs::sunspec_models::{PointIdentifier, ValueType};
use tokio::task;
use tokio::time::sleep;

#[derive(Clone, Debug)]
pub struct SunSpecUnit {
    pub addr: String,
    pub slave_id: u8,
    pub conn: SunSpecConnection,
    pub data: SunSpecData,
    pub points: Vec<MonitoredPoint>,
    pub serial_number: String,
    pub device_info: DeviceInfo,
}

impl SunSpecUnit {
    pub async fn new(addr: String, slave_id: String) -> Result<Self, GatewayError> {
        let sid: u8 = match slave_id.parse() {
            Ok(id) => id,
            Err(e) => {
                return Err(GatewayError::Error(format!(
                    "Couldn't parse slave_id {slave_id}: {e}"
                )));
            }
        };
        let mut conn = match SunSpecConnection::new(addr.clone(), Some(sid), false).await {
            Ok(c) => c,
            Err(e) => {
                return Err(GatewayError::Error(format!(
                    "Couldn't create connection to {addr} - {sid}: {e}"
                )));
            }
        };
        let data: SunSpecData = SunSpecData::default();

        let populate_conn = conn.clone();
        let populate_data = data.clone();
        let task = task::spawn(async move {
            match populate_conn.populate_models(&populate_data).await {
                Ok(m) => Ok(m.clone()),
                Err(e) => bail!("Can't populate models: {e}"),
            }
        });

        while !task.is_finished() {
            if let Some(shutting_down) = SHUTDOWN.get() {
                if *shutting_down {
                    info!("Shutdown received while populate_models running, exiting now");
                    task.abort();
                    return Err(GatewayError::ExitingThread);
                }
            }
            sleep(Duration::from_millis(GENERIC_WAIT_MILLIS)).await;
        }
        match task.await {
            Ok(taskresult) => match taskresult {
                Ok(model) => conn.models = model.clone(),
                Err(e) => {
                    return Err(GatewayError::Error(format!("{e}")));
                }
            },
            Err(e) => {
                return Err(GatewayError::Error(format!("{e}")));
            }
        }
        let common = match conn.models.get(&COMMON_MODEL_ID) {
            None => {
                return Err(GatewayError::Error(format!(
                    "Couldn't get model definition for common"
                )));
            }
            Some(m) => m,
        };
        let mut device_info = DeviceInfo::default();
        let serial_number = match conn
            .clone()
            .get_point(common.clone(), PointIdentifier::Point("SN".to_string()))
            .await
        {
            Ok(p) => {
                if let ValueType::String(str) = p.value.unwrap() {
                    str
                } else {
                    return Err(GatewayError::Error(format!(
                        "Received a point that wasn't a string for serial number."
                    )));
                }
            }
            Err(e) => {
                return Err(GatewayError::Error(format!("{e}")));
            }
        };

        if let Ok(firmware) = conn
            .clone()
            .get_point(common.clone(), PointIdentifier::Point("Vr".to_string()))
            .await
        {
            if let Some(value) = firmware.value {
                if let ValueType::String(ver) = value {
                    device_info.sw_version = ver;
                }
            }
        }
        let manufacturer: String = match conn
            .clone()
            .get_point(common.clone(), PointIdentifier::Point("Mn".to_string()))
            .await
        {
            Ok(p) => {
                if let ValueType::String(str) = p.value.unwrap() {
                    str
                } else {
                    return Err(GatewayError::Error(format!(
                        "Received a point that wasn't a string for manufacturer."
                    )));
                }
            }
            Err(e) => {
                return Err(GatewayError::Error(format!("fatal error, aborting: {e}")));
            }
        };
        let physical_model = match conn
            .clone()
            .get_point(common.clone(), PointIdentifier::Point("Md".to_string()))
            .await
        {
            Ok(p) => {
                if let ValueType::String(str) = p.value.unwrap() {
                    str
                } else {
                    return Err(GatewayError::Error(format!(
                        "Received a point that wasn't a string for physical device model name."
                    )));
                }
            }
            Err(e) => return Err(GatewayError::Error(format!("{e}"))),
        };
        device_info.model = physical_model.clone();
        device_info.manufacturer = manufacturer.clone();
        device_info.name = format!("{serial_number}: {physical_model}");
        device_info.identifiers = vec![serial_number.clone()];

        info!("Initialized {manufacturer}/{physical_model} with SN {serial_number}");
        debug!("Models: {:#?}", conn.models.keys());

        Ok(SunSpecUnit {
            slave_id: sid,
            conn,
            data,
            addr,
            points: vec![],
            serial_number,
            device_info,
        })
    }
}
