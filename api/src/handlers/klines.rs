use actix_web::{web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Deserialize)]
pub struct KlinesQuery {
    market: String,
    interval: String,
    start_time: Option<String>,
    end_time: Option<String>,
}

#[derive(Serialize)]
pub struct Kline {
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
    pub trades: String,
    pub start: String,
    pub end: String,
}

pub async fn get_klines(data: web::Query<KlinesQuery>, pool: web::Data<PgPool>) -> impl Responder {
    let query = data.into_inner();

    match get_klines_from_db(&query, &pool).await {
        Ok(klines) => HttpResponse::Ok().json(klines),
        Err(e) => {
            eprintln!("Failed to get klines: {}", e);
            HttpResponse::InternalServerError().json("Failed to get klines")
        }
    }
}

async fn get_klines_from_db(query: &KlinesQuery, pool: &PgPool) -> Result<Vec<Kline>, sqlx::Error> {
    let pg_interval = match query.interval.as_str() {
        "1m" => "1 minute",
        "5m" => "5 minutes",
        "15m" => "15 minutes",
        "1h" => "1 hour",
        "4h" => "4 hours",
        "1d" => "1 day",
        _ => "1 hour", 
    };

    let mut time_filter = String::new();
    if let Some(start) = &query.start_time {
        time_filter.push_str(&format!(" AND timestamp >= '{}'", start));
    }
    if let Some(end) = &query.end_time {
        time_filter.push_str(&format!(" AND timestamp <= '{}'", end));
    }

    let sql_query = format!(
        r#"
        SELECT 
            time_bucket('{}', timestamp) as bucket,
            first(price, timestamp) as open,
            max(price) as high,
            min(price) as low,
            last(price, timestamp) as close,
            sum(quantity) as volume,
            sum(quote_quantity) as quote_volume,
            count(*) as trades
        FROM trades 
        WHERE market = $1 {}
        GROUP BY bucket 
        ORDER BY bucket ASC
        "#,
        pg_interval, time_filter
    );

    let rows = sqlx::query(&sql_query)
        .bind(&query.market)
        .fetch_all(pool)
        .await?;

    let mut klines = Vec::new();

    for row in rows {
        let bucket: DateTime<Utc> = row.get("bucket");
        let open: rust_decimal::Decimal = row.get("open");
        let high: rust_decimal::Decimal = row.get("high");
        let low: rust_decimal::Decimal = row.get("low");
        let close: rust_decimal::Decimal = row.get("close");
        let volume: rust_decimal::Decimal = row.get("volume");
        let quote_volume: rust_decimal::Decimal = row.get("quote_volume");
        let trades: i64 = row.get("trades");

        let end_time = match query.interval.as_str() {
            "1m" => bucket + chrono::Duration::minutes(1),
            "5m" => bucket + chrono::Duration::minutes(5),
            "15m" => bucket + chrono::Duration::minutes(15),
            "1h" => bucket + chrono::Duration::hours(1),
            "4h" => bucket + chrono::Duration::hours(4),
            "1d" => bucket + chrono::Duration::days(1),
            _ => bucket + chrono::Duration::hours(1),
        };

        klines.push(Kline {
            open: open.to_string(),
            high: high.to_string(),
            low: low.to_string(),
            close: close.to_string(),
            volume: volume.to_string(),
            quote_volume: quote_volume.to_string(),
            trades: trades.to_string(),
            start: bucket.format("%Y-%m-%d %H:%M:%S").to_string(),
            end: end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

    Ok(klines)
}
