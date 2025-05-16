use std::{env, error::Error, sync::OnceLock};

use crate::types::DbMessageType;
use redis::Client;

struct DbMessage {
    db_message_type: DbMessageType,
    data: DbMessageData,
}

enum DbMessageData {
    TradeAdd(TradeAdd),
    OrderUpdate(OrderUpdate),
}

struct TradeAdd {
    id: String,
    is_buyer_maker: bool,
    price: String,
    quantity: String,
    quote_quantity: String,
    timestamp: String,
}

struct OrderUpdate {
    order_id: String,
    executed_quantity: u64,
    price: Option<String>,
    market: Option<String>,
    quantity: Option<String>,
    side: Option<String>,
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
}
