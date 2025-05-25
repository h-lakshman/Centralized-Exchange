use redis_manager::RedisManager;
use trades::engine::{Engine, ProcessParams};
mod redis_manager;
mod trades;

fn main() {
    dotenv::dotenv().ok();
    let mut engine = Engine::new();
    let redis = RedisManager::get_instance();

    println!("Engine started");

    loop {
        println!("Waiting for messages...");
        match redis.get_message() {
            Ok((client_id, message)) => {
                match ProcessParams::from_api_message(message, client_id.clone()) {
                    Ok(params) => {
                        engine.process(params);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse message: {}", e);
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get message: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
        }
    }
}
