use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct KlinesQuery {
    market: String,
    interval: String,
    start_time: String,
    end_time: String,
}

pub async fn get_klines(data: web::Query<KlinesQuery>) -> impl Responder {
    let query = data.into_inner();
    //TODO: get klines from DB
    HttpResponse::Ok().json({})
}
