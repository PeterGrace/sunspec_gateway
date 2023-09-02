use std::time::Duration;
use sunspec_rs::sunspec_data::{Point};


#[derive(Debug)]
pub struct MonitoredPoint {
    pub model: String,
    pub name: String,
    pub interval: Duration,
    pub topic: String,
}

impl MonitoredPoint {
    pub fn new(model: String, name: String, interval: Duration) -> anyhow::Result<Self> {

        info!("Creating a monitoredpoint for {model}/{name}");

        Ok(
            MonitoredPoint {
                model,
                name,
                interval,
                topic: String::default(),
            }
        )
    }
}