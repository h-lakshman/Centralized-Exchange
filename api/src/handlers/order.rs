use actix_web::{web, HttpResponse, Responder};
use crate::types::{PlaceOrderRequest, MessageToEngine, MessageType};

pub async fn create_order(
    data: web::Json<PlaceOrderRequest>,
) -> impl Responder {
   let message_to_engine = MessageToEngine {
    message_type: MessageType::PlaceOrder,
    data: data.into_inner(),
   };
   //todo: send message to engine
   HttpResponse::Ok().json(message_to_engine)
}
