use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageToEngine {
    CreateOrder(PlaceOrderRequest),
    CancelOrder(CancelOrderRequest),
    OnRamp(OnRampRequest),
    GetDepth(GetDepthRequest),
    GetOpenOrders(GetOpenOrdersRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
    pub market: String,
    pub price: String,
    pub quantity: String,
    pub side: Side,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnRampRequest {
    pub amount: String,
    pub user_id: String,
    pub txn_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDepthRequest {
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOpenOrdersRequest {
    pub user_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

// Response from orderbook

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageFromOrderbook {
    Depth(DepthPayload),
    OrderPlaced(OrderPlacedPayload),
    OrderCancelled(OrderCancelledPayload),
    OpenOrders(Vec<OpenOrder>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepthPayload {
    pub market: String,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub executed_qty: u32,
    pub fills: Vec<Fill>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderCancelledPayload {
    pub order_id: String,
    pub executed_qty: u32,
    pub remaining_qty: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fill {
    pub price: String,
    pub qty: u32,
    pub trade_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrder {
    pub order_id: String,
    pub price: String,
    pub executed_quantity: u32,
    pub quantity: u32,
    pub user_id: String,
}
