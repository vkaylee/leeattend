# LeeAttend - Attendance Management System

A comprehensive attendance management system comprising a local agent for ZKTeco devices and a central high-performance API server.

## 📂 Project Structure

```
leeattend/
├── agent-main/       # Rust Agent Orchestrator + ZK Device Worker
│   └── Native Rust hardware communication via rustzk
│
├── apiserver/        # Rust API Server (Runs on Central Cloud/Server)
│   ├── High-performance REST API (Axum)
│   └── Central Database Management
│
└── docs/             # System Documentation
```

## Project Components

1.  **apiserver**: Built with **Rust (Axum)**, responsible for agent management, log ingestion, and providing API for the Dashboard.
2.  **agent-main**: Built with **Rust**, acts as the Agent Orchestrator + ZK Worker, communicating directly with ZKTeco devices via `rustzk`.
3.  **webdashboard**: Built with **React (Vite)**, a web interface for system management and monitoring.
4.  **docs**: Detailed guidance and documentation.


## 🤖 For AI Agents & Developers

> [!IMPORTANT]
> **MANDATORY RULES:** Before running any commands or starting implementation, you MUST read:
> 👉 **[PROJECT_OS.md](./PROJECT_OS.md)**
>
> This file contains the mandatory technical protocols, build/test commands (via `docker-helper.sh`), and the Definition of Done required for this project.

### Quick Start (Dev Mode)
To spin up the entire environment with hot-reloading:
```bash
./scripts/docker-helper.sh dev up -d
```


## 📖 Documentation
- [System Architecture](docs/system-architecture.md)
- [API Reference](docs/api-reference.md)
- [Deployment Guide](docs/deployment-guide.md)

## 🧪 Testing

The project uses a highly optimized testing infrastructure designed for speed and reliability, especially on WSL2.

### Prerequisites
- Docker & Docker Compose
- Rust (for host-side compilation)

### Running Tests
Always use the unified orchestrator:
```bash
./run-tests.sh --all             # Run full suite
./run-tests.sh --api [pattern]   # Run backend tests (cargo nextest)
./run-tests.sh --web [pattern]   # Run frontend & E2E tests (playwright)
```

### Key Features
- **Smart Worker Pool**: High-efficiency orchestrator that strictly respects `NUM_WORKERS` limits (auto-capped at 2 for WSL stability).
- **Comprehensive Summary Report**: Generates a detailed, colorized report at the end of every run with per-task status and duration.
- **Cargo Nextest**: Modern test runner for faster, isolated Rust tests.
- **Docker Isolation**: All tests run inside containers. Database is internal (`test_db:5432`) with no host port mapping.
### ⚠️ WSL Performance Note
On WSL2, direct test execution (`cargo test`) will likely freeze your system due to Disk I/O issues. **Always use `./run-tests.sh`** which runs tests inside Docker containers.

## 📝 Changelog
See [CHANGELOG.md](./CHANGELOG.md) for version history.