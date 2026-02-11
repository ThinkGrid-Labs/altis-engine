# Admin CMS Guide

## Overview

The Altis Admin CMS allows airline administrators to configure their product catalog, pricing rules, bundle templates, and business logic without code changes. This is a multi-tenant SaaS platform where each airline has isolated configuration.

---

## Quick Start

### 1. Login to Admin Portal

```bash
POST /v1/admin/auth/login
{
  "email": "admin@americanairlines.com",
  "password": "your-password"
}
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 7200,
  "user": {
    "id": "admin-456",
    "email": "admin@americanairlines.com",
    "role": "ADMIN",
    "airline_id": "airline-aa",
    "permissions": [
      "manage_products",
      "manage_pricing",
      "manage_bundles",
      "view_analytics"
    ]
  }
}
```

Use this token in all subsequent requests:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

## Product Catalog Management

### Creating Products

Products are the building blocks of your offers. Everything is a product: flights, seats, meals, bags, lounge access, etc.

#### Example: Create Extra Legroom Seat

```bash
POST /v1/admin/airlines/{airline_id}/products
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "product_type": "SEAT",
  "product_code": "SEAT-EXTRA-LEG",
  "name": "Extra Legroom Seat",
  "description": "Seats with 34-36 inches of legroom in rows 12-15",
  "base_price_nuc": 3000,
  "metadata": {
    "category": "EXTRA_LEGROOM",
    "legroom_inches": 35,
    "recline_inches": 5,
    "available_rows": [12, 13, 14, 15]
  }
}
```

#### Example: Create Hot Meal

```bash
POST /v1/admin/airlines/{airline_id}/products
{
  "product_type": "MEAL",
  "product_code": "MEAL-HOT",
  "name": "Hot Meal",
  "description": "Chef-prepared hot meal with choice of protein",
  "base_price_nuc": 1500,
  "metadata": {
    "category": "HOT",
    "dietary_options": ["VEGETARIAN", "CHICKEN", "BEEF", "VEGAN"],
    "allergens": ["GLUTEN", "DAIRY"]
  }
}
```

### Listing Products

```bash
GET /v1/admin/airlines/{airline_id}/products?product_type=SEAT
```

**Response**:
```json
{
  "products": [
    {
      "id": "product-123",
      "product_type": "SEAT",
      "product_code": "SEAT-EXTRA-LEG",
      "name": "Extra Legroom Seat",
      "base_price_nuc": 3000,
      "is_active": true
    }
  ]
}
```

---

## Pricing Rules Configuration

### Continuous Pricing (Demand-Based)

Adjust prices dynamically based on seat utilization:

```bash
POST /v1/admin/airlines/{airline_id}/pricing-rules
{
  "rule_name": "Continuous Pricing - Economy Seats",
  "rule_type": "DEMAND",
  "product_id": "product-123",
  "conditions": {
    "product_type": "SEAT",
    "cabin_class": "ECONOMY",
    "utilization": {"min": 0.0}
  },
  "adjustments": {
    "type": "FORMULA",
    "formula": "1.0 + (utilization^2 * 2.0)",
    "min_multiplier": 0.5,
    "max_multiplier": 3.0
  },
  "priority": 10
}
```

**How it works**:
- 0% sold: Price × 1.0 (base price)
- 50% sold: Price × 1.5
- 70% sold: Price × 1.98
- 90% sold: Price × 2.62
- 100% sold: Price × 3.0 (capped)

### Time-Based Pricing

Increase prices for last-minute bookings:

```bash
POST /v1/admin/airlines/{airline_id}/pricing-rules
{
  "rule_name": "Last-Minute Premium",
  "rule_type": "TIME",
  "product_id": "product-123",
  "conditions": {
    "days_until_departure": {"max": 7}
  },
  "adjustments": {
    "type": "MULTIPLIER",
    "value": 1.5
  },
  "priority": 20
}
```

### Seasonal Pricing

Adjust prices for peak travel seasons:

```bash
POST /v1/admin/airlines/{airline_id}/pricing-rules
{
  "rule_name": "Summer Peak Season",
  "rule_type": "SEASONAL",
  "product_id": "product-123",
  "conditions": {
    "date_range": {
      "start": "2024-06-01",
      "end": "2024-08-31"
    }
  },
  "adjustments": {
    "type": "MULTIPLIER",
    "value": 1.3
  },
  "priority": 15
}
```

### Testing Pricing Rules

Before activating, test how your rules affect prices:

```bash
POST /v1/admin/pricing-rules/{rule_id}/test
{
  "scenario": {
    "product_id": "product-123",
    "utilization": 0.8,
    "days_until_departure": 3,
    "date": "2024-07-15"
  }
}
```

**Response**:
```json
{
  "base_price_nuc": 3000,
  "applied_rules": [
    {"rule_name": "Continuous Pricing", "multiplier": 2.28},
    {"rule_name": "Last-Minute Premium", "multiplier": 1.5},
    {"rule_name": "Summer Peak Season", "multiplier": 1.3}
  ],
  "final_price_nuc": 13338,
  "final_price_usd": "$133.38"
}
```

---

## Bundle Templates

Bundle templates define pre-configured offer combinations with discounts.

### Creating a Comfort Bundle

```bash
POST /v1/admin/airlines/{airline_id}/bundles
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

**How it works**:
- System finds matching products for each type
- Applies 10% discount to ancillaries (not flights)
- Ranks this bundle with priority 2 (higher = shown first)

### Creating a Premium Bundle

```bash
POST /v1/admin/airlines/{airline_id}/bundles
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

### Testing Bundles

Preview what offers will be generated:

```bash
POST /v1/admin/bundles/{bundle_id}/test
{
  "search_params": {
    "origin": "JFK",
    "destination": "LHR",
    "date": "2024-06-01",
    "passengers": 2
  }
}
```

**Response**:
```json
{
  "offer": {
    "bundle_name": "Comfort Bundle",
    "items": [
      {"type": "FLIGHT", "name": "UA300 JFK-LHR", "price": 36000},
      {"type": "SEAT", "name": "Extra Legroom", "price": 2700, "discount": 300},
      {"type": "MEAL", "name": "Hot Meal", "price": 1350, "discount": 150}
    ],
    "total": 40050,
    "savings": 450
  }
}
```

---

## Inventory Rules

Configure how long seats/products are held during checkout:

```bash
POST /v1/admin/airlines/{airline_id}/inventory-rules
{
  "resource_type": "SEAT",
  "hold_duration_seconds": 900,  // 15 minutes
  "overbooking_percentage": 5.0,
  "min_availability_threshold": 10,
  "auto_release_on_expiry": true,
  "notify_on_low_inventory": true
}
```

---

## Flight Management

### Adding Flights

```bash
POST /v1/admin/airlines/{airline_id}/flights
{
  "flight_number": "AA100",
  "origin": "JFK",
  "destination": "LHR",
  "departure_time": "2024-06-01T08:00:00Z",
  "arrival_time": "2024-06-01T20:00:00Z",
  "aircraft_type": "Boeing 777-300ER",
  "total_seats": 350,
  "cabin_configuration": {
    "FIRST": 8,
    "BUSINESS": 52,
    "PREMIUM_ECONOMY": 48,
    "ECONOMY": 242
  }
}
```

### Bulk Import

Upload CSV of flights:

```bash
POST /v1/admin/airlines/{airline_id}/flights/bulk
Content-Type: multipart/form-data

file: flights.csv
```

**CSV Format**:
```csv
flight_number,origin,destination,departure_time,arrival_time,aircraft_type,total_seats
AA100,JFK,LHR,2024-06-01T08:00:00Z,2024-06-01T20:00:00Z,Boeing 777,350
AA101,LHR,JFK,2024-06-02T09:00:00Z,2024-06-02T12:00:00Z,Boeing 777,350
```

---

## Analytics & Reports

### Revenue Analytics

```bash
GET /v1/admin/airlines/{airline_id}/analytics/revenue?start_date=2024-06-01&end_date=2024-06-30
```

**Response**:
```json
{
  "total_revenue_nuc": 5000000,
  "total_revenue_usd": "$50,000",
  "breakdown": {
    "FLIGHT": 4000000,
    "SEAT": 500000,
    "MEAL": 300000,
    "BAG": 200000
  },
  "average_order_value": 25000
}
```

### Conversion Analytics

```bash
GET /v1/admin/airlines/{airline_id}/analytics/conversion?start_date=2024-06-01&end_date=2024-06-30
```

**Response**:
```json
{
  "total_offers": 10000,
  "accepted_offers": 2500,
  "conversion_rate": 0.25,
  "bundle_performance": [
    {"bundle_name": "Comfort Bundle", "conversion_rate": 0.35},
    {"bundle_name": "Flight Only", "conversion_rate": 0.20},
    {"bundle_name": "Premium", "conversion_rate": 0.15}
  ]
}
```

---

## User Management

### Creating Admin Users

```bash
POST /v1/admin/users
{
  "email": "manager@americanairlines.com",
  "role": "ADMIN",
  "airline_id": "airline-aa",
  "permissions": [
    "manage_products",
    "manage_pricing",
    "view_analytics"
  ]
}
```

### Roles & Permissions

| Role | Description | Scope |
|------|-------------|-------|
| `SUPER_ADMIN` | Platform administrator | All airlines |
| `ADMIN` | Airline administrator | Single airline |
| `ANALYST` | Read-only access | Single airline |

| Permission | Allows |
|------------|--------|
| `manage_products` | Create/edit/delete products |
| `manage_pricing` | Configure pricing rules |
| `manage_bundles` | Configure bundle templates |
| `manage_flights` | Add/edit flight schedules |
| `view_analytics` | View reports and analytics |
| `manage_users` | Create/edit admin users |

---

## Audit Logs

All admin actions are automatically logged:

```bash
GET /v1/admin/airlines/{airline_id}/audit-logs?start_date=2024-06-01&limit=100
```

**Response**:
```json
{
  "logs": [
    {
      "id": "log-123",
      "action": "CREATE",
      "resource": "PRICING_RULE",
      "changed_by": "admin@americanairlines.com",
      "changes": {
        "rule_name": "Continuous Pricing - Economy Seats",
        "rule_type": "DEMAND"
      },
      "created_at": "2024-06-01T10:30:00Z"
    }
  ]
}
```

---

## Best Practices

### 1. Test Before Activating

Always test pricing rules and bundles before activating:
```bash
POST /v1/admin/pricing-rules/{id}/test
POST /v1/admin/bundles/{id}/test
```

### 2. Use Priority Wisely

Higher priority rules are applied first:
- **Priority 30+**: Seasonal/promotional overrides
- **Priority 20-29**: Time-based rules
- **Priority 10-19**: Demand-based rules
- **Priority 1-9**: Base rules

### 3. Monitor Conversion Rates

Check analytics weekly to see which bundles perform best:
```bash
GET /v1/admin/airlines/{airline_id}/analytics/conversion
```

### 4. Set Reasonable Hold Times

- **Seats**: 15 minutes (900 seconds)
- **Meals**: 10 minutes (600 seconds)
- **Lounge**: 5 minutes (300 seconds)

### 5. Enable Audit Logging

Review audit logs monthly for compliance:
```bash
GET /v1/admin/airlines/{airline_id}/audit-logs
```

---

## Common Workflows

### Workflow 1: Launch New Route

1. Add flights via bulk import
2. Create products (seats, meals) for the route
3. Configure pricing rules (demand-based)
4. Create bundle templates
5. Test bundles with sample searches
6. Activate and monitor conversion

### Workflow 2: Seasonal Promotion

1. Create seasonal pricing rule (e.g., "Summer Sale")
2. Set high priority (30+) to override other rules
3. Set valid date range
4. Test with various scenarios
5. Activate rule
6. Monitor revenue impact

### Workflow 3: Optimize Bundles

1. Review conversion analytics
2. Identify low-performing bundles
3. Adjust discount percentages
4. Test new configurations
5. Update bundle templates
6. Monitor for 2 weeks

---

## Troubleshooting

### Issue: Pricing rule not applying

**Check**:
1. Rule is active (`is_active: true`)
2. Rule priority is correct
3. Conditions match the scenario
4. Valid date range includes current date

**Solution**:
```bash
GET /v1/admin/pricing-rules/{id}
PATCH /v1/admin/pricing-rules/{id}/status
{"is_active": true}
```

### Issue: Bundle not showing in offers

**Check**:
1. All required product types exist
2. Products are active
3. Bundle priority is set
4. Bundle is active

**Solution**:
```bash
POST /v1/admin/bundles/{id}/test
```

### Issue: Seat hold expired too quickly

**Check**:
1. Inventory rule for SEAT resource
2. `hold_duration_seconds` value

**Solution**:
```bash
PUT /v1/admin/inventory-rules/{id}
{"hold_duration_seconds": 900}
```

---

## Support

For technical support:
- **Email**: support@altis.com
- **Docs**: https://docs.altis.com
- **API Reference**: `/docs/API.md`
