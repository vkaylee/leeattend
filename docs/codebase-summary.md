# Codebase Summary

## 📁 Project Structure

```
leeattend/
├── agent/             # Python Sidecar (Hardware integration)
│   └── zk_worker.py   # JSON-RPC Server for ZK devices
├── agent-rust/        # Rust Main Agent (Orchestrator)
│   ├── src/           # Rust Logic
│   └── Cargo.toml     # Rust dependencies
├── apiserver/         # Rust (Axum) Central Server
│   ├── migrations/    # SQLx migrations
│   ├── src/           # API Logic
│   │   ├── handlers/  # API endpoints
│   │   ├── services/  # Business logic
│   │   ├── repositories/ # Database access traits & SQLx impl
│   │   └── models.rs  # Domain models
│   └── tests/         # Integration tests with TestFactory
├── webdashboard/      # React (Vite) Admin UI
│   ├── src/
│   │   ├── components/ # UI Components (Modals, Switchers)
│   │   ├── hooks/      # React Query hooks (Domain state)
│   │   ├── pages/      # Route pages
│   │   ├── lib/        # API clients & Utils
│   │   └── test/       # Vitest setup & Factories
├── docs/              # System documentation
└── plans/             # Feature implementation plans
```

### 🛠 Core Components

1.  **Central API Server (Rust/Axum)**:
    - Auth: JWT-based authentication & RBAC.
    - Workspace: Organizational isolation.
    - Agent Manager: WebSocket handler for remote agents.
    - Discovery: Dynamic port selection (3000+).
    - **Heartbeat Engine**: Time-based tracking of edge agent connectivity.
2.  **Web Dashboard (React/Vite)**:
    - Modern UI with Tailwind CSS & Lucide icons.
    - **Common UI Library**: Standardized components (DropdownMenu, etc.) based on Radix UI.
    - Onboarding & Workspace management flows.
    - Real-time agent status via **Heartbeat Badges**.
    - Discovery: Auto-proxy configuration based on `.api_port`.
3.  **Hybrid Edge Agent (Rust & Python)**:
    - **Rust Orchestrator**: Manages high-level logic, WebSocket tunnel, and SQLite.
    - **Python Sidecar**: Specialized worker for ZK hardware integration via JSON-RPC.
    - Memory-safe event queue & auto-retry logic.

### 🚦 Current Status
- [x] JWT Authentication & Security (Change Password).
- [x] Workspace & Member management (RBAC).
- [x] Accurate Real-time Status (Heartbeat).
- [x] Backend Repository/Service pattern refactor.
- [ ] Automated Deployment scripts (NSSM/Service for production).

### 🛠️ Debugging & Automation Tools

- **Test Factories (Rust & Python):** 
    - `TestFactory` (Rust): Cung cấp Fluent API để tạo Users, Workspaces, Agents với cơ chế **Self-Timeout (5s)** tích hợp sẵn.
    - `AsyncTestFactory` (Python): Hỗ trợ bẻ gãy vòng lặp vô hạn và dọn dẹp task bất đồng bộ an toàn.
- **Playwright Tests (`webdashboard/tests/`):** 
    - `debug-ui.spec.ts`: Script dùng để kiểm tra luồng Auth, Loading loop và lấy logs từ trình duyệt.
- **Port Discovery:**
    - `apiserver/.api_port`: File tạm chứa port hiện tại của backend.
- **Frontend Coverage:** Core hooks (`use-profile`, `use-workspace`) đạt >80% test coverage qua Vitest.

