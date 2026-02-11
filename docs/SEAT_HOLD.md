# Seat Hold & Inventory Management

## Overview

When customers customize their order with specific seat selections, those seats are **temporarily held** to prevent double-booking while the customer completes payment.

---

## Hold Mechanism

### 1. Seat Selection (Customization)

**API Call**:
```bash
POST /v1/orders/{order_id}/customize
{
  "seat_preferences": [
    {"passenger": 1, "seat": "12A", "flight_id": "flight-aa100"},
    {"passenger": 2, "seat": "12B", "flight_id": "flight-aa100"}
  ],
  "meal_preferences": [
    {"passenger": 1, "meal": "VEGETARIAN"},
    {"passenger": 2, "meal": "CHICKEN"}
  ]
}
```

**Backend Process**:

```rust
async fn customize_order(
    order_id: Uuid,
    customization: CustomizationRequest,
) -> Result<Order, OrderError> {
    // 1. Validate order is in PROPOSED status
    let order = get_order(order_id)?;
    if order.status != OrderStatus::Proposed {
        return Err(OrderError::InvalidStatus);
    }
    
    // 2. Hold seats in Redis (15-minute TTL)
    for seat_pref in customization.seat_preferences {
        let seat_key = format!("seat:{}:{}", seat_pref.flight_id, seat_pref.seat);
        
        // Check if seat is available
        let existing_hold: Option<String> = redis.get(&seat_key).await?;
        if let Some(holder) = existing_hold {
            if holder != order_id.to_string() {
                return Err(OrderError::SeatUnavailable(seat_pref.seat));
            }
        }
        
        // Hold seat with 15-minute expiry
        redis.set_ex(&seat_key, order_id.to_string(), 900).await?;
    }
    
    // 3. Update order metadata
    update_order_customization(order_id, customization).await?;
    
    // 4. Extend order expiry to 15 minutes from now
    let expires_at = Utc::now() + Duration::minutes(15);
    update_order_expiry(order_id, expires_at).await?;
    
    Ok(order)
}
```

### 2. Payment (Lock Seats Permanently)

**API Call**:
```bash
POST /v1/orders/{order_id}/pay
{
  "payment_method": "card",
  "payment_token": "tok_visa_4242"
}
```

**Backend Process**:

```rust
async fn pay_order(
    order_id: Uuid,
    payment: PaymentRequest,
) -> Result<Order, OrderError> {
    // 1. Process payment
    let payment_result = payment_gateway.charge(payment).await?;
    
    // 2. Lock seats permanently (remove TTL)
    let order = get_order(order_id)?;
    for item in &order.items {
        if item.product_type == ProductType::Seat {
            if let Some(seat_assignments) = &item.metadata.seat_assignments {
                for assignment in seat_assignments {
                    let seat_key = format!("seat:{}:{}", assignment.flight_id, assignment.seat);
                    
                    // Remove TTL (make permanent)
                    redis.persist(&seat_key).await?;
                    
                    // Also write to database for durability
                    db.execute(
                        "INSERT INTO seat_assignments (order_id, flight_id, seat_number, passenger_index)
                         VALUES ($1, $2, $3, $4)",
                        &[&order_id, &assignment.flight_id, &assignment.seat, &assignment.passenger_index]
                    ).await?;
                }
            }
        }
    }
    
    // 3. Update order status to PAID
    update_order_status(order_id, OrderStatus::Paid).await?;
    
    Ok(order)
}
```

### 3. Expiry (Release Seats)

If customer doesn't pay within 15 minutes, Redis keys expire automatically:

```rust
// Redis automatically releases seats after TTL expires
// No manual cleanup needed!

// Optional: Background job to clean up expired orders
async fn cleanup_expired_orders() {
    let expired_orders = db.query(
        "SELECT id FROM orders 
         WHERE status = 'PROPOSED' 
         AND expires_at < NOW()"
    ).await?;
    
    for order in expired_orders {
        // Mark order as EXPIRED
        update_order_status(order.id, OrderStatus::Expired).await?;
        
        // Seats already released by Redis TTL
        tracing::info!("Order {} expired, seats released", order.id);
    }
}
```

---

## Seat Availability Check

### Real-Time Seat Map

When customer opens seat selection, show real-time availability:

```bash
GET /v1/flights/{flight_id}/seats
```

**Response**:
```json
{
  "flight_id": "flight-aa100",
  "aircraft": "Boeing 737-800",
  "seat_map": [
    {
      "row": 12,
      "seats": [
        {"number": "12A", "type": "EXTRA_LEGROOM", "status": "AVAILABLE", "price": 1800},
        {"number": "12B", "type": "EXTRA_LEGROOM", "status": "HELD", "price": 1800},
        {"number": "12C", "type": "EXTRA_LEGROOM", "status": "AVAILABLE", "price": 1800},
        {"number": "12D", "type": "EXTRA_LEGROOM", "status": "OCCUPIED", "price": null},
        {"number": "12E", "type": "EXTRA_LEGROOM", "status": "AVAILABLE", "price": 1800},
        {"number": "12F", "type": "EXTRA_LEGROOM", "status": "AVAILABLE", "price": 1800}
      ]
    }
  ]
}
```

**Seat Statuses**:
- `AVAILABLE`: Can be selected
- `HELD`: Temporarily held by another customer (show as unavailable)
- `OCCUPIED`: Permanently assigned (paid order)
- `BLOCKED`: Not available for sale (crew, broken, etc.)

**Backend Logic**:

```rust
async fn get_seat_availability(flight_id: &str) -> Result<SeatMap, Error> {
    let mut seat_map = get_aircraft_seat_map(flight_id).await?;
    
    // Check Redis for held seats
    for row in &mut seat_map.rows {
        for seat in &mut row.seats {
            let seat_key = format!("seat:{}:{}", flight_id, seat.number);
            
            // Check if seat is held or occupied
            if let Some(order_id) = redis.get::<String>(&seat_key).await? {
                seat.status = SeatStatus::Held;
            } else {
                // Check database for paid seats
                let occupied = db.query_one(
                    "SELECT 1 FROM seat_assignments 
                     WHERE flight_id = $1 AND seat_number = $2",
                    &[&flight_id, &seat.number]
                ).await;
                
                if occupied.is_some() {
                    seat.status = SeatStatus::Occupied;
                } else {
                    seat.status = SeatStatus::Available;
                }
            }
        }
    }
    
    Ok(seat_map)
}
```

---

## Edge Cases

### 1. Customer Changes Seat Selection

**Scenario**: Customer selects 12A, then changes to 14C

**API Call**:
```bash
POST /v1/orders/{order_id}/customize
{
  "seat_preferences": [
    {"passenger": 1, "seat": "14C", "flight_id": "flight-aa100"}  // Changed from 12A
  ]
}
```

**Backend Logic**:
```rust
// 1. Release old seat (12A)
redis.del("seat:flight-aa100:12A").await?;

// 2. Hold new seat (14C)
redis.set_ex("seat:flight-aa100:14C", order_id, 900).await?;

// 3. Update order metadata
```

### 2. Concurrent Seat Selection

**Scenario**: Two customers try to select the same seat simultaneously

**Solution**: Use Redis `SET NX` (set if not exists)

```rust
// Atomic operation: only succeeds if key doesn't exist
let success = redis.set_nx("seat:flight-aa100:12A", order_id).await?;

if !success {
    return Err(OrderError::SeatUnavailable("12A"));
}

// Set expiry separately
redis.expire("seat:flight-aa100:12A", 900).await?;
```

### 3. Order Expiry Extension

**Scenario**: Customer is still customizing after 10 minutes

**Solution**: Extend TTL when customer is active

```rust
// Every time customer interacts, extend TTL
async fn extend_order_hold(order_id: Uuid) -> Result<(), Error> {
    let order = get_order(order_id)?;
    
    // Extend seat holds
    for item in &order.items {
        if let Some(seat_assignments) = &item.metadata.seat_assignments {
            for assignment in seat_assignments {
                let seat_key = format!("seat:{}:{}", assignment.flight_id, assignment.seat);
                redis.expire(&seat_key, 900).await?;  // Reset to 15 minutes
            }
        }
    }
    
    Ok(())
}
```

---

## Summary

| Event | Action | Seat Status | Duration |
|-------|--------|-------------|----------|
| **Accept Offer** | Create order (PROPOSED) | No seats held yet | - |
| **Customize Seats** | Hold seats in Redis | `HELD` (visible to others as unavailable) | 15 minutes |
| **Pay Order** | Lock seats permanently | `OCCUPIED` | Permanent |
| **Expire (no payment)** | Redis TTL expires | `AVAILABLE` again | Auto-release |

**Key Benefits**:
- ✅ Prevents double-booking
- ✅ Automatic cleanup (Redis TTL)
- ✅ Real-time availability
- ✅ No manual intervention needed
- ✅ Scales horizontally (Redis cluster)

**Implementation Priority**:
1. Basic seat hold (Redis with TTL)
2. Real-time seat map API
3. Concurrent selection handling (SET NX)
4. Order expiry extension (optional)
