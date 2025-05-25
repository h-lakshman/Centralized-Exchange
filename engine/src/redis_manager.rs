use engine::types::{DbMessage, MessageFromApi, MessageToApi, WsMessage};
use redis::{Client, Commands, Connection};
use std::{
    env,
    error::Error,
    sync::{Mutex, OnceLock},
};

pub static REDIS_MANAGER: OnceLock<RedisManager> = OnceLock::new();

pub struct RedisManager {
    pub reciever: Mutex<Connection>,
    pub writer: Mutex<Connection>,
    pub publisher: Mutex<Connection>,
}

impl RedisManager {
    fn new() -> Result<Self, Box<dyn Error>> {
        let redis_url = env::var("REDIS_URL")?;
        let client = Client::open(redis_url.clone())?;
        Ok(RedisManager {
            reciever: Mutex::new(client.get_connection()?),
            writer: Mutex::new(client.get_connection()?),
            publisher: Mutex::new(client.get_connection()?),
        })
    }
    pub fn get_instance() -> &'static RedisManager {
        REDIS_MANAGER.get_or_init(|| {
            RedisManager::new()
                .expect("Failed to create RedisManager instance or REDIS_URL is not set")
        })
    }

    pub fn get_message(&self) -> Result<(String, MessageFromApi), Box<dyn Error>> {
        let mut connection = self.reciever.lock().unwrap();
        let (_, payload): (String, String) = connection.brpop("messages".to_string(), 0)?;
        let (client_id, message): (String, MessageFromApi) = serde_json::from_str(&payload)?;
        Ok((client_id, message))
    }

    pub fn push_message(&self, message: DbMessage) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.writer.lock().unwrap();
        let _: () = connection.lpush("db_processor", payload)?;
        Ok(())
    }

    pub fn send_to_api(
        &self,
        channel: String,
        message: MessageToApi,
    ) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.publisher.lock().unwrap();
        let _: () = connection.publish(channel, payload)?;
        Ok(())
    }

    pub fn publish_message(
        &self,
        channel: String,
        message: WsMessage,
    ) -> Result<(), Box<dyn Error>> {
        let payload = serde_json::to_string(&message)?;
        let mut connection = self.publisher.lock().unwrap();
        let _: () = connection.publish(channel, payload)?;
        Ok(())
    }
}
