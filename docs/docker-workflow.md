# Docker Workflow & Multi-Environment Setup

LeeAttend uses a highly optimized, layered Docker architecture to ensure fast development, reliable testing, and secure production deployments.

## 🏗 Architecture Overview

Our Docker setup is split into three layers:

1.  **Base Layer (`docker-compose.yml`)**: Contains the fundamental service definitions, network setup, and default healthchecks. This corresponds to the **Production** environment.
2.  **Dev Layer (`docker-compose.dev.yml`)**: Adds volume mounts for hot-reloading (`cargo-watch`, Vite dev server), debug logging, and dev-friendly database settings.
3.  **Test Layer (`docker-compose.test.yml`)**: Optimized for CI/automation. Uses `tmpfs` for lightning-fast database I/O, enables `sccache` for Rust, and sets up isolated test databases.

## 🐳 Multi-Stage Dockerfiles

Each service (`apiserver`, `agent-main`, `webdashboard`) uses a single multi-stage `Dockerfile` with these targets:

-   `dev`: Includes development tools like `cargo-watch` and uses the standard local target.
-   `test`: Includes test runners like `nextest` and Playwright dependencies.
-   `runtime`: (Default) The minimal production image containing only the final binary and essential system libraries.

## 🛠 Using the Docker Helper

We provide a helper script at `./scripts/docker-helper.sh` to manage environment switching easily.

### Development
```bash
./scripts/docker-helper.sh dev up -d    # Start everything
./scripts/docker-helper.sh dev build    # Rebuild all (uses cache)
./scripts/docker-helper.sh dev logs -f  # Tail logs
```

### Testing
```bash
./scripts/docker-helper.sh test up -d   # Start the test environment
./scripts/docker-helper.sh test build   # Rebuild test stages
```

### Production
```bash
./scripts/docker-helper.sh prod up -d   # Equivalent to using only docker-compose.yml
```

### ⚡ Build Performance & Advanced Caching

We employ several advanced strategies to achieve minimal build times both locally and on CI:

#### 1. Rust Build Acceleration (`cargo-chef` & `sccache`)
- **Dependency Caching:** We use `cargo-chef` in a multi-stage process (`planner`, `cacher`, `builder`). Dependencies are fully compiled in the `cacher` stage. As long as `Cargo.toml` doesn't change, Docker completely skips downloading and recompiling the 300+ libraries.
- **Layer Optimization:** Environment setup layers (like `apt-get` and downloading the `sccache` binary) are pushed to the absolute top (`chef` stage). Fast-changing layers (source code `COPY`) are kept at the absolute bottom.
- **`sccache` Integration:** We inject `sccache` (Mozilla's compiler cache) directly into both the Docker build stages AND the `docker-compose` runtime. This ensures that even when dependencies *do* change, `sccache` intercepts the compilation and serving previously compiled artifacts, minimizing the penalty.

#### 2. The Hybrid Run-Time Cache Strategy (`tmpfs` vs CI)
There is an inherent conflict between protecting local SSDs and maximizing CI caching:
- **Local Dev/Test:** `docker-compose.test.yml` mounts massive `tmpfs` (RAM disk) volumes over `/app/target`. This prevents Docker from wearing out your physical SSD when you run `cargo test` hundreds of times a day.
- **GitHub Actions (CI):** `tmpfs` is counter-productive in ephemeral CI runners because it overwrites the pre-built `cargo-chef` cache with a blank RAM drive, forcing a full 10-minute recompile. Therefore, our CI workflow `.github/workflows/manual-tests.yml` intentionally generates a `docker-compose.ci.yml` override file. This file nullifies the `tmpfs` mounts, coercing the CI to use the blazingly fast `type=gha` Docker Buildx cache baked into the hard drive. 

#### 3. Build Context Optimization (`.dockerignore`)
Every service maintains a strict `.dockerignore` file blocking `target/`, `node_modules/`, and `.test_logs/`. This guarantees the Docker daemon only receives kilobyte-sized source code payloads instead of gigabytes of metadata, making the build initialization instant.

#### 4. Node.js (webdashboard)
The `base` stage handles `npm install`. The `dev` stage mounts your local folder, so changes you make on your host are immediately reflected in the container via Vite's HMR.

## 🧪 Integration with Test Runner
The `./run-tests.sh` script is fully integrated with this architecture. It automatically layers the Compose files and uses the `test` stage containers to execute linting, unit tests, and integration tests.
