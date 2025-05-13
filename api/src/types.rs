use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageToEngine {
    pub message_type: MessageType,
    pub data: EngineMessageData
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EngineMessageData {
    PlaceOrder(PlaceOrderRequest),
    CancelOrder(CancelOrderRequest),
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
pub struct GetOpenOrdersRequest {
    pub user_id: String,
    pub market: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    PlaceOrder,
    CancelOrder,
    GetOpenOrders,
}

