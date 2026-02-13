# Builder stage
FROM rust:bookworm AS dev
WORKDIR /app
RUN apt-get update && apt-get install -y cmake build-essential protobuf-compiler && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-watch
COPY . .
CMD ["cargo", "watch", "-x", "run"]

FROM rust:bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y cmake build-essential protobuf-compiler && rm -rf /var/lib/apt/lists/*


# Copy the entire workspace
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
