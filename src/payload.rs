use crate::monitored_point::MonitoredPoint;
use crate::sunspec_unit::SunSpecUnit;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sunspec_rs::sunspec_models::{Point, ValueType};

const DEFAULT_DISPLAY_PRECISION: Option<u8> = Some(4_u8);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeviceInfo {
    pub identifiers: Vec<String>,
    pub manufacturer: String,
    pub name: String,
    pub sw_version: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(untagged)]
pub enum PayloadValueType {
    Float(f64),
    Int(i64),
    String(String),
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
pub struct HAConfigPayload {
    pub name: String,
    pub device: DeviceInfo,
    pub unique_id: String,
    pub entity_id: String,
    pub state_topic: String,
    pub expires_after: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<String>,
    #[serde(
        rename = "unit_of_measurement",
        skip_serializing_if = "Option::is_none"
    )]
    pub native_uom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_display_precision: Option<u8>,
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
    last_seen: DateTime<Utc>,
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

#[instrument]
pub fn generate_payloads(
    unit: &SunSpecUnit,
    point_data: &Point,
    monitored_point: &MonitoredPoint,
    val: &ValueType,
) -> Vec<CompoundPayload> {
    let sn = unit.serial_number.clone();
    let model = monitored_point.model.clone();
    let point_name = monitored_point.name.clone();
    let mut config_payload: HAConfigPayload = HAConfigPayload::default();
    let mut state_payload: StatePayload = StatePayload::default();
    config_payload.name = format!("{sn}: {model}-{point_name}");
    config_payload.device_class = monitored_point.device_class.clone();
    config_payload.state_class = monitored_point.state_class.clone();
    config_payload.expires_after = 300;
    config_payload.value_template = Some("{{ value_json.value }}".to_string());
    config_payload.unique_id = format!("{sn}.{model}.{point_name}");
    config_payload.entity_id = format!("sensor.{sn}_{model}_{point_name}");
    match val {
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
                config_payload.native_uom = point_data.units.clone();
            }
            state_payload.value = PayloadValueType::Int(*int as i64)
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
                config_payload.native_uom = point_data.units.clone();
            }
            state_payload.value = PayloadValueType::Float(*float as f64);
            if let Some(literal) = &point_data.literal {
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
            debug!("Response for {model}/{point_name}: {:#?}", vec);
            let concat = vec.join(", ");
            state_payload.value = PayloadValueType::String(concat)
        }
    }

    let state_topic = format!("sunspec_gateway/{sn}/{model}/{point_name}");
    let config_topic = format!("homeassistant/sensor/{sn}/{model}_{point_name}/config");

    config_payload.device = unit.device_info.clone();
    config_payload.state_topic = state_topic.clone();

    let resp = CompoundPayload {
        config: config_payload,
        state: state_payload,
        config_topic,
        state_topic,
    };
    vec![resp]
}
