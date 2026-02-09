# Multi-stage build for Rust API Server
FROM rust:1.81-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace metadata if any, or just the component
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build with optimizations
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Final runtime stage
FROM debian:bookworm-slim

WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/apiserver /usr/local/bin/apiserver
COPY --from=builder /app/migrations ./migrations

EXPOSE 3000

CMD ["apiserver"]
