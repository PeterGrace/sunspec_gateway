use std::time::Duration;
use sunspec_rs::sunspec_data::Point;
use anyhow::Result;
use sunspec_rs::sunspec::ModelData;

#[derive(Debug)]
pub struct MonitoredPoint {
    model: String,
    name: String,
    point: Point,
    interval: Duration,
    topic: String,
}

impl MonitoredPoint {
    pub fn new(model: String, name: String, interval: Duration) -> anyhow::Result<Self> {
        let point: Point = Point::default();
        info!("Creating a monitoredpoint for {model}/{name}");

        Ok(
            MonitoredPoint {
                model,
                name,
                interval,
                point,
                topic: String::default()
            }
        )
    }
}