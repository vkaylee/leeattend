# Podman Workflow & Multi-Environment Setup

LeeAttend uses a layered Podman Compose architecture for fast development, reliable testing, and production deployments.

All container operations go through **`./run-tests.sh --docker=ENV CMD`**.

---

## 🏗 Architecture Overview

| Layer | File | Purpose |
|-------|------|---------|
| **Base/Prod** | `docker-compose.yml` | Core service definitions, networking, healthchecks |
| **Dev** | `docker-compose.dev.yml` | Volume mounts for hot-reload (cargo-watch, Vite HMR), debug logging |
| **Test** | `docker-compose.test.yml` | `tmpfs` RAM volumes, `sccache`, isolated test databases |
| **CI** | `docker-compose.ci.yml` | Generated at CI time — nullifies `tmpfs` to use GitHub Actions cache |

---

## 🐳 Multi-Stage Containerfiles

Each service (`apiserver`, `agent-main`, `webdashboard`) uses a single `Dockerfile` (Containerfile) with stages:

- `dev` — includes `cargo-watch`, local target dir
- `test` — includes `nextest`, Playwright dependencies, `sccache`
- `runtime` — minimal production binary image

---

## 🛠 Environment Management

Use `./run-tests.sh --docker=ENV CMD` for all environments.

### Development

```bash
./run-tests.sh --docker=dev up -d                # Start all services
./run-tests.sh --docker=dev down                 # Stop all services
./run-tests.sh --docker=dev build                # Rebuild all images
./run-tests.sh --docker=dev build apiserver      # Rebuild one service
./run-tests.sh --docker=dev logs -f              # Tail all logs
./run-tests.sh --docker=dev logs -f apiserver    # Tail one service
./run-tests.sh --docker=dev exec apiserver bash  # Open shell in container
./run-tests.sh --docker=dev exec apiserver cargo clippy
./run-tests.sh --docker=dev exec webdashboard npm install
```

### Test Environment

```bash
./run-tests.sh --docker=test up -d    # Start test environment manually
./run-tests.sh --docker=test build    # Rebuild test images
./run-tests.sh --docker=test ps       # Check service status
```

> [!NOTE]
> For running tests, Human users should prefer `./run-tests.sh --api` or `./run-tests.sh --all` which manage the test
> environment automatically. **AI Agents** should append `</dev/null` to prevent PTY hangs.

### Production

```bash
./run-tests.sh --docker=prod up -d    # Start production (base compose only)
./run-tests.sh --docker=prod ps       # Check status
./run-tests.sh --docker=prod down     # Stop
```

---

## ⚡ Build Performance & Caching

### 1. Rust Build Acceleration (`cargo-chef` & `sccache`)
- **`cargo-chef`**: `planner` → `cacher` → `builder` stages. As long as `Cargo.toml` doesn't change, the 300+ dependency crate compilation is fully skipped.
- **`sccache`**: Intercepts Rust compilation at both build-time and runtime, serving cached objects across runs.

### 2. Hybrid Cache Strategy (`tmpfs` vs CI)
- **Local (dev/test):** `docker-compose.test.yml` mounts `tmpfs` over `/app/target`. Protects SSD from heavy Rust compile I/O.
- **CI (GitHub Actions):** `docker-compose.ci.yml` nullifies `tmpfs` to allow `type=gha` buildx cache to persist compiled artifacts across CI runs.

### 3. Build Context Optimization
`.dockerignore` blocks `target/`, `node_modules/`, `.test_logs/` — the build context gets only kilobytes of source instead of gigabytes.

### 4. Cache Management

```bash
./run-tests.sh --warm-cache     # Pre-compile deps (first run / CI setup)
./run-tests.sh --cache-stats    # Show cache sizes
./run-tests.sh --clean-cache    # Wipe cache, keep containers
./run-tests.sh --clean          # Full reset: containers + volumes + cache
```

---

## 🧪 Integration with Test Runner

`./run-tests.sh` automatically composes the correct files and uses `test` stage containers:

```bash
./run-tests.sh --all            # Full test pipeline (manages Docker automatically)
./run-tests.sh --api --smoke    # Lint + unit only (fast)
./run-tests.sh --logs           # Read test output from container volume
```

See `./run-tests.sh --help` for the complete reference.
