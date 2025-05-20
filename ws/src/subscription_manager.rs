use crate::user::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};

static SUBSCRIPTION_MANAGER: OnceCell<Arc<Mutex<SubscriptionManager>>> = OnceCell::const_new();

pub struct SubscriptionManager {
    subscriptions: HashMap<String, Vec<String>>, // topic -> user_ids
}

impl SubscriptionManager {
    fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    pub async fn get_instance() -> Arc<Mutex<SubscriptionManager>> {
        SUBSCRIPTION_MANAGER
            .get_or_init(|| async { Arc::new(Mutex::new(SubscriptionManager::new())) })
            .await
            .clone()
    }

    pub async fn subscribe(&mut self, user_id: &str, topic: String) {
        self.subscriptions
            .entry(topic)
            .or_insert_with(Vec::new)
            .push(user_id.to_string());
    }

    pub async fn unsubscribe(&mut self, user_id: &str, topic: &str) {
        if let Some(subscribers) = self.subscriptions.get_mut(topic) {
            subscribers.retain(|id| id != user_id);
            if subscribers.is_empty() {
                self.subscriptions.remove(topic);
            }
        }
    }

    pub async fn get_subscribers(&self, topic: &str) -> Vec<String> {
        self.subscriptions.get(topic).cloned().unwrap_or_default()
    }
}
