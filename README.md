# Altis - Enterprise Airline Booking Engine

Altis is a high-performance, event-driven airline booking engine built with Rust. 
It features a Hexagonal Architecture, Real-Time Availability Index (CQRS), and a Dual-Stage Hold Engine.

## Technology Stack
- **Core**: Rust, Axum (Web Framework), Tokio (Async Runtime)
- **Middleware**: Tower (Standardized Middleware)
- **Data**: PostgreSQL (Source of Truth), Redis (Holds & Cache)
- **Events**: Apache Kafka (Event Streaming)
- **Infrastructure**: Docker, Docker Compose

## Architecture & Scalability Analysis
**Rating: Enterprise-Grade**

Altis is designed as an **Event-Driven Hexagonal Architecture**, prioritizing scalability and maintainability.

### 1. Modularity (Clean Architecture)
- **Domain Layer (`altis-domain`)**: Pure business logic with zero dependencies. Defines **Traits (Ports)** for infrastructure.
- **Infrastructure Layer (`altis-infra`)**: Pluggable adapters (`PostgresFlightRepository`) implementing Domain Traits.
- **API Layer (`altis-api`)**: Stateless HTTP adapter (Axum) leveraging **Dependency Injection** (Traits) and **Typed Configuration**.

### 2. Scalability (CQRS & Async)
- **Reads**: 99% of traffic (Search) hits Redis (Availability Index) for sub-millisecond response times.
- **Writes**: Booking commits rely on ACID transactions in PostgreSQL.
- **Concurrency**: Built on **Tokio**, allowing handling of thousands of concurrent connections (e.g., SSE streams) with minimal resource usage.

### 3. Design Patterns
- **Dual-Stage Hold**: Separates logical trip sessions from physical seat locks (Mutex).
- **Distributed Rate Limiting**: Redis-backed sliding window algo for global API protection.
- **Middleware Chain**: standardized Auth, CORS, and Tracing via `Tower`.

## Getting Started

### Prerequisites
- Docker & Docker Compose

### Run the Stack
```bash
docker-compose up --build
```
The API will be available at `http://localhost:8080`.

## API Usage Guide

### 1. Authentication
**GET Guest Token** (Required for all other endpoints)
```bash
curl -X POST http://localhost:8080/v1/auth/guest
# Response: {"token": "eyJhbGci..."}
export TOKEN=<paste_token_here>
```

### 2. Flight Search (Public)
**Find Flights**
```bash
curl -X POST http://localhost:8080/v1/flights/search \
  -H "Content-Type: application/json" \
  -d '{
    "legs": [
      { "origin_airport_code": "JFK", "destination_airport_code": "LHR", "date": "2024-06-01" }
    ],
    "passenger_count": 1
  }'
```

### 3. Create Trip (Hold Itinerary)
**Start a Trip Session**
```bash
curl -X POST http://localhost:8080/v1/holds/trip \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "flight_ids": ["<FLIGHT_UUID>"] 
  }'
# Response: {"trip_id": "<TRIP_UUID>", "expires_at": 1700000000}
```

### 3a. Add Passenger (New)
**Add Guest to Session**
```bash
curl -X POST http://localhost:8080/v1/holds/trip/<TRIP_UUID>/passengers \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "first_name": "John",
    "last_name": "Doe",
    "passenger_type": "ADULT"
  }'
# Response: {"passenger_id": "<PAX_UUID>"}
```

### 4. Hold Seats
**Lock Specific Seats**
```bash
curl -X POST http://localhost:8080/v1/holds/seat \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "trip_id": "<TRIP_UUID>",
    "flight_id": "<FLIGHT_UUID>",
    "seat_number": "1A"
  }'
```

### 5. Commit Booking
**Finalize and Pay**
```bash
curl -X POST http://localhost:8080/v1/bookings/commit \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "trip_id": "<TRIP_UUID>",
    "user_email": "user@example.com",
    "payment_token": "tok_visa",
    "passengers": [
      {
        "first_name": "Jane",
        "last_name": "Doe",
        "seats": [
            {"flight_id": "<FLIGHT_UUID>", "seat_number": "1A"}
        ]
      }
    ]
  }'
# Response: {"booking_id": "...", "status": "CONFIRMED"}
```

### 6. Real-Time Updates (SSE)
**Listen for Seat Changes**
```bash
curl -N -H "Authorization: Bearer $TOKEN" http://localhost:8080/v1/flights/<FLIGHT_UUID>/stream
```

## Configuration
Configuration is managed via `config/default.toml` or Environment Variables.
- `APP_SERVER__PORT`: API Port (default: 8080)
- `APP_DATABASE__URL`: Postgres Connection String
- `APP_REDIS__URL`: Redis Connection String
- `APP_KAFKA__BROKERS`: Kafka Brokers
- `APP_AUTH__JWT_SECRET`: Secret Key for Tokens
