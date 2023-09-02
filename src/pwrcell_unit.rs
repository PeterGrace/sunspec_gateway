use sunspec_rs::sunspec::{ModelData, SunSpecConnection};
use anyhow::{Result, bail};
use sunspec_rs::sunspec_data::{Model, ResponseType, SunSpecData};
use crate::monitored_point::MonitoredPoint;


const COMMON_MODEL_ID: u16 = 1_u16;

#[derive(Debug)]
pub struct PWRCellUnit {
    pub r#type: String,
    pub addr: String,
    pub slave_id: u8,
    pub conn: SunSpecConnection,
    pub data: SunSpecData,
    pub points: Vec<MonitoredPoint>,
    pub rcp_number: Option<String>,
}

impl PWRCellUnit {
    pub async fn new(type_name: String, addr: String, slave_id: String) -> anyhow::Result<Self> {
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
        let mut data: SunSpecData = SunSpecData::default();
        conn.models = conn.clone().populate_models(data.clone()).await;

        let mut common = match conn.models.get(&COMMON_MODEL_ID) {
            None => {
                bail!("Couldn't get model definition for common");
            }
            Some(m) => m
        };
        let mut rcp_number: Option<String> = None;
        let rcp_number_p = conn.clone().get_point(common.clone(), "SN").await;
        if let ResponseType::String(rcpn) = rcp_number_p.unwrap().value.unwrap() {
            rcp_number = Some(rcpn);
        }
        info!("Initialized PWRCellUnit with SN {}",rcp_number.clone().unwrap());

        Ok(PWRCellUnit {
            r#type: type_name,
            slave_id: sid,
            conn,
            data,
            addr,
            points: vec![],
            rcp_number
        })
    }
    pub async fn poll_loop() {}
}