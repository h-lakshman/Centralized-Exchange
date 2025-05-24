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
