use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

#[derive(Clone)]
pub struct User {
    id: String,
    ws: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
}

impl User {
    pub fn new(id: String, ws: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>) -> Self {
        Self { id, ws }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }
}
