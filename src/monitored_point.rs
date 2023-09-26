use crate::config_structs::{InputType, PointConfig};
use chrono::{DateTime, Utc};
use std::ffi::CString;
use sunspec_rs::sunspec_models::Access;

// we won't let points get checked faster than every 10 seconds.
// if we change this, the modbus could get saturated very quickly
const LOWER_LIMIT_INTERVAL: u64 = 10_u64;

#[derive(Debug)]
pub struct MonitoredPoint {
    /// model number or name
    pub model: String,
    /// point name in model (Evt, Status, Ena, V, Mn, etc)
    pub name: String,
    /// how frequently this point should be measured
    pub interval: u64,
    /// the device class homeassistant should use (current, voltage, power, energy, etc)
    pub device_class: Option<String>,
    /// the homeassistant state class (measurement, total_increasing, etc)
    pub state_class: Option<String>,
    /// The unit of measure to be reported in mqtt packet
    pub uom: Option<String>,
    /// the digits of precision after the decimal point for measured point
    pub precision: Option<u8>,
    /// whether or not to report this datapoint in homeassistant autodiscovery
    pub homeassistant_discovery: bool,
    /// whether this point is writeable
    pub write_mode: Access,
    /// The preferred name for the point
    pub display_name: Option<String>,
    /// the scale factor to apply, if necessary
    pub scale_factor: Option<i32>,
    /// what type of input is needed to send write commands on the topic
    pub input_type: Option<InputType>,
}

impl MonitoredPoint {
    pub fn new(model: String, pc: PointConfig) -> anyhow::Result<Self> {
        debug!("Creating a monitoredpoint for {model}/{}", pc.point);

        let mut interval_checked: u64 = 0_u64;
        if pc.interval < LOWER_LIMIT_INTERVAL {
            interval_checked = LOWER_LIMIT_INTERVAL
        } else {
            interval_checked = pc.interval
        }
        let write_mode = match pc.readwrite {
            None => Access::ReadOnly,
            Some(v) => {
                if v {
                    Access::ReadWrite
                } else {
                    Access::ReadOnly
                }
            }
        };
        let homeassistant_discovery = match pc.homeassistant {
            None => true,
            Some(v) => v,
        };

        Ok(MonitoredPoint {
            model,
            display_name: pc.display_name,
            name: pc.point,
            interval: interval_checked,
            device_class: pc.device_class,
            state_class: pc.state_class,
            uom: pc.uom,
            precision: pc.precision,
            homeassistant_discovery,
            write_mode,
            scale_factor: pc.scale_factor,
            input_type: pc.inputs,
        })
    }
}
