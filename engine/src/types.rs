use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbMessageType {
    TradeAdded,
    OrderUpdate,
}

//Message to DB
#[derive(Serialize, Deserialize)]
pub struct DbMessage {
    #[serde(rename = "type")]
    pub db_message_type: DbMessageType,
    pub data: DbMessageData,
}

#[derive(Serialize, Deserialize)]
pub enum DbMessageData {
    TradeAdd(TradeAdd),
    OrderUpdate(OrderUpdate),
}

#[derive(Serialize, Deserialize)]
pub struct TradeAdd {
    pub id: String,
    pub is_buyer_maker: bool,
    pub price: String,
    pub quantity: String,
    pub quote_quantity: String,
    pub timestamp: String,
    pub market: String,
}

#[derive(Serialize, Deserialize)]
pub struct OrderUpdate {
    pub order_id: String,
    pub executed_quantity: u64,
    pub price: Option<String>,
    pub market: Option<String>,
    pub quantity: Option<String>,
    pub side: Option<Side>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Order {
    pub price: u64,
    pub quantity: u64,
    pub order_id: String,
    pub filled: u64,
    pub side: Side,
    pub user_id: String,
}

//Send To Api
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageToApi {
    Depth(DepthPayload),
    OrderPlaced(OrderPlacedPayload),
    OrderCancelled(OrderCancelledPayload),
    OpenOrders(Vec<Order>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DepthPayload {
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderPlacedPayload {
    pub order_id: String,
    pub executed_qty: u64,
    pub fills: Vec<Fill>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderCancelledPayload {
    pub order_id: String,
    pub executed_qty: u64,
    pub remaining_qty: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fill {
    pub price: String,
    pub qty: u64,
    pub trade_id: u64,
}

//Recieve from Api
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageFromApi {
    CreateOrder(CreateOrderPayload),
    CancelOrder(CancelOrderPayload),
    GetDepth(GetDepthPayload),
    GetOpenOrders(GetOpenOrdersPayload),
    OnRamp(OnRampPayload),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOrderPayload {
    pub market: String,
    pub price: String,
    pub quantity: String,
    pub side: Side,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelOrderPayload {
    pub order_id: String,
    pub market: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDepthPayload {
    pub market: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOpenOrdersPayload {
    pub market: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OnRampPayload {
    pub amount: String,
    pub user_id: String,
    pub txn_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &str {
        match self {
            Side::Buy => "buy",
            Side::Sell => "sell",
        }
    }
}

//msg to ws
#[derive(Serialize, Deserialize, Clone)]
pub struct WsMessage {
    pub stream: String,
    pub data: WsPayload,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum WsPayload {
    Ticker(TickerUpdateMessage),
    Depth(DepthUpdateMessage),
    Trade(TradeUpdateMessage),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TickerUpdateMessage {
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

#[derive(Serialize, Deserialize, Clone)]
pub struct DepthUpdateMessage {
    pub b: Vec<[String; 2]>,
    pub a: Vec<[String; 2]>,
    pub e: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TradeUpdateMessage {
    pub e: String,
    pub t: u64,
    pub m: bool,
    pub p: String,
    pub q: String,
    pub s: String,
}
