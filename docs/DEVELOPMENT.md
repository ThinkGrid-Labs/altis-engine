# Technical Development Guide

This guide is for developers contributing to the Altis Engine or running it locally for testing.

## 🚦 Getting Started

### Prerequisites
- **Docker**: For running the ecosystem (Postgres, Redis, Kafka).
- **Rust (1.84+)**: For compiling and running the engine.
- **sqlx-cli**: For database migrations.

### Environment Setup
1. Copy the example environment file:
   `cp .env.example .env`
2. Update `.env` with your local configurations.

### Running with Docker (Recommended)
```bash
docker-compose up --build
```

### Manual Development
1. Start infrastructure:
   `docker-compose up -d postgres redis kafka zookeeper`
2. Run migrations:
   `cargo sqlx migrate run`
3. Start the API:
   `cargo run -p altis-api`

---

## 🧪 Testing
Run the workspace test suite:
```bash
cargo test --workspace
```

## 🏗️ Workspace Structure
- `altis-core`: Domain models and IATA traits.
- `altis-api`: Axum-based REST and NDC/ONE Order endpoints.
- `altis-offer`: Merchandising and AI ranking logic.
- `altis-order`: Lifecycle management and fulfillment.
- `altis-store`: Persistence layer (SQL/Redis/Kafka).

---

## 🛡️ Security Architecture

### Authentication Implementation
- **JWT**: Stateless session management using HMAC-SHA256.
- **Guest Tokens**: Temporary authorization for search and offer exploration.
- **Validation**: Strict audience and expiration checks on every request.

### Privacy & Data Protection
- **PII Encryption**: Any stored personal data is hashed or encrypted.
- **One Identity**: Uses the `OneIdResolver` trait to verify traveler credentials via W3C standards.
- **Logging**: Sensitive data (tokens, PII) is automatically filtered from system logs.

### Resiliency (DoS Protection)
- **Circuit Breakers**: Protects against slow-loris or cascading failures via the `resiliency` middleware.
- **Metrics**: Real-time monitoring of 4xx/5xx rates for anomaly detection.
