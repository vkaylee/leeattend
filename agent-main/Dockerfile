# syntax=docker/dockerfile:1
# Multi-stage build for Rust Agent
FROM rust:latest AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y build-essential pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build with BuildKit cache mounts for cargo registry + target dir
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/agent /usr/local/bin/agent

# Final runtime stage
FROM debian:bookworm-slim

WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates openssl libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/agent /usr/local/bin/agent

CMD ["agent"]
