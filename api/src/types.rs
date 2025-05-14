use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageToEngine {
    pub message_type: MessageToType,
    pub data: EngineMessageData,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EngineMessageData {
    PlaceOrder(PlaceOrderRequest),
    CancelOrder(CancelOrderRequest),
    GetOpenOrders(GetOpenOrdersRequest),
    GetDepth(GetDepthRequest),
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
pub struct GetOpenOrdersRequest {
    pub user_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDepthRequest {
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageToType {
    PlaceOrder,
    CancelOrder,
    GetOpenOrders,
    GetDepth,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageFromType {
    Depth,
    OrderCanceled,
    OrderPlaced,
    OpenOrders,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageFromOrderbook {
    pub message_type: MessageFromType,
    pub data: OrderbookMessageData,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderbookMessageData {
    Depth(Depth),
    OrderCanceled(OrderCanceled),
    OrderPlaced(OrderPlaced),
    OpenOrders(OpenOrders),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Depth {
    pub market: String,
    asks: Vec<String>,
    bids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderCanceled {
    pub order_id: String,
    pub executed_quantity: u32,
    pub remaining_quantity: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderPlaced {
    pub order_id: String,
    pub executed_quantity: u32,
    pub fills: Vec<Fill>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fill {
    pub price: String,
    pub quantity: u32,
    pub trade_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOrders {
    pub market: String,
    pub orders: Vec<Order>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub price: String,
    pub executed_quantity: u32,
    pub quantity: u32,
    pub user_id: String,
}
