use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, OnceCell};

use crate::user::User;

static USER_MANAGER: OnceCell<Arc<Mutex<UserManager>>> = OnceCell::const_new();
pub struct UserManager {
    users: HashMap<String, User>,
}

impl UserManager {
    fn new() {
        Self {
            users: HashMap::new(),
        };
    }
    fn add_user(&mut self, user: User) {
        todo!()
    }
}
