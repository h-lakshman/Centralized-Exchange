use redis::Msg;
use redis::{aio::PubSub, Client, RedisError};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

use crate::types::OutgoingMessage;
use crate::user_manager::UserManager;

static SUBSCRIPTION_MANAGER: OnceCell<Arc<Mutex<SubscriptionManager>>> = OnceCell::const_new();

pub struct SubscriptionManager {
    subscriptions: HashMap<String, Vec<String>>,
    reverse_subscriptions: HashMap<String, Vec<String>>,
    redis_client: Client,
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
            redis_client: client,
            pubsub,
        }
    }

    pub async fn get_instance() -> Arc<Mutex<SubscriptionManager>> {
        SUBSCRIPTION_MANAGER
            .get_or_init(|| async { Arc::new(Mutex::new(SubscriptionManager::init().await)) })
            .await
            .clone()
    }

    pub async fn subscribe(
        &mut self,
        user_id: &str,
        subcsription: String,
    ) -> Result<(), RedisError> {
        let user_already_subscribed = self
            .subscriptions
            .get(user_id)
            .map(|subscriptions| subscriptions.contains(&subcsription))
            .unwrap_or(false);

        if user_already_subscribed {
            return Ok(());
        }

        self.subscriptions
            .entry(user_id.to_string())
            .or_default()
            .push(subcsription.clone());

        self.reverse_subscriptions
            .entry(subcsription.clone())
            .or_default()
            .push(user_id.to_string());

        if self.reverse_subscriptions.get(&subcsription).unwrap().len() == 1 {
            self.pubsub.subscribe(subcsription).await?;
        }
        Ok(())
    }
    pub async fn unsubscribe(
        &mut self,
        user_id: &str,
        subcsription: &str,
    ) -> Result<(), RedisError> {
        if let Some(subscriptions) = self.subscriptions.get_mut(user_id) {
            subscriptions.retain(|s| s != subcsription);
        }

        if let Some(subscribers) = self.reverse_subscriptions.get_mut(subcsription) {
            subscribers.retain(|s| s != user_id);
            if subscribers.is_empty() {
                self.reverse_subscriptions.remove(subcsription);
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

    pub async fn user_left(&mut self, user_id: &str) -> Result<(), RedisError> {
        println!("User {} left", user_id);
        let subscriptions = self.get_subscribers(user_id);
        for subscription in subscriptions {
            self.unsubscribe(user_id, &subscription).await?;
        }

        Ok(())
    }
    async fn redis_callback_handler(&mut self, channel: String, msg: String) {
        let parsed_msg = serde_json::from_str::<OutgoingMessage>(&msg).unwrap();
        if let Some(subscribers) = self.reverse_subscriptions.get(&channel) {
            for subscriber in subscribers {
                let user_manager = UserManager::get_instance().await;
                let manager_guard = user_manager.lock().await;
                let user = manager_guard.get_user(subscriber).await;
                if let Some(user) = user {
                    user.lock().await.emit(parsed_msg.clone()).await.unwrap();
                }
            }
        }
    }
}
