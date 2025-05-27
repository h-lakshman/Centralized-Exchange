use chrono::{DateTime, Utc};
use dotenv::dotenv;
use engine::types::{DbMessage, DbMessageData, DbMessageType};
use redis::{AsyncCommands, Client};
use serde_json;
use sqlx::{error::BoxDynError, Connection, PgConnection};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (mut pg_conn, mut redis_conn) = match init().await {
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
                            if let DbMessageData::TradeAdd(trade) = db_message.data {
                                println!(
                                    "Trade added: id={}, price={}, quantity={}",
                                    trade.id, trade.price, trade.quantity
                                );

                                let timestamp = trade
                                    .timestamp
                                    .parse::<DateTime<Utc>>()
                                    .unwrap_or_else(|_| Utc::now());

                                let query = "INSERT INTO trades (id, timestamp, market, price, quantity, quote_quantity, is_buyer_maker) VALUES ($1, $2, $3, $4, $5, $6, $7)";
                                if let Err(e) = sqlx::query(query)
                                    .bind(&trade.id)
                                    .bind(timestamp)
                                    .bind(&trade.market)
                                    .bind(&trade.price)
                                    .bind(&trade.quantity)
                                    .bind(&trade.quote_quantity)
                                    .bind(trade.is_buyer_maker)
                                    .execute(&mut pg_conn)
                                    .await
                                {
                                    eprintln!("Failed to insert trade into database: {}", e);
                                }
                            }
                        }
                        DbMessageType::OrderUpdate => {
                            if let DbMessageData::OrderUpdate(order_update) = db_message.data {
                                println!(
                                    "Order updated: id={}, executed_qty={}",
                                    order_update.order_id, order_update.executed_quantity
                                );

                                let timestamp = Utc::now();

                                let side = order_update.side.as_ref().map(|s| s.as_str());

                                let query = r#"
                                    INSERT INTO orders (order_id, executed_quantity, price, market, quantity, side, updated_at)
                                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                                    ON CONFLICT (order_id) 
                                    DO UPDATE SET 
                                        executed_quantity = EXCLUDED.executed_quantity,
                                        price = COALESCE(EXCLUDED.price, orders.price),
                                        market = COALESCE(EXCLUDED.market, orders.market),
                                        quantity = COALESCE(EXCLUDED.quantity, orders.quantity),
                                        side = COALESCE(EXCLUDED.side, orders.side),
                                        updated_at = EXCLUDED.updated_at
                                "#;

                                if let Err(e) = sqlx::query(query)
                                    .bind(&order_update.order_id)
                                    .bind(order_update.executed_quantity as i64)
                                    .bind(order_update.price)
                                    .bind(order_update.market.as_deref())
                                    .bind(order_update.quantity)
                                    .bind(side)
                                    .bind(timestamp)
                                    .execute(&mut pg_conn)
                                    .await
                                {
                                    eprintln!("Failed to insert/update order in database: {}", e);
                                }
                            }
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
