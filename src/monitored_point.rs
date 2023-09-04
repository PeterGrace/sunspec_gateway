use std::time::Duration;


#[derive(Debug)]
pub struct MonitoredPoint {
    pub model: String,
    pub name: String,
    pub interval: Duration,
    pub device_class: Option<String>,
    pub state_class: Option<String>,
    pub uom: Option<String>,
    pub precision: Option<u8>,
}

impl MonitoredPoint {
    pub fn new(model: String,
               name: String,
               interval: Duration,
               device_class: Option<String>,
               state_class: Option<String>,
               precision: Option<u8>) -> anyhow::Result<Self> {
        info!("Creating a monitoredpoint for {model}/{name}");

        Ok(
            MonitoredPoint {
                model,
                name,
                interval,
                device_class,
                state_class,
                uom: None,
                precision,
            }
        )
    }
}