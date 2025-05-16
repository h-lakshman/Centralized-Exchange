use crate::trades::Order;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbMessageType {
    TradeAdded,
    OrderCreated,
}

//Send To Api
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageToApi {
    Depth(DepthPayload),
    OrderPlaced(OrderPlacedPayload),
    OrderCancelled(OrderCancelledPayload),
    OpenOrders(Vec<Order>),
}

#[derive(Serialize, Deserialize)]
pub struct DepthPayload {
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

#[derive(Serialize, Deserialize)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub executed_qty: u64,
    pub fills: Vec<Fill>,
}

#[derive(Serialize, Deserialize)]
pub struct OrderCancelledPayload {
    pub order_id: String,
    pub executed_qty: u64,
    pub remaining_qty: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Fill {
    pub price: String,
    pub qty: u64,
    pub trade_id: u64,
}
