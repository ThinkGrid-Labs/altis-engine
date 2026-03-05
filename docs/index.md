---
layout: home

hero:
  name: Altis Engine
  text: IATA Offer/Order Engine for Modern Airline Retailing
  tagline: Cloud-native, built in Rust. NDC v21.3, ONE Order, continuous pricing, and IATA One Identity — replacing legacy PNR/E-Ticket systems with a unified Offer and Order world.
  actions:
    - theme: brand
      text: Get Started
      link: /DEVELOPMENT
    - theme: alt
      text: Architecture
      link: /architecture/OVERVIEW
    - theme: alt
      text: View on GitHub
      link: https://github.com/ThinkGrid-Labs/altis-engine

features:
  - title: Dynamic Merchandising Engine
    details: Real-time AI ranking prioritizes bundles by conversion probability and profit margin. Continuous pricing enables micro-adjustments without fare class jumps.
  - title: ONE Order Native Lifecycle
    details: From PROPOSED to FULFILLED, the entire lifecycle lives in a single atomic Order record — eliminating synchronization hell between PNRs, Tickets, and DCS systems.
  - title: IATA Standards Aligned
    details: NDC v21.3 for offer distribution, ONE Order for lifecycle management, Settlement with Orders (SwO) for financial reporting.
  - title: IATA One Identity
    details: Native Decentralized Identifiers (DIDs) and Verifiable Credentials (VCs) — enabling Travel-by-Face and personalized retailing without compromising privacy.
  - title: Rust Performance & Safety
    details: Memory-safe Rust eliminates buffer overflows and memory leaks. Parallel processing via rayon. Axum-based async HTTP layer with circuit breakers and rate limiting.
  - title: Modular Workspace Architecture
    details: Clean hexagonal design — altis-core, altis-offer, altis-order, altis-catalog, altis-store, and altis-api are independently testable, composable crates.
---

## Roadmap

Altis is evolving from a functional prototype into a production-grade airline retailing platform. Below is the high-level backlog — [full detail here](/ROADMAP).

### High Priority

| Item | Description |
|---|---|
| **Split Order Logic** | `POST /v1/orders/:id/split` — divide passengers into separate orders with inventory transfer |
| **Real Ticketing & EMDs** | IATA-standard 13-digit ETKTs and Electronic Miscellaneous Documents (EMD-A/EMD-S) |
| **Order Customization** | Persist seat and meal selections; dynamically recalculate order totals |
| **Real-time Ancillary Inventory** | SSE-based live availability for Meals, Bags, Wi-Fi with race condition handling |
| **Payment Gateway** | Replace `MockPaymentAdapter` with Stripe/Adyen/Worldpay + 3D Secure flows |
| **Refunds & Cancellations** | `POST /v1/orders/:id/refund` with Credit Notes and revenue recognition |

### Medium Priority

| Item | Description |
|---|---|
| **Multi-City / Open-Jaw Search** | Extended `SearchContext` for complex itineraries and flexible date search |
| **Tax Engine** | Dedicated YQ/YR, government tax, and airport fee calculation with itemized breakdown |
| **Expiration Worker** | Background tokio worker to release inventory for expired `PROPOSED` orders |
| **Notification Service** | SendGrid/Twilio integration for booking confirmations, flight updates, check-in reminders |

### Low Priority

| Item | Description |
|---|---|
| **Product Catalog UI** | Admin dashboard for Ancillaries, pricing, and rich media uploads |
| **Inventory Control** | Manual overrides for flight capacity and ancillary stock limits |
