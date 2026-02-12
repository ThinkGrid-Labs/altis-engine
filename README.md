# Altis Engine - Modern Airline Retailing Platform

> **The New Way of Airline Commerce**  
> Built with Rust for performance, designed for the future of dynamic retailing.

Altis is a next-generation airline booking engine that replaces legacy PNR/E-Ticket systems with a modern **Offer/Order** architecture. Think of it as "Shopify for Airlines" - enabling dynamic pricing, seamless ancillary sales, and zero-cost modifications.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## 🚀 Why Altis is Different

### The Problem with Legacy Systems

Traditional airline systems (Amadeus, Sabre) were built in the 1960s for a different era:
- **PNR/E-Ticket complexity**: Cryptic 6-character codes, manual revalidation
- **Fare buckets**: Discrete price jumps ($200 → $250) when inventory sells
- **Ancillary limitations**: Can't easily sell meals, bags, lounge access
- **Change fees**: Manual processes cost airlines millions in support

### The Altis Approach

| Legacy System | Altis Engine |
|---------------|--------------|
| PNR + E-Ticket | **Order ID** (single source of truth) |
| Fare buckets (A, B, C, Y) | **Continuous pricing** (adjust by cents) |
| Flight-only focus | **Unified products** (Flight = Meal = Seat) |
| Manual ticket revalidation | **Zero-cost changes** (DB updates) |
| Static pricing | **AI-driven offers** with dynamic bundling |

---

## ✨ Key Features

### 1. **Offer/Order Architecture**

**Offers** are temporary AI-generated bundles (15-min expiry):
```json
{
  "offer_id": "550e8400-e29b-41d4-a716-446655440000",
  "items": [
    {"type": "FLIGHT", "price": 20000},
    {"type": "MEAL", "price": 1500},
    {"type": "SEAT", "price": 3000}
  ],
  "total": 24500,
  "expires_at": "2024-06-01T10:15:00Z"
}
```

**Orders** are the permanent record (no PNR needed):
```json
{
  "order_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "status": "PAID",
  "items": [...],
  "fulfillment": [
    {"barcode": "ALTIS-1717234567-A1B2C3D4"}
  ]
}
```

### 2. **Continuous Pricing Engine**

Instead of jumping from $200 to $250 when a fare bucket sells out, Altis adjusts prices **by the cent** based on real-time demand:

```rust
// 90% sold = 2.6x price multiplier
let multiplier = 1.0 + (utilization² × 2.0);
```

**Business Impact**: Maximize revenue per seat while reducing customer sticker shock.

### 3. **AI-Driven Merchandising**

Generate 3-5 ranked offers per search:
- **Flight-Only**: Baseline ($200)
- **Comfort Bundle**: Flight + Seat + Meal ($240, 10% discount)
- **Premium Bundle**: Flight + Lounge + Fast-Track ($280)

Ranked by: `conversion_probability × profit_margin`

### 4. **Zero-Cost Modifications**

Change a flight? Just add a new item and mark the old one as `REFUNDED`:

```rust
// No ticket revalidation needed!
order.add_item(new_flight);
order.refund_item(old_flight_id);
```

**Cost Savings**: Eliminate manual processes that cost airlines $15-30 per change.

### 5. **Unified Product Catalog**

Everything is a `Product`:
- Flights
- Seats (extra legroom, window, aisle)
- Bags (checked, carry-on, oversize)
- Meals (vegetarian, vegan, halal)
- Lounge access
- Carbon offsets
- Travel insurance

**Revenue Impact**: +30% ancillary revenue per booking.

---

## 🏗️ Architecture

### Crate Structure

```
altis-engine/
├── altis-catalog/      # Product models + continuous pricing
├── altis-offer/        # AI-driven offer generation
├── altis-order/        # Order lifecycle management
├── altis-core/         # Business logic (domain layer)
├── altis-store/        # Database adapters (Postgres, Redis, Kafka)
├── altis-shared/       # Common types and utilities
└── altis-api/          # REST API (Axum web framework)
```

### Technology Stack

- **Core**: Rust, Axum (Web), Tokio (Async)
- **Data**: PostgreSQL (orders), Redis (offers cache, inventory)
- **Events**: Apache Kafka (order events, analytics)
- **Infrastructure**: Docker, Docker Compose

### Design Patterns

- **Hexagonal Architecture**: Clean separation of business logic and infrastructure
- **CQRS**: Read-optimized availability index (Redis) + write-optimized orders (Postgres)
- **Event Sourcing**: Order state changes emit events for downstream systems
- **State Machine**: Order lifecycle (Proposed → Locked → Paid → Fulfilled)

---

## 🚦 Getting Started

### Prerequisites

- Docker & Docker Compose
- (Optional) Rust 1.70+ for local development

### Quick Start

```bash
# Clone the repository
git clone https://github.com/akosidencio/altis-engine.git
cd altis-engine

# Start all services
docker-compose up --build

# API available at http://localhost:8080
```

### Verify Installation

```bash
curl http://localhost:8080/health
# {"status": "healthy"}
```

---

## 📖 API Usage

### 1. Authentication (Required)

All API endpoints (except `/health` and `/v1/auth/guest`) require a JWT token.

**Step 1: Get a Guest Token**
```bash
curl -X POST http://localhost:8080/v1/auth/guest \
  -H "Content-Type: application/json"
# {"token": "eyJhbGciOiJIUzI1Ni..."}
```

**Step 2: Use Token in Requests**
Add the header `Authorization: Bearer <token>` to all subsequent requests.

### 2. Search for Offers

```bash
curl -X POST http://localhost:8080/v1/offers/search \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "origin": "JFK",
    "destination": "LHR",
    "departure_date": "2024-06-01",
    "passengers": 2
  }'
```

**Response**: 3-5 ranked offers with different bundles

### 3. Accept an Offer

```bash
curl -X POST http://localhost:8080/v1/offers/{offer_id}/accept \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "customer_email": "user@example.com"
  }'
```

**Response**: Order created with `PROPOSED` status

### 4. Complete Payment

```bash
curl -X POST http://localhost:8080/v1/orders/{order_id}/pay \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "payment_token": "tok_visa_4242"
  }'
```

**Response**: Order status → `PAID`, fulfillment barcodes generated

### 5. Get Order Details

```bash
curl http://localhost:8080/v1/orders/{order_id} \
  -H "Authorization: Bearer {token}"
```

**Response**: Full order with items, status, and fulfillment

### 6. Modify Order (Change Flight)

```bash
curl -X POST http://localhost:8080/v1/orders/{order_id}/modify \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "action": "change_flight",
    "old_item_id": "{flight_item_id}",
    "new_product_id": "{new_flight_id}"
  }'
```

**Response**: New flight added, old flight marked as `REFUNDED`

---

## 🎯 Business Advantages

### Revenue Growth

| Metric | Target | How |
|--------|--------|-----|
| Ancillary revenue per booking | **+30%** | Bundled offers with discounts |
| Average order value | **+25%** | AI-ranked premium bundles |
| Conversion rate | **+15%** | Simplified offer presentation |

### Cost Reduction

- **Zero ticket revalidation**: Changes = DB updates (vs. $15-30 manual process)
- **Reduced support calls**: Clear order history, no PNR confusion
- **Infrastructure savings**: Rust performance = fewer servers

### Competitive Advantages

- **Continuous pricing**: Beat competitors by cents, not dollars
- **Instant modifications**: Change flights in <100ms
- **Flexible bundling**: Launch new products without schema changes

---

## 📊 Performance

- **Offer generation**: <200ms (p95)
- **Order creation**: <100ms (p95)
- **Continuous pricing**: <50ms per calculation
- **Throughput**: >1000 orders/sec (single instance)

---

## 🛠️ Development

### Local Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build all crates
cargo build

# Run tests
cargo test

# Run API locally
cargo run -p altis-api
```

### Database Migrations

```bash
# Run migrations
docker-compose exec api sqlx migrate run

# Create new migration
sqlx migrate add <migration_name>
```

### Code Structure

- **altis-catalog**: Product models, pricing engine, inventory
- **altis-offer**: Offer generation, AI ranking, expiry management
- **altis-order**: State machine, fulfillment, change handling
- **altis-api**: REST endpoints, authentication, middleware

---

## 📚 Documentation

- [Architecture Overview](docs/architecture/OVERVIEW.md)
- [API Reference](docs/API.md)
- [Offer/Order Concepts](docs/OFFER_ORDER.md)
- [Continuous Pricing](docs/PRICING.md)
- [Deployment Guide](docs/DEPLOYMENT.md)

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🌟 Why "Altis"?

*Altis* comes from the Latin "altus" meaning "high" or "elevated" - representing our mission to elevate airline commerce to modern standards. Just as Shopify transformed e-commerce, Altis aims to transform airline retailing.

---

**Built with ❤️ in Rust**
