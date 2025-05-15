use actix_web::{web, HttpResponse, Responder};

use crate::{
    redis_manager::RedisManager,
    types::{EngineMessageData, GetDepthRequest, MessageToEngine, MessageToType},
};

pub async fn get_depth(symbol: web::Query<String>) -> impl Responder {
    let symbol = symbol.into_inner();
    let message_to_engine = MessageToEngine {
        message_type: MessageToType::GetDepth,
        data: EngineMessageData::GetDepth(GetDepthRequest { market: symbol }),
    };
    let redis_manager = RedisManager::get_instance();

    match redis_manager.send_and_await(message_to_engine).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
