use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

mod handlers;
mod redis_manager;
mod types;
use handlers::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::JsonConfig::default())
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/order").route("/", web::post().to(create_order)))
                    .service(web::scope("/order").route("/", web::delete().to(cancel_order)))
                    .service(web::scope("/order").route("/open", web::get().to(get_open_orders))),
            )
            .service(web::scope("/depth").route("/", web::get().to(get_depth)))
            .service(web::scope("/klines").route("/", web::get().to(get_klines)))
            .service(web::scope("/tickers").route("/", web::get().to(get_tickers)))
            .service(web::scope("/trades").route("/", web::get().to(get_trades)))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
