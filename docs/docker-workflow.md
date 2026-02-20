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

## ⚡ Build Performance & Caching

### Rust (apiserver & agent-main)
We use `cargo-chef` to pre-compile dependencies in a separate layer. This means if you only change your source code, Docker only rebuilds the tiny final stage, reducing rebuild times from minutes to seconds.

### Node.js (webdashboard)
The `base` stage handles `npm install`. The `dev` stage mounts your local folder, so changes you make on your host are immediately reflected in the container via Vite's HMR.

## 🧪 Integration with Test Runner
The `./run-tests.sh` script is fully integrated with this architecture. It automatically layers the Compose files and uses the `test` stage containers to execute linting, unit tests, and integration tests.
