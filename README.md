# Altis Engine: The Future of Airline Retailing

> **A Cloud-Native, Offer/Order Engine for Modern Airline Commerce.**  
> Built in Rust for safety and performance. Natively aligned with the **IATA Modern Retailing** vision.

---

## 🌐 The Renaissance of Airline Distribution

For decades, airline commerce has been constrained by legacy PNR (Passenger Name Record) and E-Ticket architectures built on 1960s logic. These systems create fragmented states, rigid pricing, and high operational costs.

**Altis Engine** is built on the premise of the [IATA Modern Retailing](https://www.iata.org/en/programs/airline-distribution/retailing/) "End State": **A unified Offer and Order world.**

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
Native support for [IATA One Identity](https://www.iata.org/en/programs/passenger/one-id/) standards using Decentralized Identifiers (DIDs) and Verifiable Credentials (VCs). Enabling "Travel-by-Face" and personalized retailing without compromising privacy.

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

## 🛡️ Security & Privacy by Design

Altis is built for the high-stakes environment of global aviation, where trust is non-negotiable.

### 1. The Rust Advantage (Memory Safety)
By using Rust, Altis eliminates entire classes of security vulnerabilities that plague legacy C/C++ systems, such as **Buffer Overflows** and **Memory Leaks**. This ensures a robust, exploit-resistant core.

### 2. Privacy-First Identity
Aligned with **IATA One Identity**, we leverage Decentralized Identifiers (DIDs). Travelers maintain control over their data, and Altis only processes the minimum necessary Verifiable Credentials to authorize travel, reducing PII surface area.

### 3. Stateless API Security
Altis implements modern **JWT-based Authentication** across all retailing operations. Combined with built-in **Circuit Breakers** and **Rate Limiting**, the engine protects itself and its downstream suppliers from cascading failures and malicious traffic.

---

## 🚦 Getting Started

To explore the engine's capabilities:

1.  **Run with Docker**: `docker-compose up --build`
2.  **Developer Guide**: See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for API usage and setup.
3.  **Architecture Details**: Dive into our [Architecture Overview](docs/architecture/OVERVIEW.md).
4.  **Roadmap**: Check out [ROADMAP.md](docs/ROADMAP.md) for the high-level project backlog.

---

*"Just as Shopify transformed e-commerce, Altis aims to transform airline retailing by making the IATA vision a production reality."*

---

### 💡 Fun Fact: Why "Altis"?

The name **Altis** carries a dual legacy:
1.  **Etymology**: It is rooted in the Latin *Altus*, meaning "high," "deep," or "noble"—a perfect nod to the **high-altitude** world of aviation and the depth of our technical architecture.
2.  **Mythology**: It is inspired by the *sacred grove* in Olympia, Greece—the heart of the ancient Olympic Games. Just as the Altis was the central precinct where competitors gathered to strive for excellence, the Altis Engine serves as the core "sacred precinct" of an airline's digital ecosystem, where the legacy of PNRs is replaced by the excellence of modern Offer/Order retailing.

**Built with ❤️ in Rust**
