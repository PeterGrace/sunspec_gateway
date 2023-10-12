use crate::payload::Payload;

#[derive(Clone)]
pub struct InboundMessage {
    pub serial_number: String,
    pub model: String,
    pub point_name: String,
    pub payload: String,
}

#[derive(Clone)]
pub struct PublishMessage {
    pub(crate) topic: String,
    pub(crate) payload: Payload,
}

#[derive(Clone)]
pub struct IPCError {
    pub serial_number: String,
    pub msg: String,
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum IPCMessage {
    Inbound(InboundMessage),
    Outbound(PublishMessage),
    PleaseReconnect(String, u8),
    Error(IPCError),
    Shutdown,
}
