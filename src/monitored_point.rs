use std::time::Duration;
use sunspec_rs::sunspec_data::Point;
use anyhow::Result;

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
        let topic: String = String::from("wee");


        Ok(
            MonitoredPoint {
                model,
                name,
                interval,
                point,
                topic,
            }
        )
    }
}