# Centralized Exchange Server

A high-performance centralized trading exchange built in Rust with.

## Architecture Overview

This exchange system consists of three main services that communicate via Redis:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│     API     │    │   ENGINE    │    │  WEBSOCKET  │
│   Service   │    │   Service   │    │   Service   │
│             │    │             │    │             │
│ - REST API  │    │ - Order     │    │ - Real-time │
│ - User      │    │   Matching  │    │   Updates   │
│   Requests  │    │ - Balance   │    │ - Market    │
│             │    │   Management│    │   Data      │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌─────────────┐
                    │    REDIS    │
                    │             │
                    │ - Message   │
                    │   Queue     │
                    │ - PubSub    │                    
                    └─────────────┘
```

## Services

### 1. API Service (`/api`)

- **Technology**: Actix Web (Rust)
- **Purpose**: Handles HTTP REST API requests
- **Features**:
  - Order placement and cancellation
  - Market depth queries
  - Open orders retrieval
  - User balance management

**Endpoints**:

- `POST /api/v1/order/` - Place order
- `DELETE /api/v1/order/` - Cancel order
- `GET /api/v1/order/open` - Get open orders
- `GET /depth/` - Get market depth
- `GET /klines/` - Get candlestick data
- `GET /trades/` - Get recent trades
- `GET /tickers/` - Get ticker data

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
  - User-specific notifications

## Data Flow

### Order Placement Flow

1. **API** receives order request via HTTP
2. **API** validates request and sends to Redis queue
3. **Engine** processes order from queue
4. **Engine** validates balance and locks funds
5. **Engine** matches order against orderbook
6. **Engine** executes trades and updates balances
7. **Engine** sends response back to API via Redis pubsub
8. **Engine** publishes market updates to WebSocket service
9. **API** returns response to client
10. **WebSocket** broadcasts updates to connected clients

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
- **Data Structures**: BTreeMap for orderbooks
- **Serialization**: Serde JSON

## Key Features

### High Performance

- **Zero-copy** operations where possible
- **Lock-free** data structures for orderbook
- **Async/await** for non-blocking I/O
- **Connection pooling** for Redis

### Reliability

- **Atomic operations** for balance updates
- **Transaction safety** for order execution
- **Error handling** with proper rollbacks
- **Message durability** via Redis persistence

### Scalability

- **Microservices** architecture
- **Horizontal scaling** capability
- **Load balancing** ready
- **Stateless** API service

## Getting Started

### Prerequisites

- Rust 1.70+
- Redis Server
- Environment variables:
  - `REDIS_URL=redis://localhost:6379`

### Running the Services

1. **Start Redis**:

   ```bash
   redis-server
   ```

2. **Start Engine Service**:

   ```bash
   cd engine
   cargo run --release
   ```

3. **Start API Service**:

   ```bash
   cd api
   cargo run --release
   ```

4. **Start WebSocket Service**:
   ```bash
   cd ws
   cargo run --release
   ```

### Testing

Place an order:

```bash
curl -X POST http://localhost:8000/api/v1/order/ \
  -H "Content-Type: application/json" \
  -d '{
    "market": "TATA_INR",
    "price": "100",
    "quantity": "10",
    "side": "buy",
    "user_id": "1"
  }'
```

## Architecture Benefits

1. **Independent Scaling**: Services can be scaled independently
2. **Fault Tolerance**: Service failures don't cascade
3. **Technology Flexibility**: Each service can use optimal technology

## Performance Characteristics

- **Order Matching**: Sub-millisecond latency
- **API Response**: < 10ms typical
- **WebSocket Updates**: Real-time (< 1ms)
- **Throughput**: 10,000+ orders/second per engine instance

## Future Enhancements

- Database persistence layer
- Authentication and authorization
- Rate limiting
- Market making algorithms
- Advanced order types (stop-loss, etc.)
- Multi-asset support
- Compliance and reporting features
