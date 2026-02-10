# Altis - Enterprise Airline Booking Engine

Altis is a high-performance, event-driven airline booking engine built with Rust. 
It features a Hexagonal Architecture, Real-Time Availability Index (CQRS), and a Dual-Stage Hold Engine.

## Technology Stack
- **Core**: Rust, Warp (Web Framework), Tokio (Async Runtime)
- **Data**: PostgreSQL (Source of Truth), Redis (Holds & Cache)
- **Events**: Apache Kafka (Event Streaming)
- **Infrastructure**: Docker, Docker Compose

## Architecture Highlights
- **CQRS**: Search reads from Redis (sub-ms), Bookings write to Postgres.
- **Flexible Pricing Engine**: Time-limited sales, percentage discounts, and fixed surcharges configurable via `default.toml`.
- **Stateful Sessions**: Redis Hashes store Trip state (Passengers, Ancillaries) incrementally.
- **Atomic Availability**: Redis `DECR` operations ensure consistency under high concurrency.
- **Dual-Stage Holds**: 
  1. `Trip Hold`: Reserves the itinerary price/context (15 mins).
  2. `Seat Hold`: Locks specific seats (Atomic Redis Lock).
- **Security**: "Anonymous-First" JWT Authentication.

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
