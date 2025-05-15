use std::cmp::min;

#[derive(Clone)]
pub struct Order {
    price: u64,
    quantity: u64,
    order_id: String,
    filled: u64,
    side: String,
    user_id: String,
}

pub struct Fills {
    price: String,
    qty: u64,
    trade_id: u64,
    other_user_id: String,
    marker_order_id: String,
}

struct OrderCreated {
    executed_quantity: u64,
    fills: Vec<Fills>,
}

enum Side {
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

    fn ticker(&self) -> String {
        format!("{}_{}", self.base_asset, self.quote_asset)
    }
    fn add_order(&mut self, order: &mut Order) -> OrderCreated {
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
        let mut fills: Vec<Fills> = Vec::new();
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

                fills.push(Fills {
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
        let mut fills: Vec<Fills> = Vec::new();
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

                fills.push(Fills {
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
}
