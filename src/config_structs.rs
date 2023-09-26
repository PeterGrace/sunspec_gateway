use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct TracingConfig {
    pub url: String,
    pub sample_rate: f32,
}
#[derive(Deserialize, Clone, Debug, Default)]
pub struct UnitConfig {
    pub addr: String,
    pub slaves: Vec<u8>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Switchable {
    pub on: String,
    pub off: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Select(Vec<String>),
    Switch(Switchable),
    Button(String),
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct PointConfig {
    pub point: String,
    pub interval: u64,
    pub device_class: Option<String>,
    pub display_name: Option<String>,
    pub state_class: Option<String>,
    pub uom: Option<String>,
    pub precision: Option<u8>,
    pub readwrite: Option<bool>,
    pub homeassistant: Option<bool>,
    pub scale_factor: Option<i32>,
    pub inputs: Option<InputType>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct GatewayConfig {
    pub units: Vec<UnitConfig>,
    pub models: HashMap<String, Vec<PointConfig>>,
    pub mqtt_server_addr: String,
    pub mqtt_server_port: Option<u16>,
    pub mqtt_client_id: Option<String>,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub tracing: Option<TracingConfig>,
}
