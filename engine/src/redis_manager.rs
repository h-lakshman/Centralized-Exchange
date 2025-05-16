use crate::types::DbMessageType;
use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};
use std::{env, error::Error, sync::OnceLock};

#[derive(Serialize, Deserialize)]
pub struct DbMessage {
    #[serde(rename = "type")]
    db_message_type: DbMessageType,
    data: DbMessageData,
}

#[derive(Serialize, Deserialize)]
enum DbMessageData {
    TradeAdd(TradeAdd),
    OrderUpdate(OrderUpdate),
}

#[derive(Serialize, Deserialize)]
struct TradeAdd {
    id: String,
    is_buyer_maker: bool,
    price: String,
    quantity: String,
    quote_quantity: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct OrderUpdate {
    order_id: String,
    executed_quantity: u64,
    price: Option<String>,
    market: Option<String>,
    quantity: Option<String>,
    side: Option<Side>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Side {
    Buy,
    Sell,
}
pub static REDIS_MANAGER: OnceLock<RedisManager> = OnceLock::new();
pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    fn new() -> Result<Self, Box<dyn Error>> {
        let redis_url = env::var("REDIS_URL")?;
        let client = Client::open(redis_url.clone())?;
        Ok(RedisManager { client })
    }
    pub fn get_instance() -> &'static RedisManager {
        REDIS_MANAGER.get_or_init(|| {
            RedisManager::new()
                .expect("Failed to create RedisManager instance or REDIS_URL is not set")
        })
    }

    pub fn push_message(&self, message: DbMessage) -> Result<(), Box<dyn Error>> {
        let mut connection = self.client.get_connection()?;
        let payload = serde_json::to_string(&message)?;
        let _: () = connection.lpush("db_processor", payload)?;
        Ok(())
    }

    pub fn send_to_api(&self, channel: &str, message: DbMessage) -> Result<(), Box<dyn Error>> {
        let mut connection = self.client.get_connection()?;
        let payload = serde_json::to_string(&message)?;
        let _: () = connection.publish(channel, payload)?;
        Ok(())
    }

    //
}
