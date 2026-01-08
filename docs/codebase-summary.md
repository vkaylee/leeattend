# Codebase Summary

## 📁 Project Structure

```
leeattend/
├── agent/             # Python CLI Agent (Hardware integration)
├── apiserver/         # Rust (Axum) Central Server
│   ├── migrations/    # SQLx migrations
│   ├── src/           # API Logic & Models
│   └── tests/         # Integration tests with TestFactory
├── webdashboard/      # React (Vite) Admin UI
│   ├── src/
│   │   ├── components/ # UI Components (Modals, Switchers)
│   │   ├── pages/      # Route pages
│   │   └── test/       # Vitest setup & Factories
├── docs/              # System documentation
└── plans/             # Feature implementation plans
```

## 🛠 Key Components

### 1. API Server (Rust)
- **Handlers**: Chứa logic xử lý API endpoints.
- **Models**: Định nghĩa cấu trúc dữ liệu và Database Schema.
- **Agent Manager**: Quản lý kết nối WebSocket và điều phối lệnh tới Agents.
- **Security**: Extractor `MemberContext` đảm bảo Multi-tenancy và RBAC.

### 2. Web Dashboard (React)
- **Workspace Switcher**: Chuyển đổi ngữ cảnh tổ chức.
- **Role Management**: Quản lý quyền hạn thành viên (Temporal RBAC).
- **Agent/Device Management**: CRUD giao diện cho hạ tầng phần cứng.

### 3. Agent (Python)
- **Device Manager**: Giao tiếp trực tiếp với máy chấm công ZKTeco.
- **Sync Worker**: Đảm bảo dữ liệu được đẩy lên Central Server an toàn.

## 🧪 Testing Strategy
- **Backend**: Integration tests sử dụng `sqlx` và `TestFactory`. Chạy bằng `cargo test`.
- **Frontend**: Unit & Component tests sử dụng `Vitest` và `factories.ts`. Chạy bằng `npm test`.
