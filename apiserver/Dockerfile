# syntax=docker/dockerfile:1
# Multi-stage build for Rust API Server
FROM rust:1.85 AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build with BuildKit cache mounts for cargo registry + target dir
# Dependencies are cached across builds → only changed source recompiles
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/apiserver /usr/local/bin/apiserver

# Final runtime stage
FROM debian:bookworm-slim

WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
RUN useradd -r -u 1001 -s /usr/sbin/nologin appuser

COPY --from=builder /usr/local/bin/apiserver /usr/local/bin/apiserver
COPY --from=builder /app/migrations ./migrations

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 3000

CMD ["apiserver"]
