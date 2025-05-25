use std::cmp::min;
use std::collections::{BTreeMap, HashMap};

use engine::types::{DepthPayload, InternalFill, Order, Side};

use super::BASE_CURRENCY;

pub struct OrderbookFill {
    pub fill: InternalFill,
    pub other_user_id: String,
    pub marker_order_id: String,
}

pub struct OrderCreated {
    pub executed_quantity: u64,
    pub fills: Vec<OrderbookFill>,
}

pub struct Orderbook {
    pub bids: BTreeMap<u64, Vec<Order>>, // Price -> Orders at that price
    pub asks: BTreeMap<u64, Vec<Order>>, // Price -> Orders at that price
    pub base_asset: String,
    pub quote_asset: String,
    pub last_trade_id: u64,
    pub current_price: u64,
    // Cache for depth
    pub bids_depth: HashMap<u64, u64>, // Price -> Total quantity
    pub asks_depth: HashMap<u64, u64>, // Price -> Total quantity
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
        let mut orderbook = Orderbook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            base_asset,
            quote_asset: BASE_CURRENCY.to_string(),
            last_trade_id: last_trade_id.unwrap_or(0),
            current_price: current_price.unwrap_or(0),
            bids_depth: HashMap::new(),
            asks_depth: HashMap::new(),
        };

        for bid in bids {
            orderbook.add_bid_to_level(bid);
        }
        for ask in asks {
            orderbook.add_ask_to_level(ask);
        }

        orderbook
    }

    fn add_bid_to_level(&mut self, order: Order) {
        let price = order.price;
        let quantity = order.quantity - order.filled;

        *self.bids_depth.entry(price).or_insert(0) += quantity;

        self.bids.entry(price).or_insert_with(Vec::new).push(order);
    }

    fn add_ask_to_level(&mut self, order: Order) {
        let price = order.price;
        let quantity = order.quantity - order.filled;

        *self.asks_depth.entry(price).or_insert(0) += quantity;

        self.asks.entry(price).or_insert_with(Vec::new).push(order);
    }

    fn remove_from_bids_depth(&mut self, price: u64, quantity: u64) {
        if let Some(total_qty) = self.bids_depth.get_mut(&price) {
            *total_qty = total_qty.saturating_sub(quantity);
            if *total_qty == 0 {
                self.bids_depth.remove(&price);
                self.bids.remove(&price);
            }
        }
    }

    fn remove_from_asks_depth(&mut self, price: u64, quantity: u64) {
        if let Some(total_qty) = self.asks_depth.get_mut(&price) {
            *total_qty = total_qty.saturating_sub(quantity);
            if *total_qty == 0 {
                self.asks_depth.remove(&price);
                self.asks.remove(&price);
            }
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
                self.add_bid_to_level(order.clone());
                return ongoing_order;
            }
            Side::Sell => {
                let ongoing_order = self.match_bids(order);
                order.filled = ongoing_order.executed_quantity;
                if ongoing_order.executed_quantity == order.quantity {
                    return ongoing_order;
                }
                self.add_ask_to_level(order.clone());
                return ongoing_order;
            }
        }
    }

    fn match_asks(&mut self, order: &mut Order) -> OrderCreated {
        let mut fills: Vec<OrderbookFill> = Vec::new();
        let mut executed_quantity: u64 = 0;

        let mut prices_to_remove = Vec::new();

        for (&ask_price, ask_orders) in self.asks.iter_mut() {
            if ask_price > order.price || executed_quantity >= order.quantity {
                break;
            }

            let mut orders_to_remove = Vec::new();

            for (i, ask) in ask_orders.iter_mut().enumerate() {
                if executed_quantity >= order.quantity {
                    break;
                }

                let remaining_ask_qty = ask.quantity - ask.filled;
                let filled_qty = min(remaining_ask_qty, order.quantity - executed_quantity);

                executed_quantity += filled_qty;
                ask.filled += filled_qty;
                self.last_trade_id += 1;

                if let Some(depth_qty) = self.asks_depth.get_mut(&ask_price) {
                    *depth_qty = depth_qty.saturating_sub(filled_qty);
                }

                fills.push(OrderbookFill {
                    fill: InternalFill::new(ask.price, filled_qty, self.last_trade_id),
                    other_user_id: ask.user_id.clone(),
                    marker_order_id: order.order_id.clone(),
                });

                if ask.filled >= ask.quantity {
                    orders_to_remove.push(i);
                }
            }

            for &i in orders_to_remove.iter().rev() {
                ask_orders.remove(i);
            }

            if ask_orders.is_empty() {
                prices_to_remove.push(ask_price);
            }
        }

        for price in prices_to_remove {
            self.asks.remove(&price);
            self.asks_depth.remove(&price);
        }

        OrderCreated {
            fills,
            executed_quantity,
        }
    }

    fn match_bids(&mut self, order: &mut Order) -> OrderCreated {
        let mut fills: Vec<OrderbookFill> = Vec::new();
        let mut executed_qty: u64 = 0;

        let mut prices_to_remove = Vec::new();

        for (&bid_price, bid_orders) in self.bids.iter_mut().rev() {
            if bid_price < order.price || executed_qty >= order.quantity {
                break;
            }

            let mut orders_to_remove = Vec::new();

            for (i, bid) in bid_orders.iter_mut().enumerate() {
                if executed_qty >= order.quantity {
                    break;
                }

                let remaining_bid_qty = bid.quantity - bid.filled;
                let amount_remaining = min(remaining_bid_qty, order.quantity - executed_qty);

                executed_qty += amount_remaining;
                bid.filled += amount_remaining;
                self.last_trade_id += 1;

                if let Some(depth_qty) = self.bids_depth.get_mut(&bid_price) {
                    *depth_qty = depth_qty.saturating_sub(amount_remaining);
                }

                fills.push(OrderbookFill {
                    fill: InternalFill::new(bid.price, amount_remaining, self.last_trade_id),
                    other_user_id: bid.user_id.clone(),
                    marker_order_id: bid.order_id.clone(),
                });

                if bid.filled >= bid.quantity {
                    orders_to_remove.push(i);
                }
            }

            for &i in orders_to_remove.iter().rev() {
                bid_orders.remove(i);
            }

            if bid_orders.is_empty() {
                prices_to_remove.push(bid_price);
            }
        }

        for price in prices_to_remove {
            self.bids.remove(&price);
            self.bids_depth.remove(&price);
        }

        OrderCreated {
            fills,
            executed_quantity: executed_qty,
        }
    }

    //retrives depth from cache
    pub fn get_depth(&self) -> DepthPayload {
        let mut bids: Vec<[String; 2]> = self
            .bids_depth
            .iter()
            .filter(|(_, &qty)| qty > 0)
            .map(|(&price, &qty)| [price.to_string(), qty.to_string()])
            .collect();

        let mut asks: Vec<[String; 2]> = self
            .asks_depth
            .iter()
            .filter(|(_, &qty)| qty > 0)
            .map(|(&price, &qty)| [price.to_string(), qty.to_string()])
            .collect();

        bids.sort_by(|a, b| {
            let price_a: u64 = a[0].parse().unwrap_or(0);
            let price_b: u64 = b[0].parse().unwrap_or(0);
            price_b.cmp(&price_a)
        });

        asks.sort_by(|a, b| {
            let price_a: u64 = a[0].parse().unwrap_or(0);
            let price_b: u64 = b[0].parse().unwrap_or(0);
            price_a.cmp(&price_b)
        });

        DepthPayload { bids, asks }
    }

    pub fn cancel_bid(&mut self, order: &Order) -> Option<u64> {
        let price = order.price;
        let index = self
            .bids
            .get_mut(&price)
            .and_then(|bids| bids.iter().position(|bid| bid.order_id == order.order_id));
        if let Some(index) = index {
            let price: u64 = price;
            let order = self.bids.get_mut(&price).unwrap().remove(index);
            self.remove_from_bids_depth(price, order.quantity - order.filled);
            return Some(price);
        }
        return None;
    }

    pub fn cancel_ask(&mut self, order: &Order) -> Option<u64> {
        let price = order.price;
        let index = self
            .asks
            .get_mut(&price)
            .and_then(|asks| asks.iter().position(|ask| ask.order_id == order.order_id));
        if let Some(index) = index {
            let price: u64 = price;
            let order = self.asks.get_mut(&price).unwrap().remove(index);
            self.remove_from_asks_depth(price, order.quantity - order.filled);
            return Some(price);
        }
        return None;
    }

    pub fn get_open_orders(&self, user_id: String) -> Vec<Order> {
        let mut open_orders = Vec::new();

        for (_, orders) in &self.bids {
            for order in orders {
                if order.user_id == user_id {
                    open_orders.push(order.clone());
                }
            }
        }

        for (_, orders) in &self.asks {
            for order in orders {
                if order.user_id == user_id {
                    open_orders.push(order.clone());
                }
            }
        }

        open_orders
    }
}
