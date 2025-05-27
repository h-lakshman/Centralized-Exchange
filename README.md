# Centralized Exchange Server

A high-performance centralized trading exchange built in Rust.

## Architecture Overview

This exchange system consists of four main services that communicate via Redis:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│     API     │    │   ENGINE    │    │  WEBSOCKET  │    │  DATABASE   │
│   Service   │    │   Service   │    │   Service   │    │   Service   │
│             │    │             │    │             │    │             │
│ - REST API  │    │ - Order     │    │ - Real-time │    │ - Trade     │
│ - User      │    │   Matching  │    │   Updates   │    │   Storage   │
│   Requests  │    │ - Balance   │    │ - Market    │    │ - Order     │
│ - Market    │    │   Management│    │   Data      │    │   History   │
│   Data      │    │             │    │             │    │ - Analytics │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                                                           │
       └───────────────────┼───────────────────┼───────────────────┘
                           │                   │
                    ┌─────────────┐    ┌─────────────┐
                    │    REDIS    │    │ POSTGRESQL  │
                    │             │    │             │
                    │ - Message   │    │ -TimescaleDB│
                    │   Queue     │    │ -Trade Data │
                    │ - PubSub    │    │ -Historical │
                    │             │    │  Records    │
                    └─────────────┘    └─────────────┘
```

## Services

### 1. API Service (`/api`)

- **Technology**: Actix Web (Rust)
- **Purpose**: Handles HTTP REST API requests
- **Features**:
  - Order placement and cancellation
  - Market depth queries
  - Open orders retrieval
  - Recent Trade retrieval

**Endpoints**:

- `POST /api/v1/order/` - Place order
- `DELETE /api/v1/order/` - Cancel order
- `GET /api/v1/order/open` - Get open orders
- `GET /api/v1/depth/` - Get market depth
- `GET /api/v1/klines/` - Get candlestick data
- `GET /api/v1/trades/` - Get recent trades
- `GET /api/v1/tickers/` - Get ticker data

### 2. Engine Service (`/engine`)

- **Technology**: Tokio (Rust)
- **Purpose**: Core trading engine and order matching
- **Features**:
  - Order book management
  - Order matching algorithm
  - Balance validation and locking
  - Trade execution
  - Market data generation

**Key Components**:

- **Orderbook**: BTreeMap-based order matching.
- **Balance Manager**: Handles user fund locking/unlocking
- **Trade Engine**: Executes matched orders and updates balances

### 3. WebSocket Service (`/ws`)

- **Technology**: Tokio WebSockets (Rust)
- **Purpose**: Real-time market data streaming
- **Features**:
  - Live order book updates
  - Trade stream
  - Ticker updates

### 4. Database Service (`/db`)

- **Technology**: PostgreSQL with TimescaleDB (Rust + sqlx)
- **Purpose**: Persistent storage and historical data management
- **Features**:
  - Trade data persistence
  - Order history tracking
  - Time-series data optimization
  - OHLCV candlestick generation

**Database Schema**:

- **Trades Table**: Stores all executed trades with precise decimal values
- **Orders Table**: Tracks order states and execution history
- **TimescaleDB Hypertables**: Optimized for time-series queries
- **Indexes**: Efficient querying by market, timestamp, and order ID

## Data Flow

### Order Placement Flow

1. **API** receives order request via HTTP
2. **API** validates request and sends to Redis queue
3. **Engine** processes order from queue
4. **Engine** validates balance and locks funds
5. **Engine** matches order against orderbook
6. **Engine** executes trades and updates balances
7. **Engine** sends trade/order data to Database via Redis queue
8. **Database** persists trade and order updates to PostgreSQL
9. **Engine** sends response back to API via Redis pubsub
10. **Engine** publishes market updates to WebSocket service
11. **API** returns response to client
12. **WebSocket** broadcasts updates to connected clients

### Database Persistence Flow

1. **Engine** creates trade records after order matching
2. **Engine** sends trade data to Redis `db_processor` queue
3. **Database** service consumes messages from queue
4. **Database** stores trades with DECIMAL precision in TimescaleDB
5. **Database** updates order execution status with UPSERT logic
6. **API** queries database for historical data (trades, klines)
7. **TimescaleDB** provides optimized time-series aggregations

### Real-time Updates Flow

1. **Engine** generates market data (trades, depth changes)
2. **Engine** publishes to Redis channels
3. **WebSocket** service subscribes to channels
4. **WebSocket** broadcasts to connected clients

## Technology Stack

- **Language**: Rust
- **Web Framework**: Actix Web
- **Async Runtime**: Tokio
- **Message Broker**: Redis
- **Database**: PostgreSQL with TimescaleDB extension
- **Database Driver**: sqlx with async support
- **Data Structures**: BTreeMap for orderbooks
- **Precision**: rust_decimal for financial calculations
- **Serialization**: Serde JSON

## Getting Started

### Prerequisites

- Rust 1.70+
- Redis Server
- PostgreSQL with TimescaleDB extension
- Environment variables:
  - `REDIS_URL=redis://localhost:6379`
  - `DATABASE_URL=postgresql://username:password@localhost:5432/exchange_db`

### Running the Services

1. **Start Redis**:

   ```bash
   redis-server
   ```

2. **Start PostgreSQL with TimescaleDB**:

   ```bash
   # Install TimescaleDB extension
   sudo apt install timescaledb-postgresql-14
   # Or using Docker
   docker run -d --name timescaledb -p 5432:5432 -e POSTGRES_PASSWORD=password timescale/timescaledb:latest-pg14
   ```

3. **Setup Database**:

   ```bash
   cd db
   # Run migration to create tables
   sqlx migrate run
   ```

4. **Start Database Service**:

   ```bash
   cd db
   cargo run --release
   ```

5. **Start Engine Service**:

   ```bash
   cd engine
   cargo run --release
   ```

6. **Start API Service**:

   ```bash
   cd api
   cargo run --release
   ```

7. **Start WebSocket Service**:
   ```bash
   cd ws
   cargo run --release
   ```

### Testing

Place an order:

```bash
# Place an order
curl -X POST http://localhost:8000/api/v1/order/ \
  -H "Content-Type: application/json" \
  -d '{
    "market": "TATA_INR",
    "price": "100",
    "quantity": "10",
    "side": "buy",
    "user_id": "1"
  }'

# Get recent trades
curl "http://localhost:8000/api/v1/trades?symbol=TATA_INR&limit=10"

# Get candlestick data
curl "http://localhost:8000/api/v1/klines?market=TATA_INR&interval=1h&start_time=2024-01-01T00:00:00Z"
```

## Architecture Benefits

1. **Independent Scaling**: Services can be scaled independently
2. **Fault Tolerance**: Service failures don't cascade
3. **Technology Flexibility**: Each service can use optimal technology

## Performance Characteristics

- **Order Matching**: Sub-millisecond latency
- **API Response**: < 10ms typical
- **WebSocket Updates**: Real-time (< 1ms)

## Database Features

### TimescaleDB Optimizations

- **Hypertables**: Automatic partitioning by time for trades and orders
- **Time-bucket aggregations**: Efficient OHLCV candlestick generation

## Future Enhancements

- Authentication and authorization
- Rate limiting
- Market making algorithms
- Advanced order types (stop-loss, etc.)
- Multi-asset support
- Compliance and reporting features
- Database read replicas for analytics
- Data archival and backup strategies
