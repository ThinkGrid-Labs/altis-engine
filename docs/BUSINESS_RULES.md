# Configurable Business Rules System

## Overview

A flexible, database-driven business rules engine that allows airlines to configure pricing, bundling, inventory, and product offerings without code changes. Designed to support a future CMS for airline administrators.

---

## Database Schema

### 1. Airlines Table

```sql
CREATE TABLE airlines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(3) UNIQUE NOT NULL,  -- e.g., 'AA', 'UA', 'BA'
    name VARCHAR(255) NOT NULL,
    country VARCHAR(2),
    status VARCHAR(20) DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_airlines_code ON airlines(code);
```

### 2. Business Rules Table

```sql
CREATE TABLE business_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id),
    rule_type VARCHAR(50) NOT NULL,  -- 'PRICING', 'BUNDLING', 'INVENTORY', 'OFFER'
    rule_name VARCHAR(100) NOT NULL,
    rule_config JSONB NOT NULL,
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    valid_from TIMESTAMPTZ,
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_business_rules_airline ON business_rules(airline_id);
CREATE INDEX idx_business_rules_type ON business_rules(rule_type);
CREATE INDEX idx_business_rules_active ON business_rules(is_active);
```

### 3. Products Table (Airline-Specific)

```sql
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id),
    product_type VARCHAR(50) NOT NULL,  -- 'FLIGHT', 'SEAT', 'MEAL', 'BAG', etc.
    product_code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    base_price_nuc INTEGER NOT NULL,
    currency VARCHAR(3) DEFAULT 'NUC',
    metadata JSONB,  -- Product-specific attributes
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(airline_id, product_code)
);

CREATE INDEX idx_products_airline ON products(airline_id);
CREATE INDEX idx_products_type ON products(product_type);
```

### 4. Pricing Rules Table

```sql
CREATE TABLE pricing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id),
    product_id UUID REFERENCES products(id),
    rule_name VARCHAR(100) NOT NULL,
    rule_type VARCHAR(50) NOT NULL,  -- 'DEMAND', 'TIME', 'SEASONAL', 'BUNDLE'
    conditions JSONB NOT NULL,  -- When to apply this rule
    adjustments JSONB NOT NULL,  -- How to adjust price
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_pricing_rules_airline ON pricing_rules(airline_id);
CREATE INDEX idx_pricing_rules_product ON pricing_rules(product_id);
```

### 5. Bundle Templates Table

```sql
CREATE TABLE bundle_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id),
    bundle_name VARCHAR(100) NOT NULL,
    bundle_type VARCHAR(50) NOT NULL,  -- 'ECONOMY', 'COMFORT', 'PREMIUM'
    product_types JSONB NOT NULL,  -- Which product types to include
    discount_percentage DECIMAL(5,2) DEFAULT 0,
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_bundle_templates_airline ON bundle_templates(airline_id);
```

### 6. Inventory Rules Table

```sql
CREATE TABLE inventory_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id),
    resource_type VARCHAR(50) NOT NULL,  -- 'SEAT', 'MEAL', 'LOUNGE'
    hold_duration_seconds INTEGER DEFAULT 900,  -- 15 minutes
    overbooking_percentage DECIMAL(5,2) DEFAULT 0,
    min_availability_threshold INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## Rule Configuration Examples

### 1. Pricing Rules

#### Demand-Based Pricing

```json
{
  "rule_name": "Continuous Pricing - Economy",
  "rule_type": "DEMAND",
  "conditions": {
    "product_type": "FLIGHT",
    "cabin_class": "ECONOMY"
  },
  "adjustments": {
    "type": "MULTIPLIER",
    "formula": "1.0 + (utilization^2 * 2.0)",
    "min_multiplier": 0.5,
    "max_multiplier": 3.0
  }
}
```

#### Time-Based Pricing

```json
{
  "rule_name": "Last-Minute Premium",
  "rule_type": "TIME",
  "conditions": {
    "days_until_departure": {"max": 7}
  },
  "adjustments": {
    "type": "MULTIPLIER",
    "value": 1.5
  }
}
```

#### Seasonal Pricing

```json
{
  "rule_name": "Summer Peak Season",
  "rule_type": "SEASONAL",
  "conditions": {
    "date_range": {
      "start": "2024-06-01",
      "end": "2024-08-31"
    }
  },
  "adjustments": {
    "type": "MULTIPLIER",
    "value": 1.3
  }
}
```

### 2. Bundle Templates

#### Comfort Bundle

```json
{
  "bundle_name": "Comfort Bundle",
  "bundle_type": "COMFORT",
  "product_types": [
    {"type": "FLIGHT", "required": true},
    {"type": "SEAT", "category": "EXTRA_LEGROOM", "required": false},
    {"type": "MEAL", "category": "HOT", "required": false},
    {"type": "BAG", "quantity": 1, "required": false}
  ],
  "discount_percentage": 10.0,
  "priority": 2
}
```

#### Premium Bundle

```json
{
  "bundle_name": "Premium Experience",
  "bundle_type": "PREMIUM",
  "product_types": [
    {"type": "FLIGHT", "cabin_class": "BUSINESS", "required": true},
    {"type": "LOUNGE", "required": true},
    {"type": "FAST_TRACK", "required": true},
    {"type": "SEAT", "category": "PREMIUM", "required": true}
  ],
  "discount_percentage": 15.0,
  "priority": 3
}
```

### 3. Inventory Rules

```json
{
  "resource_type": "SEAT",
  "hold_duration_seconds": 900,
  "overbooking_percentage": 5.0,
  "min_availability_threshold": 10,
  "release_policy": {
    "auto_release_on_expiry": true,
    "notify_on_low_inventory": true
  }
}
```

### 4. Offer Generation Rules

```json
{
  "rule_name": "Standard Offer Mix",
  "rule_type": "OFFER",
  "rule_config": {
    "max_offers": 5,
    "offer_types": [
      {"type": "FLIGHT_ONLY", "priority": 1, "always_include": true},
      {"type": "COMFORT", "priority": 2, "always_include": true},
      {"type": "PREMIUM", "priority": 3, "always_include": false}
    ],
    "ranking_weights": {
      "conversion_probability": 0.6,
      "profit_margin": 0.4
    },
    "expiry_minutes": 15
  }
}
```

---

## API Design for CMS

### 1. Airline Management

```bash
# List airlines
GET /v1/admin/airlines

# Create airline
POST /v1/admin/airlines
{
  "code": "AA",
  "name": "American Airlines",
  "country": "US"
}

# Update airline
PUT /v1/admin/airlines/{airline_id}

# Get airline details
GET /v1/admin/airlines/{airline_id}
```

### 2. Business Rules Management

```bash
# List rules for airline
GET /v1/admin/airlines/{airline_id}/rules?type=PRICING

# Create pricing rule
POST /v1/admin/airlines/{airline_id}/rules
{
  "rule_type": "PRICING",
  "rule_name": "Continuous Pricing - Economy",
  "rule_config": {
    "conditions": {...},
    "adjustments": {...}
  },
  "priority": 10,
  "valid_from": "2024-06-01T00:00:00Z",
  "valid_until": "2024-12-31T23:59:59Z"
}

# Update rule
PUT /v1/admin/rules/{rule_id}

# Activate/Deactivate rule
PATCH /v1/admin/rules/{rule_id}/status
{
  "is_active": false
}

# Delete rule
DELETE /v1/admin/rules/{rule_id}
```

### 3. Product Management

```bash
# List products
GET /v1/admin/airlines/{airline_id}/products?type=SEAT

# Create product
POST /v1/admin/airlines/{airline_id}/products
{
  "product_type": "SEAT",
  "product_code": "SEAT-EXTRA-LEG",
  "name": "Extra Legroom Seat",
  "description": "Seats with 34-36 inches of legroom",
  "base_price_nuc": 3000,
  "metadata": {
    "legroom_inches": 35,
    "recline_inches": 5,
    "available_rows": [12, 13, 14, 15]
  }
}

# Update product
PUT /v1/admin/products/{product_id}

# Bulk import products
POST /v1/admin/airlines/{airline_id}/products/bulk
{
  "products": [...]
}
```

### 4. Bundle Template Management

```bash
# List bundle templates
GET /v1/admin/airlines/{airline_id}/bundles

# Create bundle template
POST /v1/admin/airlines/{airline_id}/bundles
{
  "bundle_name": "Comfort Bundle",
  "bundle_type": "COMFORT",
  "product_types": [...],
  "discount_percentage": 10.0
}

# Test bundle (preview offers)
POST /v1/admin/bundles/{bundle_id}/test
{
  "search_params": {
    "origin": "JFK",
    "destination": "LHR",
    "date": "2024-06-01"
  }
}
```

### 5. Pricing Rule Management

```bash
# List pricing rules
GET /v1/admin/airlines/{airline_id}/pricing-rules

# Create pricing rule
POST /v1/admin/airlines/{airline_id}/pricing-rules
{
  "rule_name": "Last-Minute Premium",
  "rule_type": "TIME",
  "conditions": {...},
  "adjustments": {...}
}

# Simulate pricing
POST /v1/admin/pricing-rules/simulate
{
  "product_id": "product-123",
  "scenario": {
    "utilization": 0.8,
    "days_until_departure": 3
  }
}
```

---

## Rule Evaluation Engine

### Pricing Calculation Flow

```rust
pub struct RuleEngine {
    db: Arc<DbClient>,
    cache: Arc<RedisClient>,
}

impl RuleEngine {
    pub async fn calculate_price(
        &self,
        airline_id: Uuid,
        product: &Product,
        context: &PricingContext,
    ) -> Result<i32, Error> {
        // 1. Get base price
        let mut price = product.base_price_nuc as f64;
        
        // 2. Fetch active pricing rules for this airline
        let rules = self.get_active_pricing_rules(airline_id, &product.product_type).await?;
        
        // 3. Apply rules in priority order
        for rule in rules.iter().sorted_by_key(|r| r.priority) {
            if self.evaluate_conditions(&rule.conditions, context)? {
                price = self.apply_adjustment(price, &rule.adjustments)?;
            }
        }
        
        Ok(price as i32)
    }
    
    fn evaluate_conditions(
        &self,
        conditions: &serde_json::Value,
        context: &PricingContext,
    ) -> Result<bool, Error> {
        // Evaluate conditions like:
        // - days_until_departure < 7
        // - utilization > 0.8
        // - date in range
        // - cabin_class == "ECONOMY"
        
        // Implementation uses a simple expression evaluator
        Ok(true)  // Simplified
    }
    
    fn apply_adjustment(
        &self,
        base_price: f64,
        adjustments: &serde_json::Value,
    ) -> Result<f64, Error> {
        let adj_type = adjustments["type"].as_str().unwrap();
        
        match adj_type {
            "MULTIPLIER" => {
                let multiplier = adjustments["value"].as_f64().unwrap();
                Ok(base_price * multiplier)
            },
            "FIXED" => {
                let amount = adjustments["value"].as_f64().unwrap();
                Ok(base_price + amount)
            },
            "FORMULA" => {
                // Evaluate formula like "1.0 + (utilization^2 * 2.0)"
                let formula = adjustments["formula"].as_str().unwrap();
                let result = self.evaluate_formula(formula, base_price)?;
                Ok(result)
            },
            _ => Ok(base_price)
        }
    }
}
```

### Bundle Generation Flow

```rust
pub async fn generate_offers(
    &self,
    airline_id: Uuid,
    search: &SearchRequest,
) -> Result<Vec<Offer>, Error> {
    // 1. Get bundle templates for airline
    let templates = self.get_active_bundle_templates(airline_id).await?;
    
    // 2. Get available products
    let flights = self.search_flights(airline_id, search).await?;
    let ancillaries = self.get_ancillary_products(airline_id).await?;
    
    // 3. Generate offers from templates
    let mut offers = Vec::new();
    
    for template in templates {
        let offer = self.build_offer_from_template(
            &template,
            &flights,
            &ancillaries,
            search,
        ).await?;
        
        offers.push(offer);
    }
    
    // 4. Rank offers
    self.rank_offers(&mut offers, airline_id).await?;
    
    // 5. Return top N offers
    Ok(offers.into_iter().take(5).collect())
}
```

---

## CMS UI Mockup

### Pricing Rules Screen

```
┌─────────────────────────────────────────────────────────────┐
│ American Airlines - Pricing Rules                           │
├─────────────────────────────────────────────────────────────┤
│ [+ New Rule]  [Import]  [Export]                           │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Rule Name: Continuous Pricing - Economy      [Active ✓]│ │
│ │ Type: Demand-Based                                      │ │
│ │ Priority: 10                                            │ │
│ │                                                         │ │
│ │ Conditions:                                             │ │
│ │   Product Type: FLIGHT                                  │ │
│ │   Cabin Class: ECONOMY                                  │ │
│ │                                                         │ │
│ │ Adjustment:                                             │ │
│ │   Formula: 1.0 + (utilization² × 2.0)                  │ │
│ │   Min Multiplier: 0.5                                   │ │
│ │   Max Multiplier: 3.0                                   │ │
│ │                                                         │ │
│ │ [Edit] [Duplicate] [Test] [Delete]                     │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Rule Name: Last-Minute Premium           [Active ✓]    │ │
│ │ Type: Time-Based                                        │ │
│ │ Priority: 20                                            │ │
│ │ ...                                                     │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Summary

| Component | Configurable? | Storage | CMS UI |
|-----------|---------------|---------|--------|
| **Pricing Rules** | ✅ Yes | Database (JSONB) | Rule builder with formula editor |
| **Bundle Templates** | ✅ Yes | Database (JSONB) | Drag-drop product selector |
| **Products** | ✅ Yes | Database | Product catalog manager |
| **Inventory Rules** | ✅ Yes | Database (JSONB) | Settings panel |
| **Offer Ranking** | ✅ Yes | Database (JSONB) | Weight sliders |

**Key Benefits**:
- ✅ No code changes for new rules
- ✅ Multi-tenant (per-airline configuration)
- ✅ Version control (valid_from/valid_until)
- ✅ A/B testing (multiple rules with priorities)
- ✅ Real-time updates (cache invalidation)
