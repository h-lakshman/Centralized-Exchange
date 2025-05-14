use redis::{Client, Connection, PubSubCommands};
use serde::Serialize;
use std::error::Error;
use std::sync::{Mutex, OnceLock};

use crate::types::{MessageFromOrderbook, MessageToEngine};

pub struct RedisManager {
    client_conn: Connection,
    publisher_conn: Connection,
}

pub static REDIS_MANAGER: OnceLock<Mutex<RedisManager>> = OnceLock::new();

impl RedisManager {
    fn new() -> Result<Self, Box<dyn Error>> {
        let client = Client::open("redis://127.0.0.1:6379")?;
        let publisher = Client::open("redis://127.0.0.1:6379")?;

        let client_conn = client.get_connection()?;
        let publisher_conn = publisher.get_connection()?;

        Ok(RedisManager {
            client_conn,
            publisher_conn,
        })
    }

    pub fn get_instance() -> &'static Mutex<RedisManager> {
        REDIS_MANAGER.get_or_init(|| {
            Mutex::new(
                RedisManager::new()
                    .expect("Failed to create RedisManager instance or instance already exists"),
            )
        })
    }

    pub async fn send_and_await(
        &mut self,
        message: MessageToEngine,
    ) -> Result<MessageFromOrderbook, Box<dyn Error>> {
        let client_id = self.get_random_client_id();
        let payload = serde_json::to_string(&(client_id.clone(), message))?;
        let mut pubsub = self.client_conn.as_pubsub();
        pubsub
            .subscribe(&client_id)
            .expect("Failed to subscribe to client ID");
        redis::cmd("LPUSH")
            .arg("messages")
            .arg(&payload)
            .execute(&mut self.publisher_conn);

        let msg = pubsub.get_message()?;
        let response: String = msg.get_payload()?;
        let result: MessageFromOrderbook = serde_json::from_str(&response)?;

        Ok(result)
    }

    pub fn get_random_client_id(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}{:x}", rng.gen::<u64>(), rng.gen::<u64>())
    }
}
