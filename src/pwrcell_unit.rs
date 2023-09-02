use sunspec_rs::sunspec::{ModelData, SunSpecConnection};
use anyhow::{Result, bail};
use sunspec_rs::sunspec_data::{Model, ResponseType, SunSpecData};
use crate::monitored_point::MonitoredPoint;
use strum_macros::{EnumString, Display};
use std::str::FromStr;
use config::Value;
use tokio::time::{Instant, Duration,sleep};
use crate::SETTINGS;

const COMMON_MODEL_ID: u16 = 1_u16;
const MINIMUM_POLL_INTERVAL: u16 = 15_u16;

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

pub async fn poll_loop(unit: &PWRCellUnit) {
    let rcpn = &unit.rcp_number;
    let config = SETTINGS.read().await;
    let mut points: Vec<MonitoredPoint> = vec![];
    for (id, _) in unit.conn.models.iter() {
        match config.get_table(format!("models.{id}").as_str()) {
            Ok(m) => {
                let point = match m.get("point") {
                    Some(p) => {
                        let fuck = p.clone();
                        fuck.into_string().unwrap()
                    },
                    None => {
                        warn!(%rcpn, "model {id} missing point def in config, skipping point");
                        continue;
                    }
                };
                let interval = match m.get("interval") {
                    Some(i) => {
                        let fuck = i.clone();
                        fuck.into_uint().unwrap()
                    },
                    None => {
                        warn!(%rcpn, "model {id} missing interval def in config, skipping point");
                        continue;
                    }
                };
                match MonitoredPoint::new(id.to_string(), point.clone(), Duration::from_secs(interval)){
                    Ok(p) => points.push(p),
                    Err(e) => {
                        warn!(%rcpn, "unable to create MonitoredPoint for {id}/{point}: {e}");
                        continue;
                    }
                }

            },
            Err(e) => {
                warn!(%rcpn, "Can't find any model points defined for models.{id} in config");
            }
        }


    }


    loop {
        for p in points.iter() {
            //let val = unit.conn.get_point(p.)
        }

        info!(%rcpn, "Device tick");
        let _ = sleep(Duration::from_secs(MINIMUM_POLL_INTERVAL.into())).await;
    }
}