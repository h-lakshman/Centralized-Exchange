use crate::types::{DbMessage, MessageToApi};
use engine::types::WsMessage;
use redis::{Client, Commands, Connection};
use std::{env, error::Error, sync::OnceLock};

pub static REDIS_MANAGER: OnceLock<RedisManager> = OnceLock::new();

pub struct RedisManager {
    pub client: Client,
    pub queue_connection: Connection,
}

impl RedisManager {
    fn new() -> Result<Self, Box<dyn Error>> {
        let redis_url = env::var("REDIS_URL")?;
        let client = Client::open(redis_url.clone())?;
        let queue_connection = client.get_connection()?;
        Ok(RedisManager {
            client,
            queue_connection,
        })
    }
    pub fn get_instance() -> &'static RedisManager {
        REDIS_MANAGER.get_or_init(|| {
            RedisManager::new()
                .expect("Failed to create RedisManager instance or REDIS_URL is not set")
        })
    }

    pub fn push_message(&mut self, message: DbMessage) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let _: () = self.queue_connection.lpush("db_processor", payload)?;
        Ok(())
    }

    pub fn send_to_api(
        &self,
        channel: String,
        message: MessageToApi,
    ) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.client.get_connection()?;
        let _: () = connection.publish(channel, payload)?;
        Ok(())
    }

    pub fn publish_message(
        &mut self,
        channel: String,
        message: WsMessage,
    ) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.client.get_connection()?;
        let _: () = connection.publish(channel, payload)?;
        Ok(())
    }
}
