use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, Default, ToSchema)]
pub struct SystemStatus {
    pub mqtt_connected: bool,
    pub mqtt_enabled: bool,
    pub mqtt_last_error: Option<String>,
    pub devices: Vec<DeviceStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DeviceStatus {
    pub name: String,
    pub connected: bool,
    pub last_seen: i64, // Unix timestamp
    pub last_error: Option<String>,
}
