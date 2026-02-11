# Customer Journey: Offer/Order vs. Traditional Booking

## Overview

This document explains how customers interact with the Offer/Order system compared to traditional flight booking.

---

## Round-Trip Flight Example

### Customer Input (Same for Both Systems)

```json
{
  "trip_type": "ROUND_TRIP",
  "origin": "JFK",
  "destination": "LHR",
  "departure_date": "2024-06-01",
  "return_date": "2024-06-08",
  "passengers": 2,
  "cabin_class": "ECONOMY"
}
```

---

## Traditional System Flow

### Step 1: Search Results

**Outbound Flights (JFK → LHR)**:
```
Flight AA100  08:00 - 20:00  $200/person
Flight BA200  10:00 - 22:00  $220/person
Flight UA300  12:00 - 00:00  $180/person
```

**Inbound Flights (LHR → JFK)**:
```
Flight AA101  09:00 - 12:00  $210/person
Flight BA201  11:00 - 14:00  $230/person
Flight UA301  13:00 - 16:00  $190/person
```

### Step 2: Customer Selects
- Outbound: UA300 ($180 × 2 = $360)
- Inbound: UA301 ($190 × 2 = $380)
- **Total: $740**

### Step 3: Add Ancillaries (Separate Process)
- Want extra bag? Go to separate page (+$50)
- Want meal? Go to separate page (+$30)
- Want seat selection? Go to separate page (+$40)

### Step 4: Final Total
- Flights: $740
- Bag: $50
- Meal: $30
- Seat: $40
- **Grand Total: $860**

**Problem**: Customer had to make 4 separate decisions, no bundle discount, confusing process.

---

## Offer/Order System Flow

### Step 1: Search Results (AI-Generated Offers)

The system generates **3-5 pre-bundled offers** ranked by conversion probability:

#### **Offer 1: Economy Saver** (Highest Conversion)
```json
{
  "offer_id": "offer-123",
  "total": "$740",
  "expires_at": "2024-06-01T10:15:00Z",
  "items": [
    {
      "type": "FLIGHT",
      "route": "JFK → LHR",
      "flight": "UA300",
      "date": "2024-06-01",
      "price": "$360"
    },
    {
      "type": "FLIGHT",
      "route": "LHR → JFK",
      "flight": "UA301",
      "date": "2024-06-08",
      "price": "$380"
    }
  ]
}
```

#### **Offer 2: Comfort Bundle** (Recommended)
```json
{
  "offer_id": "offer-124",
  "total": "$810",
  "savings": "$50 (10% bundle discount)",
  "expires_at": "2024-06-01T10:15:00Z",
  "items": [
    {
      "type": "FLIGHT",
      "route": "JFK → LHR",
      "flight": "UA300",
      "price": "$360"
    },
    {
      "type": "FLIGHT",
      "route": "LHR → JFK",
      "flight": "UA301",
      "price": "$380"
    },
    {
      "type": "SEAT",
      "description": "Extra legroom seats (2x)",
      "price": "$36",
      "original_price": "$40"
    },
    {
      "type": "MEAL",
      "description": "Hot meals (2x)",
      "price": "$27",
      "original_price": "$30"
    },
    {
      "type": "BAG",
      "description": "Checked bags (2x)",
      "price": "$45",
      "original_price": "$50"
    }
  ]
}
```

#### **Offer 3: Premium Experience**
```json
{
  "offer_id": "offer-125",
  "total": "$920",
  "savings": "$70 (10% bundle discount)",
  "expires_at": "2024-06-01T10:15:00Z",
  "items": [
    {
      "type": "FLIGHT",
      "route": "JFK → LHR",
      "flight": "BA200",
      "price": "$440"
    },
    {
      "type": "FLIGHT",
      "route": "LHR → JFK",
      "flight": "BA201",
      "price": "$460"
    },
    {
      "type": "LOUNGE",
      "description": "Airport lounge access (4 visits)",
      "price": "$90",
      "original_price": "$100"
    },
    {
      "type": "FAST_TRACK",
      "description": "Priority security (4 passes)",
      "price": "$54",
      "original_price": "$60"
    }
  ]
}
```

### Step 2: Customer Selects Offer

Customer clicks **"Select Offer 2: Comfort Bundle"**

**What Happens**:
```
POST /v1/offers/offer-124/accept
→ Creates Order (PROPOSED status)
→ Order ID: order-789
```

### Step 3: Order Created

```json
{
  "order_id": "order-789",
  "status": "PROPOSED",
  "customer_email": "user@example.com",
  "total": "$810",
  "items": [
    {"type": "FLIGHT", "JFK → LHR", "status": "ACTIVE"},
    {"type": "FLIGHT", "LHR → JFK", "status": "ACTIVE"},
    {"type": "SEAT", "Extra legroom", "status": "ACTIVE"},
    {"type": "MEAL", "Hot meals", "status": "ACTIVE"},
    {"type": "BAG", "Checked bags", "status": "ACTIVE"}
  ],
  "created_at": "2024-06-01T10:05:00Z"
}
```

### Step 4: Payment

```
POST /v1/orders/order-789/pay
{
  "payment_method": "card",
  "payment_token": "tok_visa_4242"
}
```

**Response**:
```json
{
  "order_id": "order-789",
  "status": "PAID",
  "fulfillment": [
    {
      "item_id": "item-1",
      "type": "FLIGHT",
      "barcode": "ALTIS-1717234567-A1B2C3D4",
      "qr_code_url": "/orders/order-789/qr/item-1"
    },
    {
      "item_id": "item-2",
      "type": "FLIGHT",
      "barcode": "ALTIS-1717234568-B2C3D4E5",
      "qr_code_url": "/orders/order-789/qr/item-2"
    }
  ]
}
```

---

## Key Advantages for Customers

### 1. **Simpler Decision Making**

**Traditional**: 4 separate decisions
- Which outbound flight?
- Which inbound flight?
- Do I want a bag?
- Do I want a meal?

**Offer/Order**: 1 decision
- Which bundle do I want?

### 2. **Transparent Pricing**

**Traditional**:
```
Flight: $740
+ Bag: $50 (surprise!)
+ Meal: $30 (surprise!)
+ Seat: $40 (surprise!)
= $860 (way more than expected)
```

**Offer/Order**:
```
Comfort Bundle: $810 (all-in price)
You save: $50 vs. buying separately
```

### 3. **Easy Modifications**

**Traditional**:
- Change flight → Call airline → Revalidate ticket → Pay $200 change fee
- Add bag → Call airline → Pay $75 (higher than original $50)

**Offer/Order**:
```
POST /v1/orders/order-789/modify
{
  "action": "change_flight",
  "old_item_id": "item-1",
  "new_product_id": "flight-456"
}
```
- Old flight marked as REFUNDED
- New flight added
- Price difference charged
- **No revalidation fee**

---

## Multi-City Example

### Customer Input

```json
{
  "trip_type": "MULTI_CITY",
  "legs": [
    {"origin": "JFK", "destination": "LHR", "date": "2024-06-01"},
    {"origin": "LHR", "destination": "CDG", "date": "2024-06-05"},
    {"origin": "CDG", "destination": "JFK", "date": "2024-06-10"}
  ],
  "passengers": 1
}
```

### Offer Generated

```json
{
  "offer_id": "offer-200",
  "total": "$950",
  "items": [
    {"type": "FLIGHT", "route": "JFK → LHR", "price": "$300"},
    {"type": "FLIGHT", "route": "LHR → CDG", "price": "$150"},
    {"type": "FLIGHT", "route": "CDG → JFK", "price": "$400"},
    {"type": "CARBON_OFFSET", "description": "Offset 2.5 tons CO2", "price": "$25"},
    {"type": "INSURANCE", "description": "Trip protection", "price": "$75"}
  ]
}
```

**Same process**: Accept offer → Create order → Pay → Get barcodes

---

## One-Way Example

### Customer Input

```json
{
  "trip_type": "ONE_WAY",
  "origin": "JFK",
  "destination": "LHR",
  "date": "2024-06-01",
  "passengers": 1
}
```

### Offers Generated

**Offer 1**: Flight only ($200)  
**Offer 2**: Flight + Bag ($240)  
**Offer 3**: Flight + Bag + Lounge ($290)

---

## Summary

| Aspect | Traditional | Offer/Order |
|--------|-------------|-------------|
| **Search** | Same | Same |
| **Results** | List of flights | 3-5 bundled offers |
| **Selection** | Pick flights separately | Pick one bundle |
| **Ancillaries** | Add later (confusing) | Included in bundle |
| **Pricing** | Surprises at checkout | All-in upfront |
| **Modifications** | Expensive ($200 fee) | Cheap (just price diff) |
| **Confirmation** | PNR + E-Ticket | Order ID + Barcode |

**Bottom Line**: The customer journey is **simpler, more transparent, and more flexible** with Offer/Order.
