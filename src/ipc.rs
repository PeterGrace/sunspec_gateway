use crate::sunspec_unit::SunSpecUnit;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

pub struct PublishMessage {
    pub(crate) topic: String,
    pub(crate) payload: Payload,
}

pub struct IPCError {
    pub serial_number: String,
    pub msg: String,
}

pub enum IPCMessage {
    Outbound(PublishMessage),
    PleaseReconnect(String, u8),
    Error(IPCError),
}
