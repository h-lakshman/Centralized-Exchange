use super::{Fill, Order, Orderbook};
use crate::{
    redis_manager::RedisManager,
    types::{MessageFromApi, MessageToApi, OrderCancelledPayload, OrderPlacedPayload, Side},
};
use rust_decimal::Decimal;
use std::{collections::HashMap, str::FromStr};

struct ProcessParams {
    message: MessageFromApi,
    client_id: String,
}

type UserBalance = HashMap<String, Balance>;
struct Balance {
    available: Decimal,
    locked: Decimal,
}

struct Engine {
    orderbooks: Vec<Orderbook>,
    balances: HashMap<String, UserBalance>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            orderbooks: Vec::new(),
            balances: HashMap::new(),
        }
    }

    fn process(&mut self, params: ProcessParams) {
        match params.message {
            MessageFromApi::CreateOrder(payload) => {
                let result: Result<MessageToApi, String> = self.create_order(
                    &payload.market,
                    &payload.price,
                    &payload.quantity,
                    &payload.side,
                    &payload.user_id,
                );
                let redis: &'static RedisManager = RedisManager::get_instance();
                match result {
                    Ok(order) => {
                        let _ = redis.send_to_api(params.client_id.clone(), order);
                    }
                    Err(e) => {
                        eprintln!("Create order error: {:?}", e);
                        let _ = redis.send_to_api(
                            params.client_id.clone(),
                            MessageToApi::OrderCancelled(OrderCancelledPayload {
                                order_id: "".to_string(),
                                executed_qty: 0,
                                remaining_qty: 0,
                            }),
                        );
                    }
                }
            }
            MessageFromApi::CancelOrder(cancel_order_payload) => todo!(),
            MessageFromApi::GetDepth(get_depth_payload) => todo!(),
            MessageFromApi::GetOpenOrders(get_open_orders_payload) => todo!(),
            MessageFromApi::OnRamp(on_ramp_payload) => todo!(),
        }
    }
    fn create_order(
        &mut self,
        market: &str,
        price: &str,
        quantity: &str,
        side: &Side,
        user_id: &str,
    ) -> Result<MessageToApi, String> {
        let orderbook_exists = self.orderbooks.iter().any(|ob| ob.ticker() == market);
        if !orderbook_exists {
            return Err("Orderbook not found".to_string());
        }

        let base_asset = market.split("_").next().expect("Invalid market");
        let quote_asset = market.split("_").nth(1).expect("Invalid market");

        self.check_and_lock_funds(base_asset, quote_asset, side, user_id, price, quantity)?;

        let mut order = Order {
            price: price.parse().unwrap(),
            quantity: quantity.parse().unwrap(),
            order_id: self.get_random_id(),
            filled: 0,
            side: side.as_str().to_string(),
            user_id: user_id.to_string(),
        };

        let orderbook = self
            .orderbooks
            .iter_mut()
            .find(|ob| ob.ticker() == market)
            .unwrap();
        let created = orderbook.add_order(&mut order);
        self.update_balances(
            user_id,
            base_asset,
            quote_asset,
            side,
            &created.fills,
            created.executed_quantity,
        );
        //create and update db trades,publishd to wsdepth and trades
        Ok(MessageToApi::OrderPlaced(OrderPlacedPayload {
            order_id: order.order_id,
            executed_qty: created.executed_quantity,
            fills: created
                .fills
                .iter()
                .map(|fill| crate::types::Fill {
                    price: fill.price.clone(),
                    qty: fill.qty,
                    trade_id: fill.trade_id,
                })
                .collect(),
        }))
    }

    fn cancel_order(&self) {
        let a = 1;
    }

    fn update_balances(
        &mut self,
        user_id: &str,
        base_asset: &str,
        quote_asset: &str,
        side: &Side,
        fills: &Vec<Fill>,
        executed_quantity: u64,
    ) {
        if side.as_str() == Side::Buy.as_str() {
            fills.iter().for_each(|fill| {
                //update quote balance
                let other_user = self.balances.get_mut(&fill.other_user_id).unwrap();
                other_user.get_mut(quote_asset).unwrap().available -=
                    Decimal::from_str(&fill.price).unwrap()
                        * Decimal::from_str(&fill.qty.to_string()).unwrap();
                let currrent_user = self.balances.get_mut(user_id).unwrap();
                currrent_user.get_mut(quote_asset).unwrap().locked +=
                    Decimal::from_str(&fill.price).unwrap()
                        * Decimal::from_str(&fill.qty.to_string()).unwrap();

                //update base balance
                let other_user = self.balances.get_mut(&fill.other_user_id).unwrap();
                other_user.get_mut(base_asset).unwrap().locked -=
                    Decimal::from_str(&fill.qty.to_string()).unwrap();
                let current_user = self.balances.get_mut(user_id).unwrap();
                current_user.get_mut(base_asset).unwrap().locked +=
                    Decimal::from_str(&fill.qty.to_string()).unwrap();
            })
        } else {
            fills.iter().for_each(|fill| {
                //update quote balance
                let other_user = self.balances.get_mut(&fill.other_user_id).unwrap();
                other_user.get_mut(quote_asset).unwrap().locked -=
                    Decimal::from_str(&fill.qty.to_string()).unwrap()
                        * Decimal::from_str(&fill.price).unwrap();
                let current_user = self.balances.get_mut(user_id).unwrap();
                current_user.get_mut(quote_asset).unwrap().available +=
                    Decimal::from_str(&fill.qty.to_string()).unwrap()
                        * Decimal::from_str(&fill.price).unwrap();

                //update base asset
                let other_user = self.balances.get_mut(&fill.other_user_id).unwrap();
                other_user.get_mut(base_asset).unwrap().available +=
                    Decimal::from_str(&fill.qty.to_string()).unwrap();
                let current_user = self.balances.get_mut(user_id).unwrap();
                current_user.get_mut(base_asset).unwrap().locked -=
                    Decimal::from_str(&fill.qty.to_string()).unwrap();
            })
        }
    }

    fn check_and_lock_funds(
        &mut self,
        base_asset: &str,
        quote_asset: &str,
        side: &Side,
        user_id: &str,
        price: &str,
        quantity: &str,
    ) -> Result<(), String> {
        let user = match self.balances.get_mut(user_id) {
            Some(user) => user,
            None => return Err("User not found".to_string()),
        };
        match side {
            Side::Buy => {
                let user_quote_balance = match user.get_mut(quote_asset) {
                    Some(balance) => balance,
                    None => return Err("User quote balance not found".to_string()),
                };

                if user_quote_balance.available
                    < Decimal::from_str(price).unwrap() * Decimal::from_str(quantity).unwrap()
                {
                    return Err("Insufficient quote balance".to_string());
                }

                user_quote_balance.available -=
                    Decimal::from_str(price).unwrap() * Decimal::from_str(quantity).unwrap();
                user_quote_balance.locked += Decimal::from_str(quantity).unwrap();
                Ok(())
            }
            Side::Sell => {
                let user_base_balance = match user.get_mut(base_asset) {
                    Some(balance) => balance,
                    None => return Err("User base balance not found".to_string()),
                };

                if user_base_balance.available < Decimal::from_str(quantity).unwrap() {
                    return Err("Insufficient base balance".to_string());
                }

                user_base_balance.available -= Decimal::from_str(quantity).unwrap();
                user_base_balance.locked += Decimal::from_str(quantity).unwrap();
                Ok(())
            }
        }
    }

    pub fn get_random_id(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}{:x}", rng.r#gen::<u64>(), rng.r#gen::<u64>())
    }
}
