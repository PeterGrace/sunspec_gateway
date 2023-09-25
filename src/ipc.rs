use crate::payload::Payload;

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
