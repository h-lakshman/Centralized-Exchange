use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

pub struct User {
    id: String,
    ws: WebSocketStream<TcpStream>,
}

impl User {
    fn new(id: String, ws: WebSocketStream<TcpStream>) {
        Self { id, ws };
    }
}
