use futures_util::StreamExt;
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, OnceCell};
use tokio_tungstenite::WebSocketStream;

use crate::user::User;

static USER_MANAGER: OnceCell<Arc<Mutex<UserManager>>> = OnceCell::const_new();
pub struct UserManager {
    users: HashMap<String, Arc<Mutex<User>>>,
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
    pub async fn add_user(&mut self, ws: WebSocketStream<TcpStream>) {
        let id = self.get_random_client_id();
        let (sink, stream) = ws.split();

        let user_sender = Arc::new(Mutex::new(sink));
        let user = User::new(id.clone(), user_sender.clone(), stream);

        self.users.insert(id, Arc::new(Mutex::new(user)));
    }

    pub async fn remove_user(&mut self, id: &str) {
        self.users.remove(id);
    }

    pub async fn get_user(&self, id: &str) -> Option<Arc<Mutex<User>>> {
        self.users.get(id).cloned()
    }

    pub fn get_random_client_id(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}{:x}", rng.gen::<u64>(), rng.gen::<u64>())
    }
}
