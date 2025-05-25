use futures_util::StreamExt;
use redis::{
    aio::{Connection, PubSub},
    AsyncCommands, Client,
};
use std::env;
use std::error::Error;
use tokio::sync::{Mutex, OnceCell};

use crate::types::{MessageFromOrderbook, MessageToEngine};

pub struct RedisManager {
    client: Mutex<Connection>,
    pubsub: Mutex<PubSub>,
}

pub static REDIS_MANAGER: OnceCell<RedisManager> = OnceCell::const_new();

impl RedisManager {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let redis_url = env::var("REDIS_URL")?;
        let client = Client::open(redis_url.clone())?;
        let client_conn = client.get_async_connection().await?;
        let pubsub_client = Client::open(redis_url.clone())?;
        let pubsub_conn = pubsub_client.get_async_connection().await?;

        Ok(RedisManager {
            client: Mutex::new(client_conn),
            pubsub: Mutex::new(pubsub_conn.into_pubsub()),
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

    pub async fn send_and_await(
        &self,
        message: MessageToEngine,
    ) -> Result<MessageFromOrderbook, Box<dyn Error>> {
        // async redis connectors

        let client_id = self.get_random_client_id();

        //pubsub setup
        let mut pubsub = self.pubsub.lock().await;
        pubsub.subscribe(&client_id).await?;
        let mut pubsub_stream = pubsub.on_message();

        let payload = serde_json::to_string(&(client_id.clone(), message))?;

        //push to queue
        let mut client = self.client.lock().await;
        let _: () = client.lpush("messages", payload).await?;
        drop(client);

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
