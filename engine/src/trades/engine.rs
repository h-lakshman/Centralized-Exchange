use super::{Orderbook, OrderbookFill};
use crate::redis_manager::RedisManager;
use chrono::Utc;
use engine::types::{
    DbMessage, DbMessageData, DbMessageType, DepthUpdateMessage, InternalCreateOrderPayload,
    InternalMessage, MessageFromApi, MessageToApi, Order, OrderCancelledPayload,
    OrderPlacedPayload, OrderUpdate, Side, TradeAdd, TradeUpdateMessage, WsMessage, WsPayload,
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

pub const BASE_CURRENCY: &str = "INR";

pub struct ProcessParams {
    pub message: InternalMessage,
    pub client_id: String,
}

impl ProcessParams {
    pub fn from_api_message(
        api_message: MessageFromApi,
        client_id: String,
    ) -> Result<Self, String> {
        let internal_message = InternalMessage::from_api_message(api_message)?;
        Ok(ProcessParams {
            message: internal_message,
            client_id,
        })
    }
}

type UserBalance = HashMap<String, Balance>;
struct Balance {
    available: Decimal,
    locked: Decimal,
}

pub struct Engine {
    orderbooks: Vec<Orderbook>,
    balances: HashMap<String, UserBalance>,
}

impl Engine {
    pub fn new() -> Self {
        let orderbook = Orderbook::new("TATA".to_string(), vec![], vec![], None, None);
        let mut balances: HashMap<String, Balance> = HashMap::new();
        balances.insert(
            BASE_CURRENCY.to_string(),
            Balance {
                available: Decimal::from_str("1000000").unwrap(),
                locked: Decimal::from_str("0").unwrap(),
            },
        );
        balances.insert(
            "TATA".to_string(),
            Balance {
                available: Decimal::from_str("1000000").unwrap(),
                locked: Decimal::from_str("0").unwrap(),
            },
        );

        let mut user_balances: HashMap<String, UserBalance> = HashMap::new();
        user_balances.insert("1".to_string(), balances);

        Self {
            orderbooks: vec![orderbook],
            balances: user_balances,
        }
    }

    fn u64_to_decimal(value: u64) -> Decimal {
        Decimal::from(value)
    }

    pub fn process(&mut self, params: ProcessParams) {
        match params.message {
            InternalMessage::CreateOrder(payload) => {
                let result: Result<MessageToApi, String> = self.create_order(payload);
                let redis = RedisManager::get_instance();
                match result {
                    Ok(order) => {
                        if let Err(e) = redis.send_to_api(params.client_id.clone(), order) {
                            eprintln!("Failed to send order placed message to Redis: {:?}", e);
                        }
                    }
                    Err(e) => {
                        if let Err(redis_err) = redis.send_to_api(
                            params.client_id.clone(),
                            MessageToApi::OrderCancelled(OrderCancelledPayload {
                                order_id: "".to_string(),
                                executed_qty: 0,
                                remaining_qty: 0,
                            }),
                        ) {
                            eprintln!(
                                "Failed to send order cancelled message to Redis: {:?}",
                                redis_err
                            );
                        }
                    }
                }
            }
            InternalMessage::CancelOrder(cancel_order_payload) => {
                let order_id = cancel_order_payload.order_id;
                let cancel_market = cancel_order_payload.market;
                let quote_asset = cancel_market.split("_").next().unwrap();
                let cancel_orderbook = match self
                    .orderbooks
                    .iter_mut()
                    .find(|ob| ob.ticker() == cancel_market)
                {
                    Some(ob) => ob,
                    None => {
                        eprintln!("Orderbook not found");
                        return;
                    }
                };
                let order = cancel_orderbook
                    .asks
                    .iter()
                    .find_map(|(_, orders)| orders.iter().find(|o| o.order_id == order_id))
                    .or_else(|| {
                        cancel_orderbook
                            .bids
                            .iter()
                            .find_map(|(_, orders)| orders.iter().find(|o| o.order_id == order_id))
                    })
                    .cloned();

                let order = match order {
                    Some(o) => o,
                    None => {
                        eprintln!("Order to be cancelled was not found");
                        return;
                    }
                };

                match order.side {
                    Side::Buy => {
                        if let Some(price) = cancel_orderbook.cancel_bid(&order) {
                            let left_quantity_decimal =
                                Decimal::from((order.quantity - order.filled) * order.price);
                            let balance = self.balances.get_mut(&order.user_id).unwrap();
                            balance.get_mut(BASE_CURRENCY).unwrap().available +=
                                left_quantity_decimal;
                            balance.get_mut(BASE_CURRENCY).unwrap().locked -= left_quantity_decimal;

                            self.send_updated_depth_at(&price.to_string(), &cancel_market);
                        }
                    }
                    Side::Sell => {
                        if let Some(price) = cancel_orderbook.cancel_ask(&order) {
                            let left_quantity_decimal =
                                Decimal::from(order.quantity - order.filled);
                            let balance = self.balances.get_mut(&order.user_id).unwrap();
                            balance.get_mut(quote_asset).unwrap().available +=
                                left_quantity_decimal;
                            balance.get_mut(quote_asset).unwrap().locked -= left_quantity_decimal;

                            self.send_updated_depth_at(&price.to_string(), &cancel_market);
                        }
                    }
                }
                if let Err(e) = RedisManager::get_instance().send_to_api(
                    params.client_id,
                    MessageToApi::OrderCancelled(OrderCancelledPayload {
                        order_id,
                        executed_qty: 0,
                        remaining_qty: 0,
                    }),
                ) {
                    eprintln!("Failed to send order cancelled message to Redis: {:?}", e);
                }
            }
            InternalMessage::GetOpenOrders(get_open_orders_payload) => {
                match self
                    .orderbooks
                    .iter()
                    .find(|ob| ob.ticker() == get_open_orders_payload.market)
                {
                    Some(open_order_book) => {
                        let open_orders =
                            open_order_book.get_open_orders(get_open_orders_payload.user_id);
                        if let Err(e) = RedisManager::get_instance()
                            .send_to_api(params.client_id, MessageToApi::OpenOrders(open_orders))
                        {
                            eprintln!("Failed to send open orders message to Redis: {:?}", e);
                        }
                    }
                    None => {
                        eprint!("No orderbook found");
                        return;
                    }
                }
            }
            InternalMessage::GetDepth(get_depth_payload) => {
                let market = get_depth_payload.market;
                match self.orderbooks.iter().find(|ob| ob.ticker() == market) {
                    Some(orderbook) => {
                        if let Err(e) = RedisManager::get_instance().send_to_api(
                            params.client_id,
                            MessageToApi::Depth(orderbook.get_depth()),
                        ) {
                            eprintln!("Failed to send depth message to Redis: {:?}", e);
                        }
                    }
                    None => {}
                }
            }
            InternalMessage::OnRamp(on_ramp_payload) => {
                let user_id = on_ramp_payload.user_id;
                let amount = on_ramp_payload.amount;
                self.on_ramp(&user_id, amount);
            }
        }
    }

    pub fn add_orderbook(&mut self, orderbook: Orderbook) {
        self.orderbooks.push(orderbook);
    }

    fn create_order(
        &mut self,
        payload: InternalCreateOrderPayload,
    ) -> Result<MessageToApi, String> {
        let orderbook_exists = self
            .orderbooks
            .iter()
            .any(|ob| ob.ticker() == payload.market);
        if !orderbook_exists {
            return Err("Orderbook not found".to_string());
        }

        let mut market_parts = payload.market.split("_");
        let base_asset = market_parts.next().expect("Invalid market");
        let quote_asset = market_parts.next().expect("Invalid market");

        self.check_and_lock_funds(
            base_asset,
            quote_asset,
            &payload.side,
            &payload.user_id,
            payload.price_decimal,
            payload.quantity_decimal,
        )?;

        let mut order = Order {
            price: payload.price,
            quantity: payload.quantity,
            order_id: self.get_random_id(),
            filled: 0,
            side: payload.side.clone(),
            user_id: payload.user_id.clone(),
        };

        let orderbook = self
            .orderbooks
            .iter_mut()
            .find(|ob| ob.ticker() == payload.market)
            .unwrap();
        let created = orderbook.add_order(&mut order);
        self.update_balances(
            &payload.user_id,
            base_asset,
            quote_asset,
            &payload.side,
            &created.fills,
            created.executed_quantity,
        );

        let timestamp = Utc::now().to_string();
        self.create_db_trades(&created.fills, &payload.market, &payload.side, &timestamp);
        self.update_db_orders(
            &order,
            created.executed_quantity,
            &created.fills,
            &payload.market,
        );
        self.publish_ws_depth_updates(
            &created.fills,
            &payload.price.to_string(),
            &payload.market,
            &payload.side,
        );
        self.publish_ws_trades(&created.fills, &payload.market, &payload.side);
        Ok(MessageToApi::OrderPlaced(OrderPlacedPayload {
            order_id: order.order_id,
            executed_qty: created.executed_quantity,
            fills: created
                .fills
                .iter()
                .map(|fill| fill.fill.to_external_fill())
                .collect(),
        }))
    }

    fn update_balances(
        &mut self,
        user_id: &str,
        base_asset: &str,
        quote_asset: &str,
        side: &Side,
        fills: &Vec<OrderbookFill>,
        executed_quantity: u64,
    ) {
        match side {
            Side::Buy => {
                fills.iter().for_each(|fill| {
                    // Use pre-calculated decimal values - no parsing needed!
                    let fill_qty_decimal = Self::u64_to_decimal(fill.fill.qty);
                    let fill_amount_decimal = fill.fill.price_decimal * fill_qty_decimal;

                    //update quote balance
                    let other_user = self.balances.get_mut(&fill.other_user_id).unwrap();
                    other_user.get_mut(quote_asset).unwrap().available -= fill_amount_decimal;
                    let currrent_user = self.balances.get_mut(user_id).unwrap();
                    currrent_user.get_mut(quote_asset).unwrap().locked += fill_amount_decimal;

                    //update base balance
                    let other_user = self.balances.get_mut(&fill.other_user_id).unwrap();
                    other_user.get_mut(base_asset).unwrap().locked -= fill_qty_decimal;
                    let current_user = self.balances.get_mut(user_id).unwrap();
                    current_user.get_mut(base_asset).unwrap().locked += fill_qty_decimal;
                });
            }
            Side::Sell => {
                fills.iter().for_each(|fill| {
                    let fill_qty_decimal = Self::u64_to_decimal(fill.fill.qty);
                    let fill_amount_decimal = fill.fill.price_decimal * fill_qty_decimal;

                    //update quote balance
                    let other_user = self.balances.get_mut(&fill.other_user_id).unwrap();
                    other_user.get_mut(quote_asset).unwrap().locked -= fill_amount_decimal;
                    let current_user = self.balances.get_mut(user_id).unwrap();
                    current_user.get_mut(quote_asset).unwrap().available += fill_amount_decimal;

                    //update base asset
                    let other_user = self.balances.get_mut(&fill.other_user_id).unwrap();
                    other_user.get_mut(base_asset).unwrap().available += fill_qty_decimal;
                    let current_user = self.balances.get_mut(user_id).unwrap();
                    current_user.get_mut(base_asset).unwrap().locked -= fill_qty_decimal;
                });
            }
        }
    }

    fn on_ramp(&mut self, user_id: &str, amount: Decimal) {
        match self.balances.get_mut(user_id) {
            Some(user_balance) => {
                if let Some(base_balance) = user_balance.get_mut(BASE_CURRENCY) {
                    base_balance.available += amount;
                } else {
                    user_balance.insert(
                        BASE_CURRENCY.to_string(),
                        Balance {
                            available: amount,
                            locked: Decimal::from(0),
                        },
                    );
                }
            }
            None => {
                let mut new_balance = UserBalance::new();
                new_balance.insert(
                    BASE_CURRENCY.to_string(),
                    Balance {
                        available: amount,
                        locked: Decimal::from(0),
                    },
                );
                self.balances.insert(user_id.to_string(), new_balance);
            }
        }
    }

    fn check_and_lock_funds(
        &mut self,
        base_asset: &str,
        quote_asset: &str,
        side: &Side,
        user_id: &str,
        price_decimal: Decimal,
        quantity_decimal: Decimal,
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

                let required_quote_amount = price_decimal * quantity_decimal;
                if user_quote_balance.available < required_quote_amount {
                    return Err("Insufficient quote balance".to_string());
                }

                user_quote_balance.available -= required_quote_amount;
                user_quote_balance.locked += required_quote_amount; // Lock quote currency amount, not just quantity
                Ok(())
            }
            Side::Sell => {
                let user_base_balance = match user.get_mut(base_asset) {
                    Some(balance) => balance,
                    None => return Err("User base balance not found".to_string()),
                };

                if user_base_balance.available < quantity_decimal {
                    return Err("Insufficient base balance".to_string());
                }

                user_base_balance.available -= quantity_decimal;
                user_base_balance.locked += quantity_decimal;
                Ok(())
            }
        }
    }

    fn send_updated_depth_at(&self, price: &str, market: &str) {
        let depth = match self.orderbooks.iter().find(|ob| ob.ticker() == market) {
            Some(orderbook) => orderbook.get_depth(),
            None => {
                eprintln!("Orderbook not found");
                return;
            }
        };

        let updated_bids: Vec<[String; 2]> = depth
            .bids
            .iter()
            .filter(|x| x.get(0).map_or(false, |p| p == price))
            .map(|x| [x[0].clone(), x[1].clone()])
            .collect();

        let updated_asks: Vec<[String; 2]> = depth
            .asks
            .iter()
            .filter(|x| x.get(0).map_or(false, |p| p == price))
            .map(|x| [x[0].clone(), x[1].clone()])
            .collect();

        if let Err(e) = RedisManager::get_instance().publish_message(
            format!("depth@{}", market),
            WsMessage {
                stream: format!("depth@{}", market),
                data: WsPayload::Depth(DepthUpdateMessage {
                    e: "depth".to_string(),
                    a: updated_asks,
                    b: updated_bids,
                }),
            },
        ) {
            eprintln!("Failed to send depth update message to Redis: {:?}", e);
        }
    }

    fn publish_ws_depth_updates(
        &self,
        fills: &Vec<OrderbookFill>,
        price: &str,
        market: &str,
        side: &Side,
    ) {
        let depth = match self.orderbooks.iter().find(|ob| ob.ticker() == market) {
            Some(orderbook) => orderbook.get_depth(),
            None => {
                eprintln!("Orderbook not found");
                return;
            }
        };
        let fill_prices: Vec<String> = fills
            .iter()
            .map(|f| &f.fill.price_string)
            .cloned()
            .collect();
        if let Side::Buy = side {
            let updated_asks: Vec<[String; 2]> = depth
                .asks
                .iter()
                .filter(|x| x.get(0).map_or(false, |p| fill_prices.contains(p)))
                .map(|x| [x[0].clone(), x[1].clone()])
                .collect();
            let updated_bids = depth
                .bids
                .iter()
                .filter(|x: &&[String; 2]| x[0] == price)
                .map(|x| [x[0].clone(), x[1].clone()])
                .collect();
            println!("publishing updated depth");
            if let Err(e) = RedisManager::get_instance().publish_message(
                format!("depth@{}", market),
                WsMessage {
                    stream: format!("depth@{}", market),
                    data: WsPayload::Depth(DepthUpdateMessage {
                        e: "depth".to_string(),
                        a: updated_asks,
                        b: updated_bids,
                    }),
                },
            ) {
                eprintln!("Failed to send depth update message to Redis: {:?}", e);
            }
        } else {
            let updated_bids: Vec<[String; 2]> = depth
                .bids
                .iter()
                .filter(|x| x.get(0).map_or(false, |price| fill_prices.contains(price)))
                .map(|x| [x[0].clone(), x[1].clone()])
                .collect();
            let updated_asks = depth
                .asks
                .iter()
                .filter(|x: &&[String; 2]| x[0] == price)
                .map(|x| [x[0].clone(), x[1].clone()])
                .collect();
            println!("publishing updated depth");
            if let Err(e) = RedisManager::get_instance().publish_message(
                format!("depth@{}", market),
                WsMessage {
                    stream: format!("depth@{}", market),
                    data: WsPayload::Depth(DepthUpdateMessage {
                        e: "depth".to_string(),
                        a: updated_asks,
                        b: updated_bids,
                    }),
                },
            ) {
                eprintln!("Failed to send depth update message to Redis: {:?}", e);
            }
        }
    }

    fn publish_ws_trades(&self, fills: &Vec<OrderbookFill>, market: &str, side: &Side) {
        fills.iter().for_each(|fill| {
            let is_buyer_maker = matches!(side, Side::Sell);
            if let Err(e) = RedisManager::get_instance().publish_message(
                format!("trades@{}", market),
                WsMessage {
                    stream: format!("trades@{}", market),
                    data: WsPayload::Trade(TradeUpdateMessage {
                        e: "trade".to_string(),
                        t: fill.fill.trade_id,
                        m: is_buyer_maker,
                        p: fill.fill.price_string.clone(),
                        q: fill.fill.qty.to_string(),
                        s: market.to_string(),
                    }),
                },
            ) {
                eprintln!("Failed to send trade update message to Redis: {:?}", e);
            }
        });
    }

    fn create_db_trades(
        &self,
        fills: &Vec<OrderbookFill>,
        market: &str,
        side: &Side,
        timestamp: &str,
    ) {
        fills.iter().for_each(|fill| {
            let quote_quantity = fill.fill.qty.checked_mul(fill.fill.price_u64).unwrap();
            if let Err(e) = RedisManager::get_instance().push_message(DbMessage {
                db_message_type: DbMessageType::TradeAdded,
                data: DbMessageData::TradeAdd(TradeAdd {
                    id: fill.fill.trade_id.to_string(),
                    is_buyer_maker: matches!(side, Side::Sell),
                    price: fill.fill.price_string.clone(),
                    quantity: fill.fill.qty.to_string(),
                    quote_quantity: quote_quantity.to_string(),
                    timestamp: timestamp.to_string(),
                    market: market.to_string(),
                }),
            }) {
                eprintln!("Failed to push trade added message to Redis: {:?}", e);
            }
        });
    }

    fn update_db_orders(
        &self,
        order: &Order,
        executed_quantity: u64,
        fills: &Vec<OrderbookFill>,
        market: &str,
    ) {
        if let Err(e) = RedisManager::get_instance().push_message(DbMessage {
            db_message_type: DbMessageType::OrderUpdate,
            data: DbMessageData::OrderUpdate(OrderUpdate {
                order_id: order.order_id.clone(),
                executed_quantity,
                price: Some(order.price.to_string()),
                market: Some(market.to_string()),
                quantity: Some(order.quantity.to_string()),
                side: Some(order.side.clone()),
            }),
        }) {
            eprintln!("Failed to push order update message to Redis: {:?}", e);
        }
        fills.iter().for_each(|fill| {
            if let Err(e) = RedisManager::get_instance().push_message(DbMessage {
                db_message_type: DbMessageType::OrderUpdate,
                data: DbMessageData::OrderUpdate(OrderUpdate {
                    order_id: fill.fill.trade_id.to_string(),
                    executed_quantity: fill.fill.qty,
                    price: None,
                    market: None,
                    quantity: None,
                    side: None,
                }),
            }) {
                eprintln!("Failed to push order update message to Redis: {:?}", e);
            }
        });
    }

    pub fn get_random_id(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let id1 = rng.gen::<u64>();
        let id2 = rng.gen::<u64>();
        format!("{:x}{:x}", id1, id2)
    }
}
