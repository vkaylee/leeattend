# Multi-stage build for Rust Agent supporting Dev, Test, and Prod

# Stage 1: Cargo Chef base
FROM docker.io/blackdex/rust-musl:x86_64-musl-stable AS chef
RUN rustup component add clippy rustfmt || true
RUN cargo install cargo-chef --version ^0.1.77 --locked
RUN apt-get update && apt-get install -y wget rsync && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Stage 2: Planner (Prepare recipe)
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Cacher (Compile dependencies)
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
ENV SCCACHE_DIR=/sccache
ENV RUSTC_WRAPPER=sccache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/sccache \
    sccache --start-server && \
    cargo chef cook --release --recipe-path recipe.json && \
    sccache --stop-server

# Stage 4: Dev (for hot-reloading)
FROM public.ecr.aws/docker/library/rust:1.93-bookworm AS dev
RUN apt-get update && apt-get install -y clang mold git redis-tools postgresql-client sqlite3 netcat-traditional curl dnsutils iputils-ping iproute2 nmap jq htop bash && \
    rm -rf /var/lib/apt/lists/* && \
    rustup component add clippy && \
    cargo install cargo-watch
WORKDIR /app
CMD ["cargo", "watch", "-x", "run"]

# Stage 5: Test (for running tests)
FROM chef AS test
ENV SCCACHE_DIR=/sccache
RUN curl -LsSf https://get.nexte.st/0.9.133/linux | tar zxf - -C /usr/local/bin
# Coverage tooling: llvm-cov requires llvm-tools-preview component + cargo-llvm-cov binary
RUN rustup component add llvm-tools-preview && \
    cargo install cargo-llvm-cov --locked
COPY --from=planner /app/recipe.json recipe.json
ENV RUSTC_WRAPPER=sccache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/sccache \
    sccache --start-server && \
    cargo chef cook --tests --recipe-path recipe.json && \
    sccache --stop-server
COPY . .
CMD ["cargo", "nextest", "run"]

# Stage 6: Builder (Prod build)
FROM chef AS builder
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY . .
ENV SCCACHE_DIR=/sccache
ENV RUSTC_WRAPPER=sccache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/sccache \
    sccache --start-server && \
    cargo build --release --locked -p agent && \
    sccache --stop-server && \
    cp target/x86_64-unknown-linux-musl/release/agent /usr/local/bin/agent && \
    mkdir -p /tmp/data && chown 65532:65532 /tmp/data

# Stage 7: Runtime (Production image)
FROM gcr.io/distroless/static-debian12 AS runtime
WORKDIR /app
COPY --from=builder /usr/local/bin/agent /usr/local/bin/agent
COPY --chown=65532:65532 --from=builder /tmp/data /app/data
USER 65532
VOLUME ["/app/data"]
CMD ["agent"]
