use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

mod handlers;
mod redis_manager;
mod types;
use handlers::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::JsonConfig::default())
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/order").route("/", web::post().to(create_order)))
                    .service(web::scope("/order").route("/", web::delete().to(cancel_order)))
                    .service(
                        web::scope("/order").route("/order_id", web::put().to(get_open_orders)),
                    ),
            )
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
