use crate::config_structs::{InputType, PointConfig};
use crate::consts::*;
use anyhow::bail;

use sunspec_rs::sunspec_models::{Access, PointIdentifier};

#[derive(Debug, Clone)]
pub struct MonitoredPoint {
    /// model number or name
    pub model: String,
    /// point name in model (Evt, Status, Ena, V, Mn, etc)
    pub name: PointIdentifier,
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
    /// this point is only for input and cannot be read.
    pub input_only: Option<bool>,
    /// the minimum value we expect to see for this point.
    pub value_min: Option<f64>,
    /// the maximum value we expect to see for this point.
    pub value_max: Option<f64>,
    /// how many standard deviations we'll allow before considering value nonsensical
    pub check_deviations: Option<u16>,
    pub this_address: Option<u16>,
}

impl MonitoredPoint {
    pub fn new(model: String, pc: PointConfig, hass_enabled: Option<bool>) -> anyhow::Result<Self> {
        debug!("Creating a monitoredpoint for {model}/{}", pc.name());

        let interval_checked: u64;
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
        let homeassistant_discovery = {
            if let Some(v) = hass_enabled {
                match v {
                    true => pc.homeassistant.unwrap_or_else(|| true),
                    false => false,
                }
            } else {
                pc.homeassistant.unwrap_or_else(|| true)
            }
        };

        if pc.point.is_none() && pc.catalog_ref.is_none() {
            let msg = format!("There is a defined point in model {model} that has neither point name nor catalog ref.  Skipping.");
            error!(msg);
            bail!(msg);
        }
        let mut monitored_point_target: PointIdentifier = {
            if pc.catalog_ref.is_some() {
                PointIdentifier::Catalog(pc.catalog_ref.clone().unwrap())
            } else {
                PointIdentifier::Point(pc.point.clone().unwrap())
            }
        };
        Ok(MonitoredPoint {
            model,
            display_name: pc.display_name,
            name: monitored_point_target,
            interval: interval_checked,
            device_class: pc.device_class,
            state_class: pc.state_class,
            uom: pc.uom,
            precision: pc.precision,
            homeassistant_discovery,
            write_mode,
            scale_factor: pc.scale_factor,
            input_type: pc.inputs,
            input_only: pc.input_only,
            value_min: pc.value_min,
            value_max: pc.value_max,
            check_deviations: pc.check_deviations,
            this_address: None,
        })
    }
}
