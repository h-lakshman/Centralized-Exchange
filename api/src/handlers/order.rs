use actix_web::{web, HttpResponse, Responder};
use crate::types::{PlaceOrderRequest, MessageToEngine, MessageType, EngineMessageData, CancelOrderRequest, GetOpenOrdersRequest};

pub async fn create_order(
    data: web::Json<PlaceOrderRequest>,
) -> impl Responder {
   let message_to_engine = MessageToEngine {
    message_type: MessageType::PlaceOrder,
    data: EngineMessageData::PlaceOrder(data.into_inner()),
   };
   //todo: send message to engine
   HttpResponse::Ok().json(message_to_engine)
}

pub async fn cancel_order(
    data: web::Json<CancelOrderRequest>,
) -> impl Responder {
    let message_to_engine = MessageToEngine {
        message_type: MessageType::CancelOrder,
        data: EngineMessageData::CancelOrder(data.into_inner()),
    };
    //todo: send message to engine
    HttpResponse::Ok().json(message_to_engine)
}

pub async fn get_open_orders(
    data: web::Json<GetOpenOrdersRequest>,
) -> impl Responder {
    let message_to_engine = MessageToEngine {
        message_type: MessageType::GetOpenOrders,
        data: EngineMessageData::GetOpenOrders(data.into_inner()),
    };
    //todo: send message to engine
    HttpResponse::Ok().json(message_to_engine)
}
