use serde::{Serialize,Deserialize};
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub device: DeviceInfo,
    pub unique_id: String,
    pub native_value: ValueType,
    pub device_class: Option<String>,
    pub state_class: Option<String>,
    #[serde(rename="native_unit_of_measurement")]
    pub native_uom: Option<String>,
    pub options: Option<Vec<String>>,
    pub suggested_display_precision: u8
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
pub enum ValueType {
    float(f64),
    int(i64),
    string(String),
    #[default]
    None
}
pub struct PublishMessage {
    pub(crate) topic: String,
    pub(crate) payload: Payload
}
pub struct IPCError {
    pub serial_number: String,
    pub msg: String
}
pub enum IPCMessage {
    Outbound(PublishMessage),
    Error(IPCError)
}