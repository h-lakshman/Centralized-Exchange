use serde::{Deserialize, Serialize};

pub use engine::types::{
    CancelOrderPayload as CancelOrderRequest, CreateOrderPayload as PlaceOrderRequest,
    GetDepthPayload as GetDepthRequest, GetOpenOrdersPayload as GetOpenOrdersRequest,
    MessageToApi as MessageFromOrderbook, OnRampPayload as OnRampRequest,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageToEngine {
    CreateOrder(PlaceOrderRequest),
    CancelOrder(CancelOrderRequest),
    OnRamp(OnRampRequest),
    GetDepth(GetDepthRequest),
    GetOpenOrders(GetOpenOrdersRequest),
}

//Kline route types
#[derive(Deserialize)]
pub struct KlinesQuery {
    pub market: String,
    pub interval: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Serialize)]
pub struct Kline {
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
    pub trades: String,
    pub start: String,
    pub end: String,
}

//trade route types
#[derive(Deserialize)]
pub struct TradeQuery {
    pub symbol: String,
    pub limit: u64,
}
