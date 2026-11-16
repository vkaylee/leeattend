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

1. **`apiserver`**: Go 1.26+ (`net/http`, `pgx/v5`, `sqlc`) API for attendance, payroll, files, monitoring, and dashboard access.
2. **`webdashboard`**: React, Vite, and TypeScript web interface for system management and monitoring.
3. **`docs`**: Detailed guidance and operational documentation. LeeAttend runtime uses Go plus React/Vite/TypeScript; retired implementation archives are not part of active project context.


## 🤖 For AI Agents & Developers

> **Current project context:** The active application runtime is Go plus React/Vite/TypeScript. Follow `AGENTS.md` and `.agent/rules/`; use `./leedevkit test` for verification.

### Quick Start (Dev Mode)
```bash
./leedevkit manage up dev
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
- **For humans:**
  ```bash
  ./leedevkit test all             # Full Go + web regression suite
  ./leedevkit test api [--pattern PATTERN]
  ./leedevkit test web [--pattern PATTERN]
  ```
- **For AI agents:** append `</dev/null` to commands that can block.

### Key Features
- **Hermetic Go gates:** formatting, vet, static analysis, vulnerability checks, module verification, and Go tests.
- **Web gates:** Bun unit tests, TypeScript checks, linting, i18n checks, and Playwright E2E tests.
- **Docker isolation:** All tests run inside containers with isolated database and dependency volumes.

### WSL Performance Note
Use `./leedevkit test` instead of direct host dependency commands. The orchestrator runs the Go and web checks inside Podman containers.

## 📝 Changelog
See [CHANGELOG.md](./CHANGELOG.md) for version history.