# Continuous Pricing Guide

## Overview

Continuous pricing adjusts product prices **by the cent** based on real-time demand, replacing traditional fare bucket systems.

---

## Traditional Fare Buckets

Airlines traditionally use discrete fare classes:

```
Class A: $200 (10 seats available)
Class B: $250 (20 seats available)
Class C: $300 (30 seats available)
Class Y: $400 (unlimited)
```

### Problems

1. **Revenue leakage**: Customer willing to pay $240 gets $200 ticket
2. **Sticker shock**: Price jumps from $200 → $250 when bucket sells out
3. **Complexity**: Managing dozens of fare rules and restrictions
4. **Inflexibility**: Can't adjust prices between buckets

---

## Continuous Pricing Model

Altis adjusts prices smoothly as inventory sells:

```
100 seats left: $200.00
90 seats left:  $210.00
50 seats left:  $300.00
10 seats left:  $524.00
1 seat left:    $592.00
```

### Algorithm

```rust
pub fn calculate_demand_multiplier(available: i32, capacity: i32) -> f64 {
    let utilization = 1.0 - (available as f64 / capacity as f64);
    
    // Exponential curve: price increases as seats sell
    let multiplier = 1.0 + (utilization * utilization * 2.0);
    
    // Clamp to reasonable limits
    multiplier.max(0.5).min(3.0)
}
```

### Visualization

```
Price
  ^
  |                                    ●
  |                                 ●
  |                              ●
  |                           ●
  |                        ●
  |                     ●
  |                  ●
  |               ●
  |            ●
  |         ●
  |      ●
  |   ●
  |●
  +---------------------------------> Utilization
  0%                              100%
```

---

## Pricing Components

### 1. Base Price

The starting price for a product:

```rust
pub struct Product {
    pub base_price_nuc: i32,  // e.g., 20000 = $200.00
}
```

### 2. Demand Multiplier

Based on inventory utilization:

| Utilization | Multiplier | Example Price |
|-------------|------------|---------------|
| 0% | 1.0x | $200 |
| 25% | 1.125x | $225 |
| 50% | 1.5x | $300 |
| 75% | 2.125x | $425 |
| 90% | 2.62x | $524 |
| 99% | 2.96x | $592 |

### 3. Time Multiplier

Adjust based on time until departure:

```rust
pub fn calculate_time_multiplier(days_until_departure: i32) -> f64 {
    if days_until_departure <= 1 {
        1.5  // Last-minute premium
    } else if days_until_departure <= 7 {
        1.2  // Week-out premium
    } else if days_until_departure >= 60 {
        0.8  // Early-bird discount
    } else {
        1.0  // Normal pricing
    }
}
```

### 4. Bundle Discount

Encourage multi-product purchases:

```rust
if context.is_bundled {
    price = (price as f64 * 0.9) as i32;  // 10% discount
}
```

---

## Pricing Context

```rust
pub struct PricingContext {
    pub timestamp: DateTime<Utc>,
    pub is_bundled: bool,
    pub user_segment: Option<String>,  // "business", "leisure", "vip"
    pub time_multiplier: Option<f64>,
    pub demand_multiplier: Option<f64>,
}
```

---

## Example Calculation

### Scenario

- **Product**: JFK → LHR flight
- **Base price**: $200 (20,000 NUC)
- **Available seats**: 10 / 100
- **Days until departure**: 3
- **Bundled**: Yes

### Calculation

```rust
// 1. Demand multiplier
let utilization = 1.0 - (10.0 / 100.0) = 0.9;
let demand_mult = 1.0 + (0.9 * 0.9 * 2.0) = 2.62;

// 2. Time multiplier
let time_mult = 1.2;  // 3 days out

// 3. Apply multipliers
let price = 20000.0 * 2.62 * 1.2 = 62,880 NUC

// 4. Bundle discount
let final_price = 62880.0 * 0.9 = 56,592 NUC = $565.92
```

---

## Clamping and Limits

### Min/Max Multipliers

```rust
multiplier.max(0.5).min(3.0)
```

- **Min (0.5x)**: Never sell below 50% of base price
- **Max (3.0x)**: Never exceed 3x base price

### Why Clamp?

- **Brand protection**: Avoid appearing "too cheap"
- **Customer trust**: Prevent extreme price gouging
- **Competitive positioning**: Stay within market norms

---

## Continuous Adjustment

### Real-Time Updates

Every time inventory changes:

```rust
// Booking confirmed
inventory.commit_reservation(product_id, quantity);

// Recalculate price for next customer
let new_price = pricing_engine.calculate_price(&context);
```

### Update Frequency

- **Inventory changes**: Immediate
- **Time-based**: Every hour
- **External events**: Weather, competitor pricing (future)

---

## Business Advantages

### 1. Revenue Optimization

**Traditional**: Customer pays $200, willing to pay $240 → **$40 lost**  
**Continuous**: Customer pays $238 → **$38 gained**

### 2. Competitive Pricing

**Traditional**: Competitor at $249, you at $250 → **Customer lost**  
**Continuous**: You at $248.50 → **Customer won**

### 3. Inventory Management

**Traditional**: Hold seats for high-value buckets → **Empty seats**  
**Continuous**: Sell at current market price → **Full flights**

---

## Implementation

### PricingEngine

```rust
pub struct PricingEngine {
    pub min_multiplier: f64,
    pub max_multiplier: f64,
    pub demand_sensitivity: f64,
}

impl PricingEngine {
    pub fn apply_continuous_adjustment(
        &self,
        base_price: i32,
        multiplier: f64,
    ) -> i32 {
        let clamped = multiplier.max(self.min_multiplier).min(self.max_multiplier);
        (base_price as f64 * clamped) as i32
    }
}
```

### Usage

```rust
let engine = PricingEngine::default();
let context = PricingContext {
    timestamp: Utc::now(),
    is_bundled: true,
    demand_multiplier: Some(2.5),
    time_multiplier: Some(1.2),
    ..Default::default()
};

let price = engine.calculate_price(&product, &context)?;
```

---

## Future Enhancements

### 1. ML-Based Pricing

Train a model to predict optimal price:
- **Features**: Route, day-of-week, seasonality, competitor prices
- **Target**: Revenue per seat
- **Output**: Recommended multiplier

### 2. Competitor Monitoring

Scrape competitor prices and adjust:
```rust
if competitor_price < our_price {
    multiplier *= 0.95;  // Undercut by 5%
}
```

### 3. Personalized Pricing

Adjust based on user profile:
```rust
if user.segment == "vip" {
    multiplier *= 0.9;  // VIP discount
}
```

### 4. Dynamic Bundling

Adjust bundle discounts based on conversion data:
```rust
if bundle_conversion_rate < 0.3 {
    bundle_discount = 0.15;  // Increase discount
}
```

---

## Summary

Continuous pricing enables:
- ✅ **Revenue maximization**: Capture every dollar of willingness to pay
- ✅ **Competitive advantage**: Win on price by cents, not dollars
- ✅ **Customer experience**: Smooth price curve, less sticker shock
- ✅ **Operational simplicity**: No fare bucket management

**Key Formula**: `price = base_price × demand_multiplier × time_multiplier × bundle_discount`
