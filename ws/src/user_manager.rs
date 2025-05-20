use futures_util::{stream::SplitStream, StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, OnceCell};
use tokio_tungstenite::WebSocketStream;

use crate::user::User;

static USER_MANAGER: OnceCell<Arc<Mutex<UserManager>>> = OnceCell::const_new();
pub struct UserManager {
    users: HashMap<String, User>,
}

impl UserManager {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
    pub async fn get_instance() -> Arc<Mutex<UserManager>> {
        USER_MANAGER
            .get_or_init(|| async { Arc::new(Mutex::new(UserManager::new())) })
            .await
            .clone()
    }
    pub async fn add_user(&mut self, ws: WebSocketStream<TcpStream>) -> User {
        let id = self.get_random_client_id();
        let (sink, stream) = ws.split();

        let user_sender = Arc::new(Mutex::new(sink));
        let user = User::new(id.clone(), user_sender);

        self.users.insert(id.clone(), user.clone());

        Self::register_on_close(id.clone(), stream);

        user
    }

    fn register_on_close(
        user_id: String,
        mut websocket_consumer_stream: SplitStream<WebSocketStream<TcpStream>>,
    ) {
        tokio::spawn(async move {
            while let Some(message_result) = websocket_consumer_stream.next().await {
                if let Err(e) = message_result {
                    eprintln!(
                        "Error on WebSocket stream for user {}: {}. Assuming disconnection.",
                        user_id, e
                    );
                    break;
                }
            }

            println!("Connection closed for user {}. Cleaning up.", user_id);

            let user_manager = UserManager::get_instance().await;
            let mut manager_guard = user_manager.lock().await;
            if manager_guard.users.remove(&user_id).is_some() {
                println!("User {} removed from UserManager.", user_id);
                // TODO: Notify SubscriptionManager about user leaving
            } else {
                println!(
                    "User {} not found for removal or already removed from UserManager.",
                    user_id
                );
            }

            println!(
                "Placeholder: SubscriptionManager notified about user {} leaving.",
                user_id
            );
        });
    }

    pub fn get_random_client_id(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}{:x}", rng.gen::<u64>(), rng.gen::<u64>())
    }
}
