use futures_util::StreamExt;
use redis::{aio::PubSub, Client, RedisError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

use crate::types::OutgoingMessage;
use crate::user_manager::UserManager;

static SUBSCRIPTION_MANAGER: OnceCell<Arc<Mutex<SubscriptionManager>>> = OnceCell::const_new();

pub struct SubscriptionManager {
    subscriptions: HashMap<String, Vec<String>>,
    reverse_subscriptions: HashMap<String, Vec<String>>,
    pubsub: PubSub,
}

impl SubscriptionManager {
    async fn init() -> Self {
        let client = Client::open("redis://127.0.0.1:6379").expect("Failed to connect to Redis");
        let conn = client
            .get_async_connection()
            .await
            .expect("Failed to get Redis connection");
        let pubsub = conn.into_pubsub();

        Self {
            subscriptions: HashMap::new(),
            reverse_subscriptions: HashMap::new(),
            pubsub,
        }
    }

    async fn run_redis_listener(instance_arc: Arc<Mutex<Self>>) {
        loop {
            let msg_result = {
                let mut guard = instance_arc.lock().await;
                let mut message_stream = guard.pubsub.on_message();
                message_stream.next().await
            };

            match msg_result {
                Some(redis_msg) => {
                    let channel_str: &str = redis_msg.get_channel_name();
                    let channel: String = channel_str.to_string();

                    let payload_result: redis::RedisResult<String> = redis_msg.get_payload();
                    let payload: String = match payload_result {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Error getting payload from Redis message on channel '{}': {}. Skipping.", channel, e);
                            continue;
                        }
                    };

                    let mut guard = instance_arc.lock().await;
                    guard.redis_callback_handler(channel, payload).await;
                }
                None => {
                    eprintln!("Redis PubSub stream ended. Attempting to re-establish stream in the next iteration.");
                }
            }
        }
    }

    pub async fn get_instance() -> Arc<Mutex<SubscriptionManager>> {
        SUBSCRIPTION_MANAGER
            .get_or_init(|| async {
                let manager = SubscriptionManager::init().await;
                let manager_arc = Arc::new(Mutex::new(manager));
                tokio::spawn(Self::run_redis_listener(manager_arc.clone()));

                manager_arc
            })
            .await
            .clone()
    }

    pub async fn subscribe(
        &mut self,
        user_id: String,
        subcsription: String,
    ) -> Result<(), RedisError> {
        let user_already_subscribed = self
            .subscriptions
            .get(&user_id)
            .map(|subscriptions| subscriptions.contains(&subcsription))
            .unwrap_or(false);

        if user_already_subscribed {
            return Ok(());
        }

        self.subscriptions
            .entry(user_id.to_string())
            .or_default()
            .push(subcsription.to_string());

        self.reverse_subscriptions
            .entry(subcsription.to_string())
            .or_default()
            .push(user_id.to_string());

        if self.reverse_subscriptions.get(&subcsription).unwrap().len() == 1 {
            self.pubsub.subscribe(subcsription).await?;
        }
        Ok(())
    }
    pub async fn unsubscribe(
        &mut self,
        user_id: String,
        subcsription: String,
    ) -> Result<(), RedisError> {
        if let Some(subscriptions) = self.subscriptions.get_mut(&user_id) {
            subscriptions.retain(|s| s != &subcsription);
        }

        if let Some(subscribers) = self.reverse_subscriptions.get_mut(&subcsription) {
            subscribers.retain(|s| s != &user_id);
            if subscribers.is_empty() {
                self.reverse_subscriptions.remove(&subcsription);
                self.pubsub.unsubscribe(subcsription).await?;
            }
        }
        Ok(())
    }

    pub fn get_subscribers(&mut self, user_id: &str) -> Vec<String> {
        self.subscriptions
            .entry(user_id.to_string())
            .or_default()
            .clone()
    }

    pub async fn user_left(&mut self, user_id: String) -> Result<(), RedisError> {
        println!("User {} left", user_id);
        let subscriptions = self.get_subscribers(&user_id);
        for subscription in subscriptions {
            self.unsubscribe(user_id.clone(), subscription).await?;
        }

        Ok(())
    }
    async fn redis_callback_handler(&mut self, channel: String, msg: String) {
        match serde_json::from_str::<OutgoingMessage>(&msg) {
            Ok(parsed_msg) => {
                if let Some(subscribers) = self.reverse_subscriptions.get(&channel) {
                    for subscriber_id in subscribers {
                        let user_manager = UserManager::get_instance().await;
                        let manager_guard = user_manager.lock().await;
                        if let Some(user_arc) = manager_guard.get_user(subscriber_id).await {
                            let user_guard = user_arc.lock().await;
                            if let Err(e) = user_guard.emit(parsed_msg.clone()).await {
                                eprintln!(
                                    "Error emitting message to user {}: {}",
                                    subscriber_id, e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Error parsing OutgoingMessage from Redis on channel {}: {}. Message: {}",
                    channel, e, msg
                );
            }
        }
    }
}
