use crate::pwrcell_unit::PWRCellUnit;
use crate::monitored_point::MonitoredPoint;
use crate::SETTINGS;
use config::Value;
use sunspec_rs::sunspec_data::ResponseType;
use tokio::time::{Instant, Duration, sleep};

const MINIMUM_POLL_INTERVAL: u16 = 15_u16;

pub async fn poll_loop(unit: &PWRCellUnit) {
    let rcpn = &unit.rcp_number;
    let config = SETTINGS.read().await;
    let mut points: Vec<MonitoredPoint> = vec![];
    let mut point: String = String::default();
    let mut interval: u64 = 0;
    for (id, _) in unit.conn.models.iter() {
        let mname = format!("models.{id}");
        let modelpoints = match config.get_array(&mname) {
            Ok(m) => m,
            Err(e) => {
                warn!(%rcpn, "Can't find any model points defined for models.{id} in config: {e}");
                continue;
            }
        };
        for pointname in modelpoints.iter() {
            match config.get_array(&mname) {
                Ok(m) => {
                    for item in m.iter() {
                        let map = pointname.clone().into_table().unwrap();
                        let point = match map.get("point") {
                            Some(p) => {
                                let fuck = p.clone();
                                fuck.into_string().unwrap()
                            }
                            None => {
                                warn!(%rcpn, "model {id} missing point def in config, skipping point");
                                continue;
                            }
                        };
                        let interval = match map.get("interval") {
                            Some(i) => {
                                let fuck = i.clone();
                                fuck.into_uint().unwrap()
                            }
                            None => {
                                warn!(%rcpn, "model {id} missing interval def in config, skipping point");
                                continue;
                            }
                        };


                        match MonitoredPoint::new(id.to_string(), point.clone(), Duration::from_secs(interval)) {
                            Ok(p) => points.push(p),
                            Err(e) => {
                                warn!(%rcpn, "unable to create MonitoredPoint for {id}/{point}: {e}");
                                continue;
                            }
                        };
                    }
                    info!("Points: {:#?}", points);
                }
                Err(e) => {
                    warn!(%rcpn, "Can't find any model points defined for models.{id} in config: {e}");
                }
            }
        }
    }


    loop {
        for p in points.iter() {
            let model = p.model.clone();
            let point_name = p.name.clone();
            let md = unit.conn.models.get(&p.model.parse::<u16>().unwrap()).unwrap();
            match unit.conn.clone().get_point(md.clone(), &p.name).await {
                None => {}
                Some(v) => {
                    match v.value {
                        None => {}
                        Some(val) => {
                            match val {
                                ResponseType::String(str) => {
                                    info!("Response for {model}/{point_name}: {str}")
                                }
                                ResponseType::Integer(int) => {
                                    info!("Response for {model}/{point_name}: {int}")
                                }
                                ResponseType::Float(float) => {
                                    info!("Response for {model}/{point_name}: {0:.1}", float)
                                }
                                ResponseType::Boolean(boolean) => {
                                    info!("Response for {model}/{point_name}: {boolean}")
                                }
                                ResponseType::Array(vec) => {
                                    info!("Response for {model}/{point_name}: {:#?}", vec)
                                }
                            }
                        }
                    }
                }
            }
        }

        info!(%rcpn, "Device tick");
        let _ = sleep(Duration::from_secs(MINIMUM_POLL_INTERVAL.into())).await;
    }
}

