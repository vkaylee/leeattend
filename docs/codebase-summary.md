# Codebase Summary

## 📁 Project Structure

```
leeattend/
├── agent/             # Python CLI Agent (Hardware integration)
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
2.  **Web Dashboard (React/Vite)**:
    - Modern UI with Tailwind CSS & Lucide icons.
    - Onboarding & Workspace management flows.
    - Real-time agent status via WebSocket proxy.
    - Discovery: Auto-proxy configuration based on `.api_port`.
3.  **Local Agent (Python/PySide6)**:
    - Multi-threaded worker for ZK fingerprint devices.
    - WebSocket tunnel to Central Server.
    - Memory-safe event queue & auto-retry logic.

### 🚦 Current Status
- [x] JWT Authentication & Security (Change Password).
- [x] Workspace & Member management (RBAC).
- [x] Real-time Agent/Device monitoring.
- [x] Dynamic Dev Environment Setup.
- [ ] Production Build & Deployment scripts (NSSM/Service).

### 🛠️ Debugging & Automation Tools

- **Playwright Tests (`webdashboard/tests/`):** 
    - `debug-ui.spec.ts`: Script dùng để kiểm tra luồng Auth, Loading loop và lấy logs từ trình duyệt.
- **Port Discovery:**
    - `apiserver/.api_port`: File tạm chứa port hiện tại của backend.
- **Frontend Coverage:** Core hooks (`use-profile`, `use-workspace`) đạt >80% test coverage qua Vitest.

