use std::sync::Arc;

use crate::subscription_manager::SubscriptionManager;
use crate::types::{IncomingMessage, Method, OutgoingMessage};
use crate::user_manager::UserManager;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub struct User {
    id: String,
    sender: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    stream: Option<SplitStream<WebSocketStream<TcpStream>>>,
    subscriptions: Arc<Mutex<Vec<String>>>,
}

impl User {
    pub fn new(
        id: String,
        sender: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
        stream: SplitStream<WebSocketStream<TcpStream>>,
    ) -> Self {
        let mut user = Self {
            id,
            sender,
            stream: Some(stream),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        };
        user.add_listeners();
        user
    }

    pub async fn subscribe(&self, subscription: String) {
        let mut subs = self.subscriptions.lock().await;
        subs.push(subscription);
    }

    pub async fn unsubscribe(&self, subscription: &str) {
        let mut subs = self.subscriptions.lock().await;
        subs.retain(|s| s != subscription);
    }

    pub async fn emit(&self, message: OutgoingMessage) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(&message)?;
        println!("Emitting message: {}", json);
        let mut ws = self.sender.lock().await;
        ws.send(Message::Text(json)).await?;
        println!("Message sent to user {}", self.id);
        Ok(())
    }

    fn add_listeners(&mut self) {
        let id = self.id.clone();
        let mut stream = self.stream.take().unwrap();

        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Ok(parsed_message) = serde_json::from_str::<IncomingMessage>(&text) {
                            let sub_manager = SubscriptionManager::get_instance().await;
                            let mut sub_guard = sub_manager.lock().await;
                            match parsed_message.method {
                                Method::Subscribe => {
                                    for s in parsed_message.params {
                                        let res =
                                            sub_guard.subscribe(id.clone(), s.to_string()).await;
                                        if let Err(e) = res {
                                            eprintln!("Error subscribing to {}: {}", s, e);
                                        }
                                    }
                                }
                                Method::Unsubscribe => {
                                    for s in parsed_message.params.clone() {
                                        let res = sub_guard
                                            .unsubscribe(
                                                id.clone(),
                                                parsed_message.params[0].clone(),
                                            )
                                            .await;
                                        if let Err(e) = res {
                                            eprintln!("Error unsubscribing from {}: {}", s, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(_) => {} // Ignore other message types
                    Err(e) => {
                        eprintln!("Error processing message for user {}: {}", id, e);
                        break;
                    }
                }
            }

            // Connection closed
            println!("Connection closed for user {}. Cleaning up.", id);
            let user_manager = UserManager::get_instance().await;
            let mut manager_guard = user_manager.lock().await;
            manager_guard.users.remove(&id);
            println!("User {} removed from UserManager.", id);
            let sub_manager = SubscriptionManager::get_instance().await;
            let mut sub_guard = sub_manager.lock().await;
            let res = sub_guard.user_left(id.clone()).await;
            if let Err(e) = res {
                eprintln!("Error removing user from subscription manager: {}", e);
            }
        });
    }
}
