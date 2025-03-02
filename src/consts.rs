pub const APP_NAME: &str = "sunspec_gateway";
pub const MPSC_BUFFER_SIZE: usize = 1024_usize;
pub const MINIMUM_QUERY_INTERVAL_SECS: u16 = 5_u16;
pub const CULL_HISTORY_ROWS: u8 = 200_u8;

pub const CHECK_DEVIATIONS_COUNT: u16 = 10_u16;

// we won't let points get checked faster than every 10 seconds.
// if we change this, the modbus could get saturated very quickly
pub const LOWER_LIMIT_INTERVAL: u64 = 10_u64;
pub const COMMON_MODEL_ID: u16 = 1_u16;
pub const DEFAULT_DISPLAY_PRECISION: Option<u8> = Some(4_u8);

pub const MQTT_KEEPALIVE_TIME: u64 = 5_u64;

pub const MQTT_THREAD_CHANNEL_CAPACITY: usize = 10_usize;
pub const MQTT_INBOUND_CONTROL_TOPIC: &str = "sunspec_gateway/input/#";

// poll intervals
pub const MQTT_POLL_INTERVAL_MILLIS: u64 = 100_u64;
pub const GENERIC_WAIT_MILLIS: u64 = 250_u64;
pub const MQTT_PROCESSING_PAD_MILLIS: u64 = 2000_u64;
