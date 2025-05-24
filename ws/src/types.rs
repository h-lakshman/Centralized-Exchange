use serde::{Deserialize, Serialize};

use engine::types::{WsMessage as EngineWsMessage, WsPayload as EngineWsPayload};

#[derive(Debug, Serialize, Deserialize)]
pub enum Method {
    #[serde(rename = "SUBSCRIBE")]
    Subscribe,
    #[serde(rename = "UNSUBSCRIBE")]
    Unsubscribe,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub method: Method,
    pub params: Vec<String>,
}

pub type OutgoingMessage = EngineWsMessage;

pub type WsPayload = EngineWsPayload;
