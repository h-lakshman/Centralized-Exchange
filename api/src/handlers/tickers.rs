use actix_web::{HttpResponse, Responder};

pub async fn get_tickers() -> impl Responder {
    HttpResponse::Ok().json({})
}
