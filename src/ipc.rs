use crate::sunspec_unit::SunSpecUnit;

use crate::payload::Payload;
use serde::{Deserialize, Serialize};

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
