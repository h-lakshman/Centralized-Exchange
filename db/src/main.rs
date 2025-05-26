use dotenv::dotenv;
use engine::types::{DbMessage, DbMessageType};
use redis::{AsyncCommands, Client};
use serde_json;
use sqlx::{Connection, PgConnection, error::BoxDynError};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (pg_conn, mut redis_conn) = match init().await {
        Ok((pg_conn, redis_conn)) => (pg_conn, redis_conn),
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    loop {
        match redis_conn
            .brpop::<_, (String, String)>("db_processor", 0)
            .await
        {
            Ok((_, msg)) => {
                if let Ok(db_message) = serde_json::from_str::<DbMessage>(&msg) {
                    match db_message.db_message_type {
                        DbMessageType::TradeAdded => {
                            println!("Trade added: {:?}", db_message.data);
                            todo!("add trade to db");
                        }
                        DbMessageType::OrderUpdate => {
                            println!("Order updated: {:?}", db_message.data);
                            todo!("update order in db");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get message {}", e);
                continue;
            }
        }
    }
}

async fn init() -> Result<(PgConnection, redis::aio::Connection), BoxDynError> {
    let pg_conn = PgConnection::connect(&std::env::var("DATABASE_URL")?).await?;
    println!("Connected to PG database");

    let redis = Client::open(std::env::var("REDIS_URL").unwrap())?;
    let redis_conn = redis.get_async_connection().await?;
    println!("Connected to redis");
    Ok((pg_conn, redis_conn))
}
