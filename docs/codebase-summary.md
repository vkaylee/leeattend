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

## 🛠 Key Components

### 1. API Server (Rust)
- **Handlers**: Chứa logic xử lý API endpoints.
- **Services**: Chứa logic nghiệp vụ chính (Domain Logic).
- **Repositories**: Trừu tượng hóa truy cập Database (SQLx).
- **Agent Manager**: Quản lý kết nối WebSocket và điều phối lệnh tới Agents.

### 2. Web Dashboard (React)
- **React Hooks**: Quản lý Server State (fetching/caching) qua React Query.
- **Workspace Switcher**: Chuyển đổi ngữ cảnh tổ chức.
- **Role Management**: Quản lý quyền hạn thành viên (Temporal RBAC).
- **Agent/Device Management**: CRUD giao diện cho hạ tầng phần cứng.

### 3. Agent (Python)
- **Device Manager**: Giao tiếp trực tiếp với máy chấm công ZKTeco.
- **Sync Worker**: Đảm bảo dữ liệu được đẩy lên Central Server an toàn.

### 🛠️ Debugging & Automation Tools

- **Playwright Tests (`webdashboard/tests/`):** 
    - `debug-ui.spec.ts`: Script dùng để kiểm tra luồng Auth, Loading loop và lấy logs từ trình duyệt.
- **Port Discovery:**
    - `apiserver/.api_port`: File tạm chứa port hiện tại của backend.
- **Frontend Coverage:** Core hooks (`use-profile`, `use-workspace`) đạt >80% test coverage qua Vitest.

