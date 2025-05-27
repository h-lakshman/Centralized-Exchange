use crate::types::TradeQuery;
use actix_web::{web, HttpResponse, Responder};
use engine::types::TradeAdd as Trade;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

pub async fn get_trades(data: web::Query<TradeQuery>, pool: web::Data<PgPool>) -> impl Responder {
    let query_params = data.into_inner();

    let sql_query = "SELECT id, is_buyer_maker, price, quantity, quote_quantity, timestamp, market 
            FROM trades WHERE market = $1 ORDER BY id DESC LIMIT $2";

    match sqlx::query(sql_query)
        .bind(&query_params.symbol)
        .bind(query_params.limit as i64)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(rows) => {
            let trades: Vec<Trade> = rows
                .iter()
                .map(|row| {
                    let price: Decimal = row.get("price");
                    let quantity: Decimal = row.get("quantity");
                    let quote_quantity: Decimal = row.get("quote_quantity");

                    Trade {
                        id: row.get("id"),
                        market: row.get("market"),
                        price: price.to_string(),
                        quantity: quantity.to_string(),
                        quote_quantity: quote_quantity.to_string(),
                        timestamp: row
                            .get::<chrono::DateTime<chrono::Utc>, _>("timestamp")
                            .to_rfc3339(),
                        is_buyer_maker: row.get("is_buyer_maker"),
                    }
                })
                .collect();

            HttpResponse::Ok().json(trades)
        }
        Err(e) => {
            eprintln!("Database error fetching trades: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch trades"
            }))
        }
    }
}
