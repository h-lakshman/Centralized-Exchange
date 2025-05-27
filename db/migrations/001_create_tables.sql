CREATE TABLE IF NOT EXISTS trades (
    id TEXT PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    market TEXT NOT NULL,
    price DECIMAL NOT NULL,
    quantity DECIMAL NOT NULL,
    quote_quantity DECIMAL NOT NULL,
    is_buyer_maker BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS orders (
    order_id TEXT PRIMARY KEY,
    executed_quantity DECIMAL NOT NULL,
    price DECIMAL,
    market TEXT,
    quantity DECIMAL,
    side TEXT,
    updated_at TIMESTAMPTZ NOT NULL
);


SELECT create_hypertable('trades', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('orders', 'updated_at', if_not_exists => TRUE);

CREATE UNIQUE INDEX idx_trades_id ON trades (id, timestamp);
CREATE UNIQUE INDEX idx_orders_order_id ON orders (order_id, updated_at);