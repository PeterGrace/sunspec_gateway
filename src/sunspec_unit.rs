use crate::ipc::DeviceInfo;
use crate::monitored_point::MonitoredPoint;
use anyhow::bail;
use std::collections::HashMap;
use std::str::FromStr;
use sunspec_rs::model_data::ModelData;
use sunspec_rs::sunspec_connection::{SunSpecConnection, SunSpecPointError};
use sunspec_rs::sunspec_data::SunSpecData;
use sunspec_rs::sunspec_models::ValueType;

const COMMON_MODEL_ID: u16 = 1_u16;

#[derive(Debug)]
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
    pub async fn new(addr: String, slave_id: String) -> anyhow::Result<Self> {
        let sid: u8 = match slave_id.parse() {
            Ok(id) => id,
            Err(e) => {
                bail!("Couldn't parse slave_id {slave_id}: {e}");
            }
        };
        let mut conn = match SunSpecConnection::new(addr.clone(), Some(sid), false).await {
            Ok(c) => c,
            Err(e) => {
                bail!("Couldn't create connection to {addr} - {sid}: {e}");
            }
        };
        let data: SunSpecData = SunSpecData::default();
        match conn.clone().populate_models(&data).await {
            Ok(m) => conn.models = m.clone(),
            Err(e) => bail!("Can't populate models: {e}"),
        };

        let mut common = match conn.models.get(&COMMON_MODEL_ID) {
            None => {
                bail!("Couldn't get model definition for common");
            }
            Some(m) => m,
        };
        let mut device_info = DeviceInfo::default();

        if let Ok(firmware) = conn.clone().get_point(common.clone(), "Vr").await {
            if let Some(value) = firmware.value {
                if let ValueType::String(ver) = value {
                    device_info.sw_version = ver;
                }
            }
        }
        let manufacturer: String = match conn.clone().get_point(common.clone(), "Mn").await {
            Ok(p) => {
                if let ValueType::String(str) = p.value.unwrap() {
                    str
                } else {
                    anyhow::bail!("Received a point that wasn't a string for manufacturer.");
                }
            }
            Err(e) => {
                anyhow::bail!("fatal error, aborting: {e}")
            }
        };
        let serial_number = match conn.clone().get_point(common.clone(), "SN").await {
            Ok(p) => {
                if let ValueType::String(str) = p.value.unwrap() {
                    str
                } else {
                    anyhow::bail!("Received a point that wasn't a string for serial number.");
                }
            }
            Err(e) => anyhow::bail!(e),
        };
        let physical_model = match conn.clone().get_point(common.clone(), "Md").await {
            Ok(p) => {
                if let ValueType::String(str) = p.value.unwrap() {
                    str
                } else {
                    anyhow::bail!(
                        "Received a point that wasn't a string for physical device model name."
                    );
                }
            }
            Err(e) => anyhow::bail!(e),
        };
        device_info.name = physical_model.clone();
        device_info.manufacturer = manufacturer.clone();
        device_info.identifiers = vec![serial_number.clone()];

        info!("Initialized {manufacturer}/{physical_model} with SN {serial_number}");

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
