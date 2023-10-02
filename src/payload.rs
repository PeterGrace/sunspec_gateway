use crate::config_structs::InputType;
use crate::consts::*;
use crate::monitored_point::MonitoredPoint;
use crate::state_mgmt::{check_needs_adjust, get_history};
use crate::sunspec_unit::SunSpecUnit;
use chrono::{DateTime, Utc};
use num_traits::pow::Pow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sunspec_rs::sunspec_models::{Access, Point, ValueType};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeviceInfo {
    pub identifiers: Vec<String>,
    pub manufacturer: String,
    pub name: String,
    pub model: String,
    pub sw_version: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(untagged)]
pub enum PayloadValueType {
    Float(f32),
    Int(i64),
    String(String),
    Boolean(bool),
    #[default]
    None,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    Config(HAConfigPayload),
    CurrentState(StatePayload),
    #[default]
    None,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityCategory {
    Config,
    #[default]
    Diagnostic,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HAConfigPayload {
    pub name: String,
    pub device: DeviceInfo,
    pub unique_id: String,
    pub entity_id: String,
    pub state_topic: String,
    pub expires_after: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_category: Option<EntityCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_off: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "unit_of_measurement")]
    pub native_uom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_display_precision: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumed_state: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_state_attributes: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_entity_name: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub should_poll: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_press: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePayload {
    pub value: PayloadValueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(with = "crate::date_serializer")]
    pub last_seen: DateTime<Utc>,
}

impl Default for StatePayload {
    fn default() -> Self {
        StatePayload {
            value: PayloadValueType::None,
            last_seen: Utc::now(),
            description: None,
            label: None,
            notes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompoundPayload {
    pub(crate) config: HAConfigPayload,
    pub(crate) config_topic: String,
    pub(crate) state: StatePayload,
    pub(crate) state_topic: String,
}

pub async fn generate_payloads(
    unit: &SunSpecUnit,
    point_data: Option<&Point>,
    monitored_point: &MonitoredPoint,
    val: Option<&ValueType>,
) -> Vec<CompoundPayload> {
    let sn = unit.serial_number.clone();
    let model = monitored_point.model.clone();
    let point_name = monitored_point.name.clone();
    let log_prefix = format!(
        "[{}:{} {sn} {model}/{point_name}]",
        unit.addr, unit.slave_id
    );
    let mut config_payload: HAConfigPayload = HAConfigPayload::default();
    let mut state_payload: StatePayload = StatePayload::default();
    if let Some(display_name) = monitored_point.display_name.clone() {
        config_payload.name = display_name;
    } else {
        config_payload.name = format!("{model}-{point_name}");
    }
    config_payload.device_class = monitored_point.device_class.clone();
    config_payload.state_class = monitored_point.state_class.clone();
    config_payload.expires_after = 300;
    config_payload.value_template = Some("{{ value_json.value }}".to_string());
    config_payload.unique_id = format!("{sn}.{model}.{point_name}");
    config_payload.entity_id = format!("sensor.{model}_{point_name}");
    config_payload.device = unit.device_info.clone();
    if val.is_some() && point_data.is_some() {
        match val.unwrap() {
            ValueType::String(str) => {
                debug!("Response for {model}/{point_name}: {str}");
                state_payload.value = PayloadValueType::String(str.to_owned())
            }
            ValueType::Integer(int) => {
                debug!("Response for {model}/{point_name}: {int}");
                if monitored_point.uom.is_some() {
                    // we are overriding the default uom from config
                    config_payload.native_uom = monitored_point.uom.clone();
                } else {
                    config_payload.native_uom = point_data.unwrap().units.clone();
                }

                // if we are employing a scale factor on an int, it becomes a float
                if let Some(scale) = monitored_point.scale_factor {
                    let mut scaled_value: f32 = 0.0;
                    if scale >= 0 {
                        scaled_value = (*int as f32 * (f32::pow(10.0, scale.abs() as f32))).into();
                    } else {
                        scaled_value = (*int as f32 / (f32::pow(10.0, scale.abs() as f32))).into();
                    }
                    if monitored_point.precision.is_some() {
                        config_payload.suggested_display_precision = monitored_point.precision;
                    } else {
                        config_payload.suggested_display_precision = DEFAULT_DISPLAY_PRECISION;
                    };
                    if let Some(minimum) = monitored_point.value_min {
                        if scaled_value < minimum {
                            warn!("{log_prefix}: {scaled_value} is less than the minimum value specified ({minimum})");
                            return vec![];
                        }
                    }
                    if let Some(maximum) = monitored_point.value_max {
                        if scaled_value > maximum {
                            warn!("{log_prefix}: {scaled_value} is greater than the maximum value specified ({maximum})");
                            return vec![];
                        }
                    }
                    state_payload.value = PayloadValueType::Float(scaled_value)
                } else {
                    state_payload.value = PayloadValueType::Int(*int as i64)
                }
            }
            ValueType::Float(float) => {
                debug!("Response for {model}/{point_name}: {0:.1}", float);
                if monitored_point.precision.is_some() {
                    config_payload.suggested_display_precision = monitored_point.precision;
                } else {
                    config_payload.suggested_display_precision = DEFAULT_DISPLAY_PRECISION;
                };
                if monitored_point.uom.is_some() {
                    // we are overriding the default uom from config
                    config_payload.native_uom = monitored_point.uom.clone();
                } else {
                    config_payload.native_uom = point_data.unwrap().units.clone();
                }
                let mut scaled_value: f32 = 0.0;
                if let Some(scale) = monitored_point.scale_factor {
                    if scale >= 0 {
                        scaled_value = (float * f32::pow(10.0, scale.abs() as f32)).into();
                    } else {
                        scaled_value = (float / f32::pow(10.0, scale.abs() as f32)).into();
                    }
                } else {
                    scaled_value = *float;
                }
                if let Some(minimum) = monitored_point.value_min {
                    if scaled_value < minimum {
                        warn!("{log_prefix}: {scaled_value} is less than the minimum value specified ({minimum})");
                        return vec![];
                    }
                }
                if let Some(maximum) = monitored_point.value_max {
                    if scaled_value > maximum {
                        warn!("{log_prefix}: {scaled_value} is greater than the maximum value specified ({maximum})");
                        return vec![];
                    }
                }
                match get_history(format!("{sn}.{model}.{point_name}")).await {
                    Ok(ag) => {
                        let mut deviations: u16 = CHECK_DEVIATIONS_COUNT;
                        if monitored_point.check_deviations.is_some() {
                            deviations = monitored_point.check_deviations.unwrap();
                        }
                        let mut stdev_checked: f32 = 0.0;
                        if ag.stdev.abs() < 1.0 {
                            stdev_checked = ag.stdev.abs() + 1.0
                        } else {
                            stdev_checked = ag.stdev.abs()
                        }

                        if scaled_value < ag.min {
                            // our point is lower than the lowest seen so far
                            if scaled_value.abs() > (ag.median + stdev_checked * deviations as f32)
                            {
                                warn!("{log_prefix}: {scaled_value} is more than {deviations} deviations away from median. {ag:#?}");
                            }
                        }
                        if scaled_value > ag.max {
                            // our point is the highest point measured so far
                            if scaled_value.abs() > (ag.median + stdev_checked * deviations as f32)
                            {
                                warn!("{log_prefix}: {scaled_value} is more than {deviations} deviations away from median. {ag:#?}");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("{log_prefix}: Error retrieving historical results, assuming point is valid: {e}");
                    }
                };

                state_payload.value = PayloadValueType::Float(scaled_value);
                if let Some(literal) = &point_data.unwrap().literal {
                    if literal.label.is_some() {
                        config_payload.name = literal.label.clone().unwrap();
                    }
                    state_payload.label = literal.clone().label;
                    state_payload.description = literal.clone().description;
                    state_payload.notes = literal.clone().notes;
                }
            }
            ValueType::Boolean(boolean) => {
                debug!("Response for {model}/{point_name}: {boolean}");
                state_payload.value = PayloadValueType::String(boolean.to_string());
            }
            ValueType::Array(vec) => {
                // we have an array of strings, we need to make binary sensors.
                let string_on: String = String::from("on");
                let string_off: String = String::from("off");

                let mut payloads: Vec<CompoundPayload> = vec![];
                let mut updated_uniques: Vec<String> = vec![];
                for state in vec {
                    // clone preexisiting objects
                    let mut config_payload = config_payload.clone();
                    let mut state_payload = state_payload.clone();
                    // configure this point's state addresses
                    let config_topic = format!(
                        "homeassistant/binary_sensor/{sn}/{model}_{point_name}_{state}/config"
                    );
                    let state_topic = format!("sunspec_gateway/{sn}/{model}/{point_name}_{state}");
                    config_payload.unique_id = format!("{sn}.{model}.{point_name}.{state}");
                    config_payload.entity_id =
                        format!("binary_sensor.{model}_{point_name}_{state}");
                    config_payload.name = format!("{model}/{point_name}: {state}");
                    config_payload.state_topic = state_topic.clone();
                    config_payload.payload_on = Some(string_on.clone());
                    config_payload.payload_off = Some(string_off.clone());
                    state_payload.value = PayloadValueType::String(string_on.clone());
                    payloads.push(CompoundPayload {
                        state_topic,
                        config_topic,
                        config: config_payload.clone(),
                        state: state_payload.clone(),
                    });
                    updated_uniques.push(config_payload.unique_id.clone());
                }
                // now, lets set state to off for points we didn't see
                match check_needs_adjust(updated_uniques).await {
                    Ok(stale) => {
                        if stale.len() > 0 {
                            info!(
                                "{log_prefix}: sending off for {} binary sensors",
                                stale.len()
                            );
                        }
                        for stale_unique in stale {
                            let mut splitval = stale_unique.splitn(4, ".");
                            let (sn, model, point_name, state) = (
                                splitval.next().unwrap().to_string(),
                                splitval.next().unwrap().to_string(),
                                splitval.next().unwrap().to_string(),
                                splitval.next().unwrap().to_string(),
                            );
                            // clone preexisiting objects
                            let mut config_payload = config_payload.clone();
                            let mut state_payload = state_payload.clone();
                            // configure this point's state addresses
                            let config_topic = format!(
                                "homeassistant/binary_sensor/{sn}/{model}_{point_name}_{state}/config"
                            );
                            let state_topic =
                                format!("sunspec_gateway/{sn}/{model}/{point_name}_{state}");
                            config_payload.unique_id = format!("{sn}.{model}.{point_name}.{state}");
                            config_payload.entity_id =
                                format!("binary_sensor.{model}_{point_name}_{state}");
                            config_payload.name = format!("{model}/{point_name}: {state}");
                            config_payload.state_topic = state_topic.clone();
                            config_payload.payload_on = Some(string_on.clone());
                            config_payload.payload_off = Some(string_off.clone());
                            state_payload.value = PayloadValueType::String(string_off.clone());
                            payloads.push(CompoundPayload {
                                state_topic,
                                config_topic,
                                config: config_payload,
                                state: state_payload,
                            });
                        }
                    }
                    Err(e) => {
                        warn!("{e}");
                    }
                };

                return payloads;
                // debug!("Response for {model}/{point_name}: {:#?}", vec);
                // let concat = vec.join(", ");
                // state_payload.value = PayloadValueType::String(concat)
            }
        }
    }

    let mut config_topic: String = format!("homeassistant/sensor/{sn}/{model}_{point_name}/config");
    if matches!(monitored_point.write_mode, Access::ReadWrite) {
        config_payload.command_topic =
            Some(format!("sunspec_gateway/input/{sn}/{model}/{point_name}"));
        match &monitored_point.input_type {
            Some(input) => match input {
                InputType::Select(options) => {
                    config_payload.options = Some(options.to_vec());
                    config_payload.entity_category = Some(EntityCategory::Config);
                    config_payload.entity_id = format!("select.{model}_{point_name}");
                    config_topic = format!("homeassistant/select/{sn}/{model}_{point_name}/config");
                }
                InputType::Switch(switch) => {
                    let switch = switch.clone();
                    config_payload.payload_on = Some(switch.on);
                    config_payload.payload_off = Some(switch.off);
                    config_payload.entity_category = Some(EntityCategory::Config);
                    config_payload.entity_id = format!("switch.{model}_{point_name}");
                    config_topic = format!("homeassistant/switch/{sn}/{model}_{point_name}/config");
                }
                InputType::Button(button) => {
                    config_payload.payload_press = Some(button.to_string());
                    config_payload.entity_category = Some(EntityCategory::Config);
                    config_payload.entity_id = format!("button.{model}_{point_name}");
                    config_topic = format!("homeassistant/button/{sn}/{model}_{point_name}/config");
                }
                InputType::Number(num) => {
                    config_payload.min = Some(num.min);
                    config_payload.max = Some(num.max);
                    config_payload.step = num.step;
                    config_payload.mode = num.mode.clone();
                    config_payload.entity_category = Some(EntityCategory::Config);
                    config_payload.entity_id = format!("number.{model}_{point_name}");
                    config_topic = format!("homeassistant/number/{sn}/{model}_{point_name}/config");
                }
            },
            None => {
                error!("{log_prefix}: readwrite set in config, but no inputs specified.");
            }
        };
    }

    let state_topic = format!("sunspec_gateway/{sn}/{model}/{point_name}");

    config_payload.state_topic = state_topic.clone();

    let resp = CompoundPayload {
        config: config_payload,
        state: state_payload,
        config_topic,
        state_topic,
    };
    vec![resp]
}
