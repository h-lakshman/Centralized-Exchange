use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct Order {
    pub price: u64,
    pub quantity: u64,
    pub order_id: String,
    pub filled: u64,
    pub side: String,
    pub user_id: String,
}

pub struct Fill {
    pub price: String,
    pub qty: u64,
    pub trade_id: u64,
    pub other_user_id: String,
    pub marker_order_id: String,
}

pub struct OrderCreated {
    pub executed_quantity: u64,
    pub fills: Vec<Fill>,
}

pub struct OrderbookDepth {
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

pub enum Side {
    Buy,
    Sell,
}

pub struct Orderbook {
    bids: Vec<Order>,
    asks: Vec<Order>,
    base_asset: String,
    //Todo: set quote_asset to base currency from engine
    quote_asset: String,
    last_trade_id: u64,
    current_price: u64,
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
            //Todo: set quote_asset to base currency from engine,this is a temp fix
            quote_asset: "USD".to_string(),
            last_trade_id: last_trade_id.unwrap_or(0),
            current_price: current_price.unwrap_or(0),
        }
    }

    pub fn ticker(&self) -> String {
        format!("{}_{}", self.base_asset, self.quote_asset)
    }
    pub fn add_order(&mut self, order: &mut Order) -> OrderCreated {
        match order.side.as_str() {
            "buy" => {
                let ongoing_order = self.match_asks(order);
                order.filled = ongoing_order.executed_quantity;
                if ongoing_order.executed_quantity == order.quantity {
                    return ongoing_order;
                }
                self.bids.push(order.clone());
                return ongoing_order;
            }
            "sell" => {
                let ongoing_order = self.match_bids(order);
                order.filled = ongoing_order.executed_quantity;
                if ongoing_order.executed_quantity == order.quantity {
                    return ongoing_order;
                }
                self.asks.push(order.clone());
                return ongoing_order;
            }
            _ => panic!("Invalid order side: {}", order.side),
        }
    }

    fn match_asks(&mut self, order: &mut Order) -> OrderCreated {
        let mut fills: Vec<Fill> = Vec::new();
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

                fills.push(Fill {
                    price: ask.price.to_string(),
                    qty: filled_qty,
                    trade_id: self.last_trade_id,
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
        let mut fills: Vec<Fill> = Vec::new();
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

                fills.push(Fill {
                    price: bid.price.to_string(),
                    qty: amount_remaining,
                    trade_id: self.last_trade_id,
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
    fn get_depth(&self) -> OrderbookDepth {
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
        OrderbookDepth { bids, asks }
    }

    fn cancel_bid(&mut self, order: Order) -> u64 {
        let index = self
            .bids
            .iter()
            .position(|bid| bid.order_id == order.order_id);
        if let Some(index) = index {
            self.bids.remove(index);
            return order.price;
        }
        return 0;
    }

    fn cancel_ask(&mut self, order: Order) -> u64 {
        let index = self
            .asks
            .iter()
            .position(|ask| ask.order_id == order.order_id);
        if let Some(index) = index {
            let price = self.asks[index].price;
            self.asks.remove(index);
            return price;
        }
        return 0;
    }

    fn get_open_orders(&self, user_id: String) -> Vec<Order> {
        self.bids
            .iter()
            .chain(self.asks.iter())
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect()
    }
}
