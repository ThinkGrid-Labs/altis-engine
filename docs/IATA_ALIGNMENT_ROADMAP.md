# IATA Airline Retailing Alignment & Roadmap

This report evaluates the **Altis Engine** against the IATA vision for a modern airline retailing system based on **Offers and Orders**.

## 🏗️ Executive Summary

Altis Engine is **strongly aligned** with the core tenets of IATA's vision. Unlike legacy PSS systems trying to retrofit modern APIs, Altis is built as a native **Offer/Order Management System (OMS)**. 

The fundamental architectural choice to use a single `Order` record as the source of truth (replacing PNR, ET, and EMD) puts Altis at the "End State" of the IATA technology roadmap. However, there are functional gaps in complex servicing, financial settlement, and interline capabilities that need to be addressed in upcoming phases.

---

## 🔍 Alignment Analysis

### 1. Unified Offer/Order Architecture
*   **IATA Vision**: A single integrated customer record (ONE Order) to streamline distribution, delivery, and accounting.
*   **Altis Status**: ✅ **Fully Aligned**.
*   **Analysis**: Altis already implements a robust `Order` model that combines flights, meals, seats, and fulfillment (barcodes). It completely bypasses legacy PNR concepts.

### 2. Dynamic Offer Creation & Continuous Pricing
*   **IATA Vision**: Replace static fare filing and "booking buckets" with real-time, personalized offers and granular pricing.
*   **Altis Status**: 🟡 **Foundation Ready**.
*   **Analysis**: Altis implements "Continuous Pricing" based on utilization and time-based multipliers (cent-level adjustments). However, the "Offer Generation" logic currently uses hardcoded strategies rather than a truly dynamic, rule-based engine.

### 3. Servicing & Change Management
*   **IATA Vision**: Simplified "zero-cost" modifications and seamless handling of voluntary/involuntary changes.
*   **Altis Status**: 🟡 **Partial Implementation**.
*   **Analysis**: Altis supports "voluntary" changes (Change Flight) via a refund-and-add mechanism. It lacks logic for **involuntary changes** (disruptions), re-accommodation, and protection statuses.

### 4. Financial Settlement & Accounting
*   **IATA Vision**: Use Orders for real-time revenue recognition and simplified settlement (reducing the need for proration).
*   **Altis Status**: 🔴 **Experimental/Missing**.
*   **Analysis**: Altis has a `Paid` status but lacks a dedicated financial ledger or settlement module to handle revenue accounting or IATA-compliant reporting (BSP/ARC).

---

## 🗺️ Next Phases of Development

### Phase 1: Dynamic Merchandising Engine
**Objective**: Transition from static bundles to data-driven "Hyper-Retailing."
*   **Rule-Based Offers**: Implement an `OfferRules` engine allowing commercial teams to set bundling logic (e.g., "Always bundle Lounge for Gold frequent flyers").
*   **Willingness to Pay (WTP)**: Integrate user-segment-aware pricing multipliers into the `PricingEngine`.
*   **Rank Optimization**: Optimize offer ranking based on `conversion_probability × margin` (as noted in current placeholders).

### Phase 2: Disruption & Involuntary Servicing
**Objective**: Automate the handling of "Involuntary Changes" (cancellations/delays).
*   **Disruption Manager**: Create a module to auto-reaccommodate passengers when flights are cancelled.
*   **Item Protection**: Add a `Protected` status to `OrderItem` to hold seats during change negotiations.
*   **Refund Automation**: Automated calculation of refund balances for involuntary vs. voluntary scenarios.

### Phase 3: Settlement with Orders (Financials)
**Objective**: Fulfill the "ONE Order" promise of simplified finance.
*   **Order Ledger**: Implement a financial repository that tracks unearned vs. earned revenue based on consumption events.
*   **Real-time Revenue Recognition**: Emit accounting events (Kafka) whenever a barcode is scanned/consumed.
*   **IATA Settlement Adaptors**: Build reporting tools for IATA settlement standards (replacing old RET/HOT files).

### Phase 4: Retailer-Supplier Interline
**Objective**: Expand Altis into a multi-party marketplace.
*   **Supplier Interface**: Define a standard trait for external product providers (other airlines, ground transport, hotels).
*   **Multi-Party Orders**: Enable an Order to contain items from different operating carriers with status synchronization.
*   **Commission & Net-Rate Handling**: Logic for secondary settlement between the Retailer (Altis) and external Suppliers.

---

## 🚀 Conclusion

Altis is an excellent platform for airlines looking to bypass legacy baggage and leap-frog into **Full Scale Retailing**. By focusing on Phase 1 (Dynamic Rules) and Phase 2 (Disruptions) next, the engine will move from a "Clean Prototype" to a "Production-Ready Challenger" in the airline tech industry.
