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
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum OutgoingMessage {
    Depth(DepthMessage),
    Ticker(TickerMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DepthMessage {
    pub e: String,
    pub b: Vec<[String; 2]>,
    pub a: Vec<[String; 2]>,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TickerMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub l: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub V: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<String>,

    pub id: u64,
    pub e: String,
}
