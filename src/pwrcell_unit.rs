use sunspec_rs::sunspec::{ModelData, SunSpecConnection};
use anyhow::{Result, bail};
use sunspec_rs::sunspec_data::{Model, Point, ResponseType, SunSpecData};
use crate::monitored_point::MonitoredPoint;
use strum_macros::{EnumString, Display};
use std::str::FromStr;


const COMMON_MODEL_ID: u16 = 1_u16;


#[derive(EnumString, Debug, Eq, PartialEq, Display)]
#[strum(ascii_case_insensitive)]
pub enum PWRCellUnitClass {
    #[strum(serialize = "REbus Beacon")]
    Beacon,
    ICM,
    #[strum(serialize = "PWRcell XVT076A03 Inverter")]
    Inverter,
    #[strum(serialize = "PWRcell Battery")]
    Battery,
    #[strum(serialize = "PV Link")]
    PVLink,
    None
}

#[derive(Debug)]
pub struct PWRCellUnit {
    pub class: PWRCellUnitClass,
    pub addr: String,
    pub slave_id: u8,
    pub conn: SunSpecConnection,
    pub data: SunSpecData,
    pub points: Vec<MonitoredPoint>,
    pub rcp_number: String,
}

impl PWRCellUnit {
    pub async fn new(addr: String, slave_id: String) -> anyhow::Result<Self> {
        let sid: u8 = match slave_id.parse() {
            Ok(id) => id,
            Err(e) => {
                bail!("Couldn't parse slave_id {slave_id}: {e}");
            }
        };
        let mut conn = match SunSpecConnection::new(addr.clone(), Some(sid)).await {
            Ok(c) => c,
            Err(e) => {
                bail!("Couldn't create connection to {addr} - {sid}: {e}");
            }
        };
        let data: SunSpecData = SunSpecData::default();
        conn.models = conn.clone().populate_models(data.clone()).await;

        let mut common = match conn.models.get(&COMMON_MODEL_ID) {
            None => {
                bail!("Couldn't get model definition for common");
            }
            Some(m) => m
        };
        let mut rcp_number: String = String::default();
        let rcp_number_p = conn.clone().get_point(common.clone(), "SN").await;
        let o_model = conn.clone().get_point(common.clone(), "Md").await;
        let mut class: PWRCellUnitClass = PWRCellUnitClass::None;
        if let Some(model) = o_model {
            if let Some(response_type) = model.value {
                if let ResponseType::String(class_name) = response_type {
                    let p = class_name.clone();
                    let mut class_str = p.to_ascii_lowercase();
                    class_str = class_str.trim_matches(char::from(0)).parse()?;
                    class = PWRCellUnitClass::from_str(&class_str).unwrap();
                }
            }
        }
        if let ResponseType::String(rcpn) = rcp_number_p.unwrap().value.unwrap() {
            rcp_number = rcpn;
        }
        info!("Initialized PWRCellUnit (class {class}) with SN {}",rcp_number.clone());

        Ok(PWRCellUnit {
            class: PWRCellUnitClass::Beacon,
            slave_id: sid,
            conn,
            data,
            addr,
            points: vec![],
            rcp_number,
        })
    }

}

