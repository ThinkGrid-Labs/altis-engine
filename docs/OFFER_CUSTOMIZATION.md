# Offer Selection & Customization Flow

## How Customers Select Flights, Seats, and Meals

---

## Step 1: Search Results - Multiple Offers with Different Flight Times

When a customer searches for JFK → LHR on June 1st, the system generates **3-5 offers**, each with **different flight combinations**:

### Offer Display

```
┌─────────────────────────────────────────────────────────────┐
│ 🏆 RECOMMENDED: Comfort Bundle                    $810      │
├─────────────────────────────────────────────────────────────┤
│ Outbound: AA100  JFK → LHR                                  │
│           Departs: 08:00  Arrives: 20:00  (12h 0m)         │
│                                                             │
│ Inbound:  AA101  LHR → JFK                                  │
│           Departs: 09:00  Arrives: 12:00  (8h 0m)          │
│                                                             │
│ ✓ Extra legroom seats (2x)                                 │
│ ✓ Hot meals (2x)                                           │
│ ✓ Checked bags (2x)                                        │
│                                                             │
│ You save $50 vs. buying separately                         │
│ [Select This Offer]                                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Economy Saver                                     $740      │
├─────────────────────────────────────────────────────────────┤
│ Outbound: UA300  JFK → LHR                                  │
│           Departs: 12:00  Arrives: 00:00+1  (12h 0m)       │
│                                                             │
│ Inbound:  UA301  LHR → JFK                                  │
│           Departs: 13:00  Arrives: 16:00  (8h 0m)          │
│                                                             │
│ Flights only - no extras                                   │
│ [Select This Offer]                                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Premium Experience                                $920      │
├─────────────────────────────────────────────────────────────┤
│ Outbound: BA200  JFK → LHR                                  │
│           Departs: 10:00  Arrives: 22:00  (12h 0m)         │
│                                                             │
│ Inbound:  BA201  LHR → JFK                                  │
│           Departs: 11:00  Arrives: 14:00  (8h 0m)          │
│                                                             │
│ ✓ Airport lounge access (4 visits)                         │
│ ✓ Priority security (4 passes)                             │
│ ✓ Extra legroom seats (2x)                                 │
│                                                             │
│ [Select This Offer]                                        │
└─────────────────────────────────────────────────────────────┘
```

**Key Points**:
- Each offer shows **specific flight numbers and times**
- Customer sees **all flights for the day** across different offers
- AI ranks offers by conversion probability (best deals first)

---

## Step 2: Customize Offer (Before Payment)

When customer clicks **"Select This Offer"**, they see a customization screen:

### Option A: Generic Bundle → Customize After Selection

```
┌─────────────────────────────────────────────────────────────┐
│ Review Your Order                                           │
├─────────────────────────────────────────────────────────────┤
│ Order ID: order-789                                         │
│ Status: PROPOSED (not paid yet)                             │
│                                                             │
│ ✈️  Outbound Flight: AA100                                  │
│     JFK → LHR  |  June 1, 08:00 - 20:00                    │
│                                                             │
│ ✈️  Inbound Flight: AA101                                   │
│     LHR → JFK  |  June 8, 09:00 - 12:00                    │
│                                                             │
│ 💺 Extra Legroom Seats (2x) - $36                          │
│    [Choose Seats] ← Click to select specific seats         │
│                                                             │
│ 🍽️  Hot Meals (2x) - $27                                    │
│    [Choose Meals] ← Click to select meal types             │
│                                                             │
│ 🧳 Checked Bags (2x) - $45                                  │
│    ✓ Included                                              │
│                                                             │
│ Total: $810                                                 │
│ [Proceed to Payment]                                        │
└─────────────────────────────────────────────────────────────┘
```

#### Seat Selection Modal

```
┌─────────────────────────────────────────────────────────────┐
│ Select Seats for AA100 (JFK → LHR)                         │
├─────────────────────────────────────────────────────────────┤
│ Passenger 1: John Doe                                       │
│                                                             │
│   A   B   C       D   E   F                                │
│ 10 [X] [X] [X]   [X] [X] [X]  Standard ($0)               │
│ 11 [X] [X] [X]   [X] [X] [X]                              │
│ 12 [✓] [ ] [ ]   [ ] [ ] [ ]  Extra Legroom (Included)    │
│ 13 [ ] [ ] [ ]   [ ] [ ] [ ]                              │
│ 14 [ ] [💰] [💰]   [💰] [💰] [ ]  Premium ($20 upgrade)       │
│                                                             │
│ Selected: 12A (Extra Legroom) ✓                            │
│                                                             │
│ Passenger 2: Jane Doe                                       │
│ Selected: 12B (Extra Legroom) ✓                            │
│                                                             │
│ [Confirm Seat Selection]                                    │
└─────────────────────────────────────────────────────────────┘
```

#### Meal Selection Modal

```
┌─────────────────────────────────────────────────────────────┐
│ Select Meals for AA100 (JFK → LHR)                         │
├─────────────────────────────────────────────────────────────┤
│ Passenger 1: John Doe                                       │
│                                                             │
│ ( ) Chicken with Rice                                      │
│ (●) Vegetarian Pasta  ← Selected                           │
│ ( ) Beef with Potatoes                                     │
│ ( ) Vegan Salad                                            │
│ ( ) No meal preference                                     │
│                                                             │
│ Passenger 2: Jane Doe                                       │
│ (●) Chicken with Rice  ← Selected                          │
│                                                             │
│ [Confirm Meal Selection]                                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Step 3: API Flow for Customization

### 1. Accept Offer (Creates Order)

```bash
POST /v1/offers/offer-124/accept
{
  "customer_email": "user@example.com",
  "passengers": [
    {"first_name": "John", "last_name": "Doe"},
    {"first_name": "Jane", "last_name": "Doe"}
  ]
}
```

**Response**:
```json
{
  "order_id": "order-789",
  "status": "PROPOSED",
  "total_nuc": 81000,
  "items": [
    {
      "id": "item-1",
      "type": "FLIGHT",
      "product_id": "flight-aa100",
      "status": "ACTIVE"
    },
    {
      "id": "item-2",
      "type": "FLIGHT",
      "product_id": "flight-aa101",
      "status": "ACTIVE"
    },
    {
      "id": "item-3",
      "type": "SEAT",
      "product_id": "seat-extra-legroom",
      "quantity": 2,
      "status": "ACTIVE",
      "customizable": true,
      "metadata": {
        "seat_numbers": null  // Not selected yet
      }
    },
    {
      "id": "item-4",
      "type": "MEAL",
      "product_id": "meal-hot",
      "quantity": 2,
      "status": "ACTIVE",
      "customizable": true,
      "metadata": {
        "meal_types": null  // Not selected yet
      }
    }
  ]
}
```

### 2. Customize Seat Selection

```bash
POST /v1/orders/order-789/items/item-3/customize
{
  "seat_assignments": [
    {
      "passenger_index": 0,
      "seat_number": "12A",
      "flight_id": "flight-aa100"
    },
    {
      "passenger_index": 1,
      "seat_number": "12B",
      "flight_id": "flight-aa100"
    }
  ]
}
```

**Response**:
```json
{
  "item_id": "item-3",
  "status": "ACTIVE",
  "metadata": {
    "seat_numbers": ["12A", "12B"]
  }
}
```

### 3. Customize Meal Selection

```bash
POST /v1/orders/order-789/items/item-4/customize
{
  "meal_assignments": [
    {
      "passenger_index": 0,
      "meal_type": "VEGETARIAN",
      "flight_id": "flight-aa100"
    },
    {
      "passenger_index": 1,
      "meal_type": "CHICKEN",
      "flight_id": "flight-aa100"
    }
  ]
}
```

### 4. Pay for Order

```bash
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
  "confirmation_code": "ALTIS-789",
  "fulfillment": [
    {
      "item_id": "item-1",
      "type": "FLIGHT",
      "barcode": "ALTIS-1717234567-A1B2C3D4",
      "seat": "12A",
      "meal": "VEGETARIAN"
    },
    {
      "item_id": "item-2",
      "type": "FLIGHT",
      "barcode": "ALTIS-1717234568-B2C3D4E5",
      "seat": "12B",
      "meal": "CHICKEN"
    }
  ]
}
```

---

## Alternative: "See All Flights" Option

If customer wants to see **all available flights** for a specific time:

```
┌─────────────────────────────────────────────────────────────┐
│ Don't see what you want?                                    │
│ [View All Flights for June 1st]                            │
└─────────────────────────────────────────────────────────────┘
```

Clicking this shows a **traditional flight list**:

```
JFK → LHR on June 1st

AA100  08:00 - 20:00  $200  [Build Custom Offer]
BA200  10:00 - 22:00  $220  [Build Custom Offer]
UA300  12:00 - 00:00  $180  [Build Custom Offer]
DL400  14:00 - 02:00  $190  [Build Custom Offer]
```

Customer can **build a custom offer** by selecting specific flights and ancillaries.

---

## Summary

| Question | Answer |
|----------|--------|
| **Multiple flights per day?** | Yes! Each offer includes specific flight times. Different offers = different flight times. |
| **How to select seat?** | After accepting offer, click "Choose Seats" to open seat map and select specific seats. |
| **How to select meal?** | After accepting offer, click "Choose Meals" to select meal type per passenger. |
| **Can I change my mind?** | Yes! Order is PROPOSED until payment. You can modify seats/meals or cancel. |
| **What if I want a different flight time?** | Select a different offer (each has different flight times) or click "View All Flights". |

**Key Insight**: The offer system **pre-bundles** the best combinations, but customers can still **customize details** (seats, meals) before paying.
