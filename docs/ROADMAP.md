# Altis Engine Roadmap

This document outlines the high-level roadmap for evolving Altis Engine from a functional prototype into a production-grade Airline Retailing Platform, aligned with IATA's "Modern Retailing" vision.

## 1. Core Order Management (Priority: High)
*Focus: Completing the NDC/ONE Order lifecycle capabilities.*

- [ ] **Split Order Logic**: 
    - Implement `POST /v1/orders/:id/split` to divide passengers into new orders (e.g., when one passenger changes their flight).
    - Handle secure transfer of inventory ownership and payments.
- [ ] **Real Ticketing & EMDs**:
    - Move beyond UUID barcodes to generate IATA-standard 13-digit Ticket Numbers (ETKT).
    - Implement Electronic Miscellaneous Documents (EMD-A/EMD-S) for ancillary services.
- [ ] **Order Customization**:
    - Flesh out the `customize_order` endpoint to actually persist seat and meal selections.
    - Dynamically recalculate order totals and trigger unnecessary re-pricing.

## 2. Payments & Finance (Priority: High)
*Focus: Real money handling and financial reconciliation.*

- [ ] **Payment Gateway Integration**:
    - Replace `MockPaymentAdapter` with a real SDK (Stripe, Adyen, or Worldpay).
    - Implement **3D Secure (3DS)** redirect flows within the `pay_order` orchestration.
- [ ] **Refunds & Cancellations**:
    - Implement `POST /v1/orders/:id/refund` for partial and full refunds.
    - Integrate with the financial ledger to issue Credit Notes and update revenue recognition status.

## 3. Shopping & Pricing (Priority: Medium)
*Focus: Enhanced retailing capabilities.*

- [ ] **Advanced Search Contexts**:
    - Support **Multi-City** and **Open-Jaw** itineraries in `SearchContext`.
    - Implement calendar-based search (fleixble dates).
- [ ] **Tax Engine**:
    - Introduce a dedicated tax calculation engine for YQ/YR, Government Taxes, and Airport Fees.
    - Break down `NdcPrice` into Base Fare + Taxes for transparent display.

## 4. Infrastructure & Operations (Priority: Medium)
*Focus: Reliability and scale.*

- [ ] **Expiration Worker**:
    - Implement a dedicated background worker (e.g., via `tokio-cron-scheduler`) to actively release inventory for expired `PROPOSED` orders.
- [ ] **Notification Service**:
    - Integrate an email/SMS provider (SendGrid/Twilio) to send:
        - Booking Confirmations (with PDF receipt).
        - Flight Status Updates.
        - Check-in Reminders.
