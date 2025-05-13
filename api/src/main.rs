use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

mod handlers;
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
                        .service(web::scope("/order").route("/", web::post().to(create_order))),
            )
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}
