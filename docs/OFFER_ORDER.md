# Offer/Order Concepts

## The Modern Airline Retailing Model

Traditional airline systems use **PNR (Passenger Name Record)** and **E-Tickets**. Altis uses **Offers** and **Orders**.

---

## What is an Offer?

An **Offer** is a temporary, AI-generated bundle of products presented to a customer.

### Key Characteristics

- **Temporary**: 15-minute expiry (prevents stale pricing)
- **AI-Ranked**: Sorted by conversion probability × profit margin
- **Bundled**: Combines flights + ancillaries with discounts
- **Stateless**: Stored in Redis cache, not permanent database

### Example Offer

```json
{
  "offer_id": "550e8400-e29b-41d4-a716-446655440000",
  "expires_at": "2024-06-01T10:15:00Z",
  "total_nuc": 24500,
  "items": [
    {"type": "FLIGHT", "price": 20000},
    {"type": "MEAL", "price": 1500, "discount": 10%},
    {"type": "SEAT", "price": 3000, "discount": 10%}
  ]
}
```

### Why 15 Minutes?

- **Inventory protection**: Prevents hoarding of low prices
- **Fresh pricing**: Ensures prices reflect current demand
- **Conversion urgency**: Creates psychological pressure to book

---

## What is an Order?

An **Order** is the permanent record of a customer's purchase.

### Key Characteristics

- **Permanent**: Stored in PostgreSQL, never deleted
- **Single Source of Truth**: Replaces PNR + E-Ticket
- **Modifiable**: Add/remove items without revalidation
- **Auditable**: Full history of changes

### Order Lifecycle

```
PROPOSED → LOCKED → PAID → FULFILLED → ARCHIVED
```

| Status | Description | Actions Allowed |
|--------|-------------|-----------------|
| **PROPOSED** | Offer accepted, not paid | Modify, cancel |
| **LOCKED** | Inventory reserved | Pay, cancel |
| **PAID** | Payment confirmed | Modify (with fee), cancel (refund) |
| **FULFILLED** | All items delivered | Archive only |
| **ARCHIVED** | Completed | Read-only |

### Example Order

```json
{
  "order_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "customer_id": "user@example.com",
  "status": "PAID",
  "items": [
    {
      "id": "item-1",
      "type": "FLIGHT",
      "status": "ACTIVE",
      "price_nuc": 20000
    },
    {
      "id": "item-2",
      "type": "MEAL",
      "status": "REFUNDED",
      "price_nuc": 1500,
      "refunded_at": "2024-06-01T11:00:00Z"
    }
  ],
  "total_nuc": 20000,
  "created_at": "2024-06-01T10:00:00Z",
  "updated_at": "2024-06-01T11:00:00Z"
}
```

---

## Offer vs. Order Comparison

| Aspect | Offer | Order |
|--------|-------|-------|
| **Lifespan** | 15 minutes | Permanent |
| **Storage** | Redis (cache) | PostgreSQL (database) |
| **Purpose** | Shopping cart | Purchase record |
| **Modifiable** | No (expires) | Yes (add/refund items) |
| **Payment** | Not required | Required for PAID status |
| **Fulfillment** | None | Barcodes generated |

---

## Order Items

Every product in an order is an **OrderItem**.

### Unified Model

```rust
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_type: ProductType,  // FLIGHT, MEAL, SEAT, etc.
    pub price_nuc: i32,
    pub status: OrderItemStatus,    // ACTIVE, REFUNDED, CANCELLED
}
```

### Why Unified?

- **Flexibility**: Add new product types without schema changes
- **Simplicity**: Same logic for flights, meals, bags, etc.
- **Bundling**: Easy to combine any products

---

## Order Modifications

### The Old Way (PNR/E-Ticket)

1. Customer wants to change flight
2. Agent calls Amadeus API
3. Ticket must be **revalidated** (manual process)
4. New ticket issued
5. Cost: $15-30 per change

### The Altis Way

1. Customer requests change via API
2. Add new flight item to order
3. Mark old flight item as `REFUNDED`
4. Update total price
5. Cost: **$0** (just a database update)

```rust
// Zero-cost flight change
order.add_item(new_flight);
order.refund_item(old_flight_id);
```

---

## Fulfillment

### What is Fulfillment?

**Fulfillment** is the delivery of an order item to the customer.

For flights, this means:
- **Barcode** for boarding pass
- **QR code** for mobile check-in
- **Seat assignment** confirmation

### Example

```json
{
  "fulfillment": [
    {
      "item_id": "item-1",
      "barcode": "ALTIS-1717234567-A1B2C3D4",
      "qr_code_data": "{...}",
      "is_consumed": false,
      "created_at": "2024-06-01T10:05:00Z"
    }
  ]
}
```

### Consumption Tracking

When a barcode is scanned at the gate:
```rust
fulfillment.consume();
fulfillment.is_consumed = true;
fulfillment.consumed_at = Some(Utc::now());
```

**Prevents**: Double-boarding, fraud

---

## Continuous Pricing

### The Problem with Fare Buckets

Traditional airlines use discrete fare classes:

```
A: $200 (10 seats)
B: $250 (20 seats)
C: $300 (30 seats)
```

When A sells out, price jumps from $200 → $250 instantly.

### Continuous Pricing

Altis adjusts prices **by the cent** based on real-time demand:

```
90 seats left: $200.00
89 seats left: $200.50
88 seats left: $201.00
...
10 seats left: $450.00
```

### Algorithm

```rust
let utilization = 1.0 - (available / capacity);
let multiplier = 1.0 + (utilization² × 2.0);
let price = base_price × multiplier;
```

### Advantages

- **Revenue optimization**: Capture maximum willingness to pay
- **Customer experience**: Smooth price curve, less sticker shock
- **Competitive edge**: Undercut competitors by cents, not dollars

---

## AI-Driven Offer Ranking

### Rule-Based (Current)

```rust
let score = (conversion_probability × 0.6) + (profit_margin × 0.4);
```

- **Conversion probability**: Estimated from item count (fewer = higher)
- **Profit margin**: Normalized price (higher = better)

### ML-Based (Future)

Train a model on historical data:
- **Features**: Route, dates, user profile, time-of-day
- **Target**: Conversion (0/1)
- **Output**: Probability score (0.0 - 1.0)

Rank offers by: `predicted_conversion × profit_margin`

---

## Business Impact

### Revenue Growth

| Metric | Traditional | Altis | Improvement |
|--------|-------------|-------|-------------|
| Ancillary revenue per booking | $10 | $13 | **+30%** |
| Average order value | $200 | $250 | **+25%** |
| Conversion rate | 3% | 3.45% | **+15%** |

### Cost Reduction

| Process | Traditional | Altis | Savings |
|---------|-------------|-------|---------|
| Flight change | $15-30 | $0 | **100%** |
| Ticket revalidation | Manual | Automated | **100%** |
| Support calls | High | Low | **40%** |

---

## Summary

**Offers** = Temporary shopping carts with AI-ranked bundles  
**Orders** = Permanent purchase records with zero-cost modifications

This architecture enables:
- ✅ Continuous pricing (maximize revenue)
- ✅ Seamless ancillary sales (increase basket size)
- ✅ Zero-cost changes (reduce support costs)
- ✅ AI-driven merchandising (optimize conversion)
