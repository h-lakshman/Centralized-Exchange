mod user;
mod user_manager;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use user_manager::UserManager;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
    println!("Ws Server Listening");
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: TcpStream) {
    match accept_async(stream).await {
        Ok(ws_stream) => {
            let user_manager = UserManager::get_instance().await;
            let mut manager_guard = user_manager.lock().await;
            let _user = manager_guard.add_user(ws_stream).await;
            println!("User {} connected.", _user.get_id());
        }
        Err(e) => {
            println!("Error accepting connection: {}", e);
        }
    }
}
