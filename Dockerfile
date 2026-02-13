# Builder stage
FROM rust:bookworm AS dev
WORKDIR /app
RUN apt-get update && apt-get install -y cmake build-essential protobuf-compiler && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-watch
RUN cargo install sqlx-cli --no-default-features --features postgres
COPY . .
CMD ["cargo", "watch", "-x", "run"]

# Build stage
FROM rust:1.84-slim-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    git \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Pre-fetch dependencies (caching layer)
COPY Cargo.toml Cargo.lock ./
COPY altis-core/Cargo.toml altis-core/
COPY altis-api/Cargo.toml altis-api/
COPY altis-catalog/Cargo.toml altis-catalog/
COPY altis-offer/Cargo.toml altis-offer/
COPY altis-order/Cargo.toml altis-order/
COPY altis-shared/Cargo.toml altis-shared/
COPY altis-store/Cargo.toml altis-store/

# Create dummy source files for dependency caching
RUN mkdir -p altis-core/src altis-api/src altis-catalog/src altis-offer/src altis-order/src altis-shared/src altis-store/src \
    && echo "fn main() {}" > altis-api/src/main.rs \
    && touch altis-core/src/lib.rs altis-catalog/src/lib.rs altis-offer/src/lib.rs altis-order/src/lib.rs altis-shared/src/lib.rs altis-store/src/lib.rs

RUN cargo fetch

# Copy actual source and workspace config
COPY . .

# Build the application
RUN cargo build --release --bin altis-api

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /app/target/release/altis-api /app/altis-api

# Copy configuration files
COPY config /app/config

# Set environment variables
ENV RUST_LOG=info
ENV ALTIS__SERVER__PORT=8080

EXPOSE 8080

CMD ["./altis-api"]
