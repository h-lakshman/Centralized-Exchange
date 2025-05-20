use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

mod user;
mod user_manager;
#[tokio::main]
async fn main() {
    let listener= TcpListener::bind("127.0.0.1:9001").await.unwrap();
    println!("Ws Server Listening");
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: TcpStream) {
    match accept_async(stream).await {
        Ok(ws_stream) => todo!(),
        Err(e) => {
            println!("Error accepting connection: {}", e);
        }
    }
}
