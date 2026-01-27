# LeeAttend - Attendance Management System

A comprehensive attendance management system comprising a local agent for ZKTeco devices and a central high-performance API server.

## 📂 Project Structure

```
leeattend/
├── agent-main/       # Rust Agent Orchestrator (Main Entry Point)
│   └── Manages Python Sidecar processes
│
├── agent-zk-worker/  # Python ZK Worker (Sidecar)
│   ├── Connects to ZKTeco devices
│   └── Syncs data to Central Server
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
3.  **agent-zk-worker**: Viết bằng **Python**, chứa logic xử lý thiết bị ZKTeco và đồng bộ dữ liệu.
4.  **webdashboard**: Viết bằng **React (Vite)**, giao diện web để quản lý và theo dõi hệ thống.
5.  **docs**: Tài liệu hướng dẫn chi tiết.


## 🤖 For AI Agents: How to Debug UI

If you need to verify UI changes or fix loading issues:

1.  **Ensure Backend is running**: 
    ```bash
    AI_MANAGED=true (cd apiserver && cargo run) > /tmp/backend.log 2>&1 &
    ```
2.  **Ensure Frontend is running**: 
    ```bash
    AI_MANAGED=true (cd webdashboard && npm run dev) > /tmp/frontend.log 2>&1 &
    ```
3.  **Run Automated Debug**:
    ```bash
    cd webdashboard
    npx playwright test tests/debug-ui.spec.ts --reporter=line
    ```
4.  **Parallel E2E Testing**: To run all tests in isolated parallel workers:
    ```bash
    cd webdashboard
    npx playwright test --workers=2
    ```
5.  **🚨 IMPORTANT: Cleanup**: After testing, kill ONLY the AI-managed processes:
    ```bash
    pkill -f "AI_MANAGED=true" || true
    ```


## 📖 Documentation
- [System Architecture](docs/system-architecture.md)
- [API Reference](docs/api-reference.md)
- [Deployment Guide](docs/deployment-guide.md)

## 🧪 Testing
Run the unified test suite with automatic environment management:
```bash
./run-tests.sh --all             # Run everything (Lint + Unit + Integration)
./run-tests.sh --api [pattern]   # Run only API/Backend tests
./run-tests.sh --web [pattern]   # Run only Web/Frontend/E2E tests
```

> **Note**: The test runner automatically detects your system resources (RAM/CPU) and tunes concurrency to prevent system freezes. On low-memory systems (<8GB), it automatically switches to serial execution for stability.

*Note: Linting (Phase 0) is mandatory and runs before tests in all modes.*

Manual execution (Internal scripts):
```bash
./scripts/test-lint.sh      # Phase 0
./scripts/test-unit.sh      # Phase 1
./setup-test-env.sh ./scripts/test-integration.sh # Phase 3 (needs DB environment)
```

## 📝 Changelog
See [CHANGELOG.md](./CHANGELOG.md) for version history.