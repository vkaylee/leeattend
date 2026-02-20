# LeeAttend - AI Context Header

> [!CAUTION]
> **CORE DIRECTIVES:**
> 1. **DOCKER ONLY:** NEVER run `cargo`, `npm`, or `pytest` on the host. Use `./scripts/docker-helper.sh`.
> 2. **SSD SAFETY:** Always use `tmpfs` mounts via Docker for builds/tests to prevent WSL2 freezes.
> 3. **SSOT:** Read `PROJECT_OS.md` before ANY implementation.
> 4. **MODULARITY:** Files MUST be < 200 lines. Split immediately if exceeded.

## 🧭 Discovery Paths
- **Full Rules:** [PROJECT_OS.md](./PROJECT_OS.md)
- **Architecture:** [docs/system-architecture.md](./docs/system-architecture.md)
- **Docker/Layers:** [docs/docker-workflow.md](./docs/docker-workflow.md)
- **AI Index:** [llms.txt](./llms.txt)

## 🛠️ Stack Summary
- **Backend:** Rust (Axum, SQLx/Postgres, Redis)
- **Frontend:** React 19 (Vite, TS, Tailwind V4)
- **Edge:** Rust (Device Orchestration) + Python (Device Drivers)
