use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbMessageType {
    TradeAdded,
    OrderCreated,
}
pub const TRADE_ADDED: &str = "TRADE_ADDED";
pub const ORDER_CREATED: &str = "ORDER_CREATED";

//Send To Api Types
enum SendToApiType {
    TradeAdded,
    OrderCreated,
}
