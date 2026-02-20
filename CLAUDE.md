> [!CAUTION]
> **CORE DIRECTIVES (NON-NEGOTIABLE):**
> 1. **SSOT:** [PROJECT_OS.md](./PROJECT_OS.md) is the absolute authority. Read it before ANY action.
> 2. **DOCKER ONLY:** NEVER run `cargo`, `npm`, or `pytest` on the host. Use `./scripts/docker-helper.sh`.
> 3. **SSD SAFETY:** Use `tmpfs` mounts via Docker to prevent WSL2 freezes.
> 4. **MODULARITY:** Hard limit of 200 lines per file. Split immediately.

## 🧭 Discovery Paths
- **Full Rules:** [PROJECT_OS.md](./PROJECT_OS.md)
- **Architecture:** [docs/system-architecture.md](./docs/system-architecture.md)
- **Docker/Layers:** [docs/docker-workflow.md](./docs/docker-workflow.md)
- **AI Index:** [llms.txt](./llms.txt)

## 🛠️ Stack Summary
- **Backend:** Rust (Axum, SQLx/Postgres, Redis)
- **Frontend:** React 19 (Vite, TS, Tailwind V4)
- **Edge:** Rust (Device Orchestration) + Python (Device Drivers)
