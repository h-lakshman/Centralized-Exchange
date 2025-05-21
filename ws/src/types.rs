use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutgoingMessage {
    pub method: String,
    pub data: serde_json::Value,
}
