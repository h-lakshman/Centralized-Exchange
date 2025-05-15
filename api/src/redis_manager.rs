use futures_util::StreamExt;
use redis::{AsyncCommands, Client};
use std::env;
use std::error::Error;
use std::sync::OnceLock;

use crate::types::{MessageFromOrderbook, MessageToEngine};

pub struct RedisManager {
    client: Client,
    publisher: Client,
}

pub static REDIS_MANAGER: OnceLock<RedisManager> = OnceLock::new();

impl RedisManager {
    fn new() -> Result<Self, Box<dyn Error>> {
        let redis_url = env::var("REDIS_URL")?;
        let client = Client::open(redis_url.clone())?;
        let publisher = Client::open(redis_url.clone())?;

        Ok(RedisManager { client, publisher })
    }

    pub fn get_instance() -> &'static RedisManager {
        REDIS_MANAGER.get_or_init(|| {
            RedisManager::new()
                .expect("Failed to create RedisManager instance or REDIS_URL is not set")
        })
    }

    pub async fn send_and_await(
        &self,
        message: MessageToEngine,
    ) -> Result<MessageFromOrderbook, Box<dyn Error>> {
        // async redis connectors
        let client_conn = self.client.get_async_connection().await?;
        let mut publisher_conn = self.publisher.get_async_connection().await?;

        let client_id = self.get_random_client_id();

        //pubsub setup
        let mut pubsub = client_conn.into_pubsub();
        pubsub.subscribe(&client_id).await?;
        let mut pubsub_stream = pubsub.on_message();

        let payload = serde_json::to_string(&(client_id.clone(), message))?;

        //push to queue
        let _: () = publisher_conn.lpush("messages", payload).await?;

        // await the stream to get message and response
        if let Some(msg) = pubsub_stream.next().await {
            let response: String = msg.get_payload()?;
            let result: MessageFromOrderbook = serde_json::from_str(&response)?;
            drop(pubsub_stream); //droppin stream cos on_message takes mut ref
            pubsub.unsubscribe(&client_id).await?;
            Ok(result)
        } else {
            Err("No message received".into())
        }
    }

    pub fn get_random_client_id(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}{:x}", rng.gen::<u64>(), rng.gen::<u64>())
    }
}
