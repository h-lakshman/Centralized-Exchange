use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::env;

mod handlers;
mod redis_manager;
mod types;
use handlers::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database pool");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::JsonConfig::default())
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/v1")
                    .route("/order", web::post().to(create_order))
                    .route("/order", web::delete().to(cancel_order))
                    .route("/order/open", web::get().to(get_open_orders))
                    .route("/depth", web::get().to(get_depth))
                    .route("/klines", web::get().to(get_klines))
                    .route("/tickers", web::get().to(get_tickers))
                    .route("/trades", web::get().to(get_trades)),
            )
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
