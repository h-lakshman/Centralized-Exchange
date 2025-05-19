use crate::types::MessageFromApi;
use redis::Commands;
use redis_manager::RedisManager;
use trades::engine::{Engine, ProcessParams};

mod redis_manager;
mod trades;
mod types;

fn main() {
    dotenv::dotenv().ok();
    let mut engine = Engine::new();
    let redis = RedisManager::get_instance();

    println!("Engine started, waiting for messages...");

    loop {
        match redis.client.get_connection() {
            Ok(mut conn) => {
                if let Ok(Some((_, response))) =
                    conn.brpop::<_, Option<(String, String)>>("messages", 0)
                {
                    if let Ok((client_id, message)) =
                        serde_json::from_str::<(String, MessageFromApi)>(&response)
                    {
                        engine.process(ProcessParams { message, client_id });
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get Redis connection: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
        }
    }
}
