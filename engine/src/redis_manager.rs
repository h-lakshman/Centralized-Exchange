use engine::types::{DbMessage, MessageFromApi, MessageToApi, WsMessage};
use redis::Client;
use redis::{aio::Connection, AsyncCommands};
use std::{env, error::Error};
use tokio::sync::{Mutex, OnceCell};

pub static REDIS_MANAGER: OnceCell<RedisManager> = OnceCell::const_new();

pub struct RedisManager {
    pub reciever: Mutex<Connection>,
    pub writer: Mutex<Connection>,
    pub publisher: Mutex<Connection>,
}

impl RedisManager {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let redis_url = env::var("REDIS_URL")?;
        let client = Client::open(redis_url.clone())?;
        Ok(RedisManager {
            reciever: Mutex::new(client.get_async_connection().await?),
            writer: Mutex::new(client.get_async_connection().await?),
            publisher: Mutex::new(client.get_async_connection().await?),
        })
    }
    pub async fn get_instance() -> &'static RedisManager {
        REDIS_MANAGER
            .get_or_init(|| async {
                RedisManager::new()
                    .await
                    .expect("Failed to create RedisManager instance or REDIS_URL is not set")
            })
            .await
    }

    pub async fn get_message(&self) -> Result<(String, MessageFromApi), Box<dyn Error>> {
        let mut connection = self.reciever.lock().await;
        let (_, payload): (String, String) = connection.brpop("messages".to_string(), 0).await?;
        let (client_id, message): (String, MessageFromApi) = serde_json::from_str(&payload)?;
        Ok((client_id, message))
    }

    pub async fn push_message(&self, message: DbMessage) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.writer.lock().await;
        let _: () = connection.lpush("db_processor", payload).await?;
        Ok(())
    }

    pub async fn send_to_api(
        &self,
        channel: String,
        message: MessageToApi,
    ) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.publisher.lock().await;
        let _: () = connection.publish(channel, payload).await?;
        Ok(())
    }

    pub async fn publish_message(
        &self,
        channel: String,
        message: WsMessage,
    ) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.publisher.lock().await;
        let _: () = connection.publish(channel, payload).await?;
        Ok(())
    }
}
