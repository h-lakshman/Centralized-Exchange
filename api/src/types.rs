use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageToEngine {
    pub message_type: MessageType,
    pub data: PlaceOrderRequest,
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
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    PlaceOrder,
    CancelOrder,
    ModifyOrder,
    GetOrder,
}