# LeeAttend - Attendance Management System

A comprehensive attendance management system with central API server and web dashboard.

## 📂 Project Structure

```
leeattend/
├── apiserver/        # Go API Server
│   ├── REST API and database layer
└── docs/             # System Documentation
```

## Project Components

1.  **apiserver**: Built with **Go** (`net/http`, `pgx/v5`, `sqlc`), responsible for attendance management, log ingestion, and providing API for the Dashboard. Migrated from Rust/Axum in October 2026 — see `docs/migration/rust-go/final-evidence.md`.
2.  **webdashboard**: Built with **React (Vite)**, a web interface for system management and monitoring.
3.  **docs**: Detailed guidance and documentation.


## 🤖 For AI Agents & Developers

> [!IMPORTANT]
> **MANDATORY RULES:** Before running any commands or starting implementation, you MUST read:
> 👉 **[PROJECT_OS.md](./PROJECT_OS.md)**
>
> This file contains the mandatory technical protocols, build/test commands (via `test.sh`), and the Definition of Done required for this project.

### Quick Start (Dev Mode)
To spin up the entire environment with hot-reloading:
```bash
./manage.sh up
```


## 📖 Documentation
- [System Architecture](docs/system-architecture.md)
- [API Reference](docs/api-reference.md)
- [Deployment Guide](docs/deployment-guide.md)


## Standalone On-Prem Deployment

Use [`docker-compose.standalone.yml`](./docker-compose.standalone.yml) for the approved one-company Linux appliance deployment. Installation is documented in [`docs/standalone-deployment.md`](docs/standalone-deployment.md); encrypted backup and restore are documented in [`docs/runbooks/standalone-backup-restore.md`](docs/runbooks/standalone-backup-restore.md).
## 🧪 Testing

The project uses a highly optimized testing infrastructure designed for speed and reliability, especially on WSL2.

### Prerequisites
- Docker & Docker Compose
- Go 1.26+ (for local host-side Go builds; not required when using `./leedevkit`)

### Running Tests
Always use the unified orchestrator:
- **For Humans**:
  ```bash
  ./leedevkit test --all             # Run full suite
  ./leedevkit test --api [pattern]   # Run backend tests (cargo nextest)
  ./leedevkit test --web [pattern]   # Run frontend & E2E tests (playwright)
  ```
- **For AI Agents (IMPORTANT: prevents PTY hangs)**:
  ```bash
  ./leedevkit test --all </dev/null             # Run full suite
  ./leedevkit test --api [pattern] </dev/null   # Run backend tests
  ./leedevkit test --web [pattern] </dev/null   # Run frontend tests
  ```

### Key Features
- **Smart Worker Pool**: High-efficiency orchestrator that strictly respects `NUM_WORKERS` limits (auto-capped at 2 for WSL stability).
- **Comprehensive Summary Report**: Generates a detailed, colorized report at the end of every run with per-task status and duration.
- **Cargo Nextest & Go Test Harness**: Isolated test runners for backend Go API and Rust agent tests.
- **Docker Isolation**: All tests run inside containers. Database is internal (`test_db:5432`) with no host port mapping.
### ⚠️ WSL Performance Note
On WSL2, direct test execution (`cargo test`) will likely freeze your system due to Disk I/O issues. **Always use `./leedevkit test`** which runs tests inside Podman containers. AI Agents should append `</dev/null`.

## 📝 Changelog
See [CHANGELOG.md](./CHANGELOG.md) for version history.