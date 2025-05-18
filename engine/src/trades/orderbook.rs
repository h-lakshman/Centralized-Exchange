use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashMap;

use crate::types::{DepthPayload, Fill, Order, Side};

use super::BASE_CURRENCY;

// Extended Fill struct with additional fields needed for orderbook
pub struct OrderbookFill {
    pub fill: Fill,
    pub other_user_id: String,
    pub marker_order_id: String,
}

pub struct OrderCreated {
    pub executed_quantity: u64,
    pub fills: Vec<OrderbookFill>,
}

pub struct Orderbook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub base_asset: String,
    pub quote_asset: String,
    pub last_trade_id: u64,
    pub current_price: u64,
}

//add self trade protection
impl Orderbook {
    //remember to pass last_trade_id and current_price as Option<u64>
    pub fn new(
        base_asset: String,
        bids: Vec<Order>,
        asks: Vec<Order>,
        last_trade_id: Option<u64>,
        current_price: Option<u64>,
    ) -> Self {
        Orderbook {
            bids,
            asks,
            base_asset,
            quote_asset: BASE_CURRENCY.to_string(),
            last_trade_id: last_trade_id.unwrap_or(0),
            current_price: current_price.unwrap_or(0),
        }
    }

    pub fn ticker(&self) -> String {
        format!("{}_{}", self.base_asset, self.quote_asset)
    }
    pub fn add_order(&mut self, order: &mut Order) -> OrderCreated {
        match order.side {
            Side::Buy => {
                let ongoing_order = self.match_asks(order);
                order.filled = ongoing_order.executed_quantity;
                if ongoing_order.executed_quantity == order.quantity {
                    return ongoing_order;
                }
                self.bids.push(order.clone());
                return ongoing_order;
            }
            Side::Sell => {
                let ongoing_order = self.match_bids(order);
                order.filled = ongoing_order.executed_quantity;
                if ongoing_order.executed_quantity == order.quantity {
                    return ongoing_order;
                }
                self.asks.push(order.clone());
                return ongoing_order;
            }
            _ => panic!("Invalid order side: {}", order.side.as_str()),
        }
    }

    fn match_asks(&mut self, order: &mut Order) -> OrderCreated {
        let mut fills: Vec<OrderbookFill> = Vec::new();
        let mut executed_quantity: u64 = 0;

        self.asks.sort_by_key(|ask| ask.price);
        let mut i = 0;
        while i < self.asks.len() && executed_quantity < order.quantity {
            let ask = &mut self.asks[i];
            if ask.price <= order.price {
                let remaining_ask_qty = ask.quantity - ask.filled;
                let filled_qty = min(remaining_ask_qty, order.quantity - executed_quantity);

                executed_quantity += filled_qty;
                ask.filled += filled_qty;
                self.last_trade_id += 1;

                fills.push(OrderbookFill {
                    fill: Fill {
                        price: ask.price.to_string(),
                        qty: filled_qty,
                        trade_id: self.last_trade_id,
                    },
                    other_user_id: ask.user_id.clone(),
                    marker_order_id: order.order_id.clone(),
                });
            }
            i += 1;
        }

        self.asks.retain(|ask| ask.filled < ask.quantity);

        OrderCreated {
            fills,
            executed_quantity,
        }
    }

    fn match_bids(&mut self, order: &mut Order) -> OrderCreated {
        let mut fills: Vec<OrderbookFill> = Vec::new();
        let mut executed_qty: u64 = 0;

        self.bids.sort_by(|a, b| b.price.cmp(&a.price));
        let mut i = 0;
        while i < self.bids.len() && executed_qty < order.quantity {
            let bid = &mut self.bids[i];
            if bid.price >= order.price {
                let remaining_ask_qty = bid.quantity - bid.filled;
                let amount_remaining = min(remaining_ask_qty, order.quantity - executed_qty);

                executed_qty += amount_remaining;
                bid.filled += amount_remaining;
                self.last_trade_id += 1;

                fills.push(OrderbookFill {
                    fill: Fill {
                        price: bid.price.to_string(),
                        qty: amount_remaining,
                        trade_id: self.last_trade_id,
                    },
                    other_user_id: bid.user_id.clone(),
                    marker_order_id: bid.order_id.clone(),
                });
            }
            i += 1;
        }
        self.asks.retain(|bid| bid.filled < bid.quantity);
        OrderCreated {
            fills,
            executed_quantity: executed_qty,
        }
    }

    //make this faster ,compute this during order matching
    pub fn get_depth(&self) -> DepthPayload {
        let mut bids: Vec<[String; 2]> = Vec::new();
        let mut asks: Vec<[String; 2]> = Vec::new();

        let mut bids_map: HashMap<String, u64> = HashMap::new();
        let mut asks_map: HashMap<String, u64> = HashMap::new();

        for order in &self.bids {
            let price = order.price.to_string();
            *bids_map.entry(price).or_insert(0) += order.quantity;
        }
        for order in &self.asks {
            let price = order.price.to_string();
            *asks_map.entry(price).or_insert(0) += order.quantity;
        }

        for (price, qty) in bids_map {
            bids.push([price, qty.to_string()]);
        }
        for (price, qty) in asks_map {
            asks.push([price, qty.to_string()]);
        }
        DepthPayload { bids, asks }
    }

    pub fn cancel_bid(&mut self, order: &Order) -> Option<u64> {
        let index = self
            .bids
            .iter()
            .position(|bid| bid.order_id == order.order_id);
        if let Some(index) = index {
            let price: u64 = self.bids[index].price;
            self.bids.remove(index);
            return Some(price);
        }
        return None;
    }

    pub fn cancel_ask(&mut self, order: &Order) -> Option<u64> {
        let index = self
            .asks
            .iter()
            .position(|ask| ask.order_id == order.order_id);
        if let Some(index) = index {
            let price: u64 = self.asks[index].price;
            self.asks.remove(index);
            return Some(price);
        }
        return None;
    }

    pub fn get_open_orders(&self, user_id: String) -> Vec<Order> {
        self.bids
            .iter()
            .chain(self.asks.iter())
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect()
    }
}
