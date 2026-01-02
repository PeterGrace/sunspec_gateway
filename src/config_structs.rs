use serde::Deserialize;
use std::collections::HashMap;
use sunspec_rs::sunspec_connection::TlsConfig;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct TracingConfig {
    pub url: String,
    pub sample_rate: f32,
}
#[derive(Deserialize, Clone, Debug, Default)]
pub struct UnitConfig {
    pub addr: String,
    pub slaves: Vec<u8>,
    pub tls: Option<TlsConfig>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Switchable {
    pub on: String,
    pub off: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Numerable {
    pub min: i32,
    pub max: i32,
    pub step: Option<i32>,
    pub mode: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Select(Vec<String>),
    Switch(Switchable),
    Button(String),
    Number(Numerable),
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct PointConfig {
    pub point: Option<String>,
    pub catalog_ref: Option<String>,
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
    pub input_only: Option<bool>,
    pub value_min: Option<f64>,
    pub value_max: Option<f64>,
    pub check_deviations: Option<u16>,
    pub topic_name: Option<String>,
}
impl PointConfig {
    pub fn name(&self) -> String {
        if self.catalog_ref.is_some() {
            return format!("{}", self.catalog_ref.clone().unwrap());
        }
        if self.point.is_some() {
            return format!("{}", self.point.clone().unwrap());
        }
        "".to_string()
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct GatewayConfig {
    pub hass_enabled: Option<bool>,
    pub units: Vec<UnitConfig>,
    pub models: HashMap<String, Vec<PointConfig>>,
    pub mqtt_server_addr: String,
    pub mqtt_server_port: Option<u16>,
    pub mqtt_client_id: Option<String>,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub tracing: Option<TracingConfig>,
}
