use crate::{
    redis_manager::RedisManager,
    types::{
        CancelOrderRequest, EngineMessageData, GetOpenOrdersRequest, MessageToEngine,
        MessageToType, PlaceOrderRequest,
    },
};
use actix_web::{web, HttpResponse, Responder};

pub async fn create_order(data: web::Json<PlaceOrderRequest>) -> impl Responder {
    let message_to_engine = MessageToEngine {
        message_type: MessageToType::PlaceOrder,
        data: EngineMessageData::PlaceOrder(data.into_inner()),
    };

    let redis_manager = RedisManager::get_instance();
    match redis_manager.lock() {
        Ok(manager) => match manager.send_and_await(message_to_engine).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Redis manager lock error: {}", e))
        }
    }
}

pub async fn cancel_order(data: web::Json<CancelOrderRequest>) -> impl Responder {
    let message_to_engine = MessageToEngine {
        message_type: MessageToType::CancelOrder,
        data: EngineMessageData::CancelOrder(data.into_inner()),
    };

    let redis_manager = RedisManager::get_instance();
    match redis_manager.lock() {
        Ok(manager) => match manager.send_and_await(message_to_engine).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Redis manager lock error: {}", e))
        }
    }
}

pub async fn get_open_orders(data: web::Query<GetOpenOrdersRequest>) -> impl Responder {
    let message_to_engine = MessageToEngine {
        message_type: MessageToType::GetOpenOrders,
        data: EngineMessageData::GetOpenOrders(data.into_inner()),
    };

    let redis_manager = RedisManager::get_instance();
    match redis_manager.lock() {
        Ok(manager) => match manager.send_and_await(message_to_engine).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Redis manager lock error: {}", e))
        }
    }
}
