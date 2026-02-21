# syntax=docker/dockerfile:1
# Multi-stage build for Rust API Server supporting Dev, Test, and Prod

# Stage 1: Cargo Chef base
FROM rust:1.88-slim-bookworm AS chef
RUN cargo install cargo-chef
RUN apt-get update && apt-get install -y build-essential pkg-config libssl-dev clang mold curl tar postgresql-client wget && rm -rf /var/lib/apt/lists/*
RUN wget https://github.com/mozilla/sccache/releases/download/v0.8.1/sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz && \
    tar xzf sccache-v0.8.1-x86_64-unknown-linux-musl.tar.gz && \
    mv sccache-v0.8.1-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache && \
    chmod +x /usr/local/bin/sccache && \
    rm -rf sccache*
WORKDIR /app

# Stage 2: Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Cacher
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
ENV SCCACHE_DIR=/sccache
ENV RUSTC_WRAPPER=sccache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/sccache \
    cargo chef cook --release --recipe-path recipe.json

# Stage 4: Dev
FROM rust:1.88-bookworm AS dev
RUN apt-get update && apt-get install -y clang mold git redis-tools postgresql-client sqlite3 netcat-traditional curl dnsutils iputils-ping iproute2 nmap jq htop bash && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch
WORKDIR /app
CMD ["cargo", "watch", "-x", "run"]

# Stage 5: Test
FROM chef AS test
RUN cargo install sqlx-cli --locked --no-default-features --features postgres
RUN curl -LsSf https://get.nexte.st/0.9.96/linux | tar zxf - -C /usr/local/bin
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --tests --recipe-path recipe.json
COPY . .
CMD ["cargo", "nextest", "run"]

# Stage 6: Builder
FROM chef AS builder
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY . .
ENV SQLX_OFFLINE=true
ENV SCCACHE_DIR=/sccache
ENV RUSTC_WRAPPER=sccache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/sccache \
    cargo build --release --locked && \
    cp target/release/apiserver /usr/local/bin/apiserver

# Stage 7: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 postgresql-client && rm -rf /var/lib/apt/lists/*
RUN useradd -r -u 1001 -s /usr/sbin/nologin appuser
COPY --from=builder /usr/local/bin/apiserver /usr/local/bin/apiserver
COPY migrations ./migrations
RUN chown -R appuser:appuser /app
USER appuser
EXPOSE 3000
CMD ["apiserver"]
