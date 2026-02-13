# Customer API Guide

This guide explains how to interact with the Altis Engine as a client (e.g., a travel agency or airline frontend).

## 🌍 Base URL

The API is available at: `http://localhost:8080` (Default)

---

## 🔑 Authentication

Most endpoints require a JSON Web Token (JWT).

### 1. Get a Guest Token
Start your session by obtaining a temporary token.
```bash
curl -X POST http://localhost:8080/v1/auth/guest \
  -H "Content-Type: application/json"
# Returns: {"token": "..."}
```

### 2. Using the Token
Include the token in the `Authorization` header for all protected requests:
`Authorization: Bearer <your_token>`

---

## 🛒 The Retailing Journey

### 1. Search for Offers
Search for flights and dynamic bundles.
```bash
curl -X POST http://localhost:8080/v1/offers/search \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "origin": "SIN",
    "destination": "KUL",
    "departure_date": "2024-06-01",
    "passengers": 1
  }'
```

### 2. Accept an Offer
Create a `PROPOSED` order from an offer.
```bash
curl -X POST http://localhost:8080/v1/offers/{offer_id}/accept \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "customer_email": "traveler@example.com"
  }'
```

### 3. Complete Payment
Finalize the order and generate fulfillment.
```bash
curl -X POST http://localhost:8080/v1/orders/{order_id}/pay \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "payment_token": "tok_mock_success",
    "payment_reference": "ref_123"
  }'
```

### 4. Retrieve Order
View order status and fulfillment barcodes.
```bash
curl http://localhost:8080/v1/orders/{order_id} \
  -H "Authorization: Bearer {token}"
```

---

## 🆘 Industry Standards Support

### NDC AirShopping (v21.3)
```bash
curl -X POST http://localhost:8080/v1/ndc/airshopping \
  -H "Content-Type: application/json" \
  -d '{ ... NDC XML-mapped JSON ... }'
```

### ONE Order Retrieve
```bash
curl -X POST http://localhost:8080/v1/oneorder/retrieve \
  -H "Content-Type: application/json" \
  -d '{"order_id": "..."}'
```
