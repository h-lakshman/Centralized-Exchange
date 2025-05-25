use crate::{
    redis_manager::RedisManager,
    types::{CancelOrderRequest, GetOpenOrdersRequest, MessageToEngine, PlaceOrderRequest},
};
use actix_web::{web, HttpResponse, Responder};

pub async fn create_order(data: web::Json<PlaceOrderRequest>) -> impl Responder {
    let message_to_engine = MessageToEngine::CreateOrder(data.into_inner());

    let redis_manager = RedisManager::get_instance().await;
    match redis_manager.send_and_await(message_to_engine).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn cancel_order(data: web::Json<CancelOrderRequest>) -> impl Responder {
    let message_to_engine = MessageToEngine::CancelOrder(data.into_inner());

    let redis_manager = RedisManager::get_instance().await;
    match redis_manager.send_and_await(message_to_engine).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub async fn get_open_orders(data: web::Query<GetOpenOrdersRequest>) -> impl Responder {
    let message_to_engine = MessageToEngine::GetOpenOrders(data.into_inner());

    let redis_manager = RedisManager::get_instance().await;
    match redis_manager.send_and_await(message_to_engine).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
