use actix_web::{HttpResponse, Responder};

pub async fn get_trades() -> impl Responder {
    HttpResponse::Ok().json({})
}
