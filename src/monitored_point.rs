use chrono::{DateTime, Utc};

// we won't let points get checked faster than every 10 seconds.
// if we change this, the modbus could get saturated very quickly
const LOWER_LIMIT_INTERVAL: u64 = 10_u64;

#[derive(Debug)]
pub struct MonitoredPoint {
    pub model: String,
    pub name: String,
    pub interval: u64,
    pub device_class: Option<String>,
    pub state_class: Option<String>,
    pub uom: Option<String>,
    pub precision: Option<u8>,
    pub last_report: DateTime<Utc>,
    pub homeassistant_discovery: bool,
}

impl MonitoredPoint {
    pub fn new(
        model: String,
        name: String,
        interval: u64,
        device_class: Option<String>,
        state_class: Option<String>,
        precision: Option<u8>,
        uom: Option<String>,
        homeassistant: bool,
    ) -> anyhow::Result<Self> {
        info!("Creating a monitoredpoint for {model}/{name}");

        let mut interval_checked: u64 = 0_u64;
        if interval < LOWER_LIMIT_INTERVAL {
            interval_checked = LOWER_LIMIT_INTERVAL
        } else {
            interval_checked = interval
        }

        Ok(MonitoredPoint {
            model,
            name,
            interval: interval_checked,
            device_class,
            state_class,
            uom,
            precision,
            last_report: Utc::now(),
            homeassistant_discovery: homeassistant,
        })
    }
}
