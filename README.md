# Altis Engine: The Future of Airline Retailing

> **A Cloud-Native, Offer/Order Engine for Modern Airline Commerce.**  
> Built in Rust for safety and performance. Natively aligned with the **IATA Modern Retailing** vision.

---

## 🌐 The Renaissance of Airline Distribution

For decades, airline commerce has been constrained by legacy PNR (Passenger Name Record) and E-Ticket architectures built on 1960s logic. These systems create fragmented states, rigid pricing, and high operational costs.

**Altis Engine** is built on the premise of the [IATA Modern Retailing](https://www.iata.org/en/programs/distribution/modern-retailing/) "End State": **A unified Offer and Order world.**

### Why Altis? (Legacy vs. Modern Retailing)

| Feature | Legacy PSS (PNR/Ticket) | Altis Engine (Offer/Order) |
| :--- | :--- | :--- |
| **Data Model** | Fragmented PNRs, Tickets, EMDs | **Unified Order** (Single source of truth) |
| **Pricing** | Static 26-letter Fare Classes | **Continuous Pricing** (AI-driven margins) |
| **Merchandising** | Filed via 3rd parties (ATPCO) | **Dynamic Bundling** (Storefront-native) |
| **Servicing** | Manual revalidation / High friction | **Automated Re-accommodation** |
| **Identity** | Siloed airline accounts | **Decentralized One Identity** (DID/VC) |

---

## 🚀 Key Capabilities

### 1. Dynamic Merchandising Engine
Move beyond static shelves. Altis uses real-time AI ranking to prioritize bundles based on **Conversion Probability** and **Profit Margin**. pricing is continuous, allowing for micro-adjustments that maximize yield without "fare class jumps."

### 2. ONE Order Native Lifecycle
From `PROPOSED` to `FULFILLED`, the entire lifecycle resides in a single, atomic Order record. This eliminates the "Synchronization Hell" between PNRs, Tickets, and DCS systems, reducing servicing costs by up to 40%.

### 3. IATA One Identity Integration
Native support for [IATA One Identity](https://www.iata.org/en/programs/distribution/one-identity/) standards using Decentralized Identifiers (DIDs) and Verifiable Credentials (VCs). Enabling "Travel-by-Face" and personalized retailing without compromising privacy.

### 4. Modern Settlement with Orders (SwO)
Bypass legacy proration complexities. Altis implements **Settlement with Orders (SwO)**, providing real-time financial visibility and automated interline clearing directly from the Order record.

---

## 🏗️ Technical Blueprint

Altis is engineered as a high-performance **Modular Rust Workspace**, following Clean Architecture principles:

- **`altis-offer`**: AI-driven generator and rule engine.
- **`altis-order`**: State-machine based lifecycle and fulfillment.
- **`altis-catalog`**: Inventory and continuous pricing logic.
- **`altis-core`**: The IATA Domain Layer (Traits, Models, Protocols).
- **`altis-store`**: Pluggable persistence (PostgreSQL + Redis + Kafka).

---

## 🔗 Industry Standards Alignment

Altis is not just an app; it's a implementation of global airline standards:
- **IATA NDC v21.3**: Standardized Offer distribution.
- **IATA ONE Order**: Standardized Order retrieval and management.
- **IATA Settlement with Orders (SwO)**: Modernized financial reporting.
- **W3C Decentralized Identifiers (DID)**: Transforming traveler identity.

---

## 🚦 Getting Started

To explore the engine's capabilities:

1.  **Run with Docker**: `docker-compose up --build`
2.  **Developer Guide**: See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for API usage and setup.
3.  **Architecture Details**: Dive into our [Architecture Overview](docs/architecture/OVERVIEW.md).

---

*"Just as Shopify transformed e-commerce, Altis aims to transform airline retailing by making the IATA vision a production reality."*

**Built with ❤️ in Rust**
