# LeeAttend - Attendance Management System

A comprehensive attendance management system comprising a local agent for ZKTeco devices and a central high-performance API server.

## 📂 Project Structure

```
leeattend/
├── agent-main/       # Rust Agent Orchestrator (Main Entry Point)
│   └── Manages Python Sidecar processes
│
├── agent-zk-worker-rs/ # Rust ZK Worker (New)
│   ├── High-performance ZK protocol implementation
│   └── Native Rust hardware communication
│
├── apiserver/        # Rust API Server (Runs on Central Cloud/Server)
│   ├── High-performance REST API (Axum)
│   └── Central Database Management
│
└── docs/             # System Documentation
```

## Dự án bao gồm các thành phần:

1.  **apiserver**: Viết bằng **Rust (Axum)**, chịu trách nhiệm quản lý Agent, nhận log và cung cấp API cho Dashboard.
2.  **agent-main**: Viết bằng **Rust**, đóng vai trò Agent Orchestrator quản lý tiến trình.
3.  **agent-zk-worker-rs**: Viết bằng **Rust**, chứa logic xử lý thiết bị ZKTeco (Native Rust) và đồng bộ dữ liệu.
4.  **webdashboard**: Viết bằng **React (Vite)**, giao diện web để quản lý và theo dõi hệ thống.
5.  **docs**: Tài liệu hướng dẫn chi tiết.


## 🤖 For AI Agents: How to Run Commands

> ⚠️ **NEVER** run `cargo`, `npm`, or `pytest` directly on the host. Always use Docker containers. See [PROJECT_OS.md](./PROJECT_OS.md) for details.

### Start the dev environment
```bash
docker compose -f docker-compose.dev.yml up -d
```

### Running commands inside containers
```bash
# Rust (apiserver) — build, test, lint, migrations
docker compose -f docker-compose.dev.yml exec apiserver cargo build
docker compose -f docker-compose.dev.yml exec apiserver cargo test
docker compose -f docker-compose.dev.yml exec apiserver cargo clippy -- -D warnings
docker compose -f docker-compose.dev.yml exec apiserver cargo sqlx migrate run

# Rust (agent-main)
docker compose -f docker-compose.dev.yml exec agent-main cargo build
docker compose -f docker-compose.dev.yml exec agent-main cargo test

# Node.js (webdashboard) — install, build, lint, test
docker compose -f docker-compose.dev.yml exec webdashboard npm install
docker compose -f docker-compose.dev.yml exec webdashboard npm run build
docker compose -f docker-compose.dev.yml exec webdashboard npm run lint
docker compose -f docker-compose.dev.yml exec webdashboard npx playwright test
```

### Debug UI flows
```bash
docker compose -f docker-compose.dev.yml exec webdashboard npx playwright test tests/debug-ui.spec.ts --reporter=line
```

### Stop the dev environment
```bash
docker compose -f docker-compose.dev.yml down
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