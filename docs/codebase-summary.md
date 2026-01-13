# Codebase Summary

## 📁 Project Structure

```
leeattend/
├── agent-main/        # Rust Main Agent (Orchestrator)
│   ├── src/           # Rust Logic (Management, WS, SQLite)
│   └── Cargo.toml     # Rust dependencies
├── agent-zk-worker/   # Python Sidecar (Hardware integration)
│   ├── zk_worker.py   # JSON-RPC Server & ZK Logic
│   └── main.py        # Entry Point
├── apiserver/         # Rust (Axum) Central Server
│   ├── migrations/    # SQLx migrations
│   ├── src/           # API Logic
│   │   ├── handlers.rs # API endpoints
│   │   ├── services/  # Business logic (PayrollService, etc.)
│   │   ├── repositories/ # Database access (Dept, Roster, Payroll)
│   │   ├── models.rs  # Domain models (Employee, Shift, Roster)
│   │   └── dev_initialization.rs # Automatic seeding for debug mode
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
    - **Temporal Engine**: Logic gán ca (`employee_shifts`) dựa trên thời gian. Hỗ trợ ghi đè ca tạm thời lên ca vĩnh viễn và tự động fallback.
    - **Heartbeat Engine**: Time-based tracking of edge agent connectivity.
- **Adjustment Engine**: Workflow for attendance confirmation and manual record corrections. Supports **Robust Asynchronous Batch Import** (Excel/CSV) with auto-recovery and real-time progress tracking.
    - **Multi-format Support**: Integrated support for `.xlsx`, `.xls` (legacy), and `.csv` files.
    - **Workspace Locale Sensitivity**: Configurable `date_format` per workspace to resolve `DD/MM` vs `MM/DD` ambiguity during imports.
- **Time Tracking**: Core logic for punch in/out, shift calculation, and payroll-ready duration processing.
    - **Overtime Engine**: Priority-based calculation logic supporting worked-through breaks and daily OT thresholds.
    - **Leave Management**: Tracking of annual leave entitlements, carry-over balances, and usage history.
    - **Dynamic Accrual Engine**: Automated leave balance calculation based on configurable rules (Monthly/Quarterly/Yearly) and Seniority Bonuses.
    - **Resilience**: **PortFileGuard** ensures robust resource cleanup (port files) even during unexpected shutdowns or panics.
2.  **Web Dashboard (React/Vite)**:
    - Modern UI with Tailwind CSS & Lucide icons.
    - **Common UI Library**: Standardized components (DropdownMenu, etc.) based on Radix UI.
    - **Global Toast System**: Non-blocking notification engine for consistent UX.
    - Onboarding & Workspace management flows.
    - Real-time agent status via **Heartbeat Badges**.
    - Discovery: Auto-proxy configuration based on `.api_port`.
3.  **Hybrid Edge Agent (Rust & Python)**:
    - **Rust Orchestrator (`agent-main`)**: Trái tim của hệ thống. Quản lý SQLite (`attendance.db`), logic đồng bộ dữ liệu (ACK protocol), WebSocket bền bỉ (Exponential Backoff), và điều phối lệnh tới driver.
    - **Python Sidecar (`agent-zk-worker`)**: Một "Thin Driver" siêu nhẹ. Chỉ cung cấp giao thức JSON-RPC để Rust điều khiển thiết bị ZKTeco. Đã được loại bỏ hoàn toàn các logic dư thừa (DB, Cloud logic).
    - **Protocol**: JSON-RPC over TCP (Localhost:5000) giúp tách biệt lỗi phần cứng khỏi logic nghiệp vụ.

### 🚦 Current Status
- [x] JWT Authentication & Security (Change Password).
- [x] Workspace & Member management (RBAC).
- [x] **Full Employee Lifecycle**: CRUD operations, Department transfers with historical tracking (`employee_department_history`).
- [x] **Department Management**: Creation and hierarchical structure foundation.
- [x] Time-series Salary History.
- **Shift Rostering System (Weekly calendar & Priority logic)**:
    - **Selection-First UX**: Người dùng chọn một hoặc nhiều ngày trên lịch để kích hoạt thanh công cụ (Bulk Action Bar), sau đó mới chọn ca làm việc để áp dụng. Không sử dụng các nút hành động trực tiếp trên từng thẻ ngày để giảm nhiễu giao diện.
    - **Visual Indicators**: Hệ thống sử dụng màu sắc và biểu tượng (Zap cho linh hoạt, Shield cho cố định) để phân biệt nguồn gốc của lịch làm việc.
- [x] Attendance Adjustment Workflow (Manual confirmation requests).
- [x] **Payroll Engine**: Advanced logic for worked-through breaks and configurable early clock-in grace periods.
- [x] **Reporting Suite**: Attendance reports supporting monthly summaries and detailed daily punch logs (including break times).
- [x] **Shift Versioning**: Automated tracking of shift configuration changes to maintain historical record integrity.
- [x] **Configurable Work Units**: Support for 15m/30m/60m rounding blocks with configurable policies (Floor/Ceil/Round).
- [x] Accurate Real-time Status (Heartbeat).
- [x] **RBAC Audit**: Full security audit of all administrative endpoints; strict enforcement of `MemberContext` roles (Owner/Admin).
- [x] **Leave Management**: Comprehensive tracking of leave requests, approvals, and annual balances.
- [x] **Dynamic Leave Policies**: Configurable accrual rules (Monthly, Quarterly) with temporal assignment (Effective Date) and seniority bonuses.
- [ ] Overnight Shift Support (Handling punches spanning two calendar days).
- [ ] Automated Deployment scripts (NSSM/Service for production).

### 🛠️ Debugging & Automation Tools

- **Test Factories (Rust & Python):** 
    - `TestFactory` (Rust): Cung cấp Fluent API để tạo Users, Workspaces, Agents với cơ chế **Self-Timeout (5s)** tích hợp sẵn. Extended with builders for `Leaves`, `Adjustments`, `Employees`, `Departments`, and `Policies`.
    - `AsyncTestFactory` (Python): Hỗ trợ bẻ gãy vòng lặp vô hạn và dọn dẹp task bất đồng bộ an toàn.
- **Debug Login:** 
    - Backend: Endpoint `/api/dev/super-user` (debug mode only) tự động sinh token cho admin mặc định.
    - Frontend: Nút "Debug Login" trên `LoginPage` giúp đăng nhập ngay lập tức không cần nhập mật khẩu.
- **Playwright Tests (`webdashboard/tests/`):** 
    - `debug-login.spec.ts`: Kiểm tra luồng đăng nhập và tự động hóa việc vào dashboard.
    - `debug-ui.spec.ts`: Script dùng để kiểm tra luồng Auth, Loading loop và lấy logs từ trình duyệt.
- **Port Discovery:**
    - `apiserver/.api_port`: File tạm chứa port hiện tại của backend.


### Testing Infrastructure
- **Mandatory Quality Gate**: Bắt buộc 100% các bài test (Unit, Integration, Frontend) phải PASS trước khi merge.
- **Parallel E2E Orchestration**: Hệ thống hỗ trợ chạy E2E tests song song (multi-worker) với sự cô lập hoàn toàn.
    - **Worker-Specific Environments**: Mỗi Playwright worker khởi tạo một cặp Backend-Frontend riêng biệt.
    - **Dynamic Port Discovery**: Sử dụng các file tín hiệu port riêng cho từng worker (ví dụ: `.api_port_0`, `.api_port_1`) để tránh xung đột `ADDRINUSE`.
    - **Port Range Isolation**: Tự động tính toán start port dựa trên `workerIndex` với bước nhảy (offset) là 50 port để đảm bảo an toàn tài nguyên networking.
- **Coverage Goal**: Achieved >80% coverage for core business logic.
- **Automated DB Cleanup**: The `apiserver` uses a specialized `TestDatabase` helper that:
    - Creates a unique database for each test session.
    - Automatically runs migrations.
    - Uses a **Detached Background Cleanup** mechanism (via `Drop` trait) to physically remove the database from Postgres after use.
    - Includes a **Signal Trap** (SIGINT/SIGTERM) to ensure orphan databases are cleaned up even if tests are interrupted.
- **Integration Tests**: Comprehensive test suite ensuring 100% reliability of the cleanup logic (`tests/db_trap_test.rs`)

## External Integrations