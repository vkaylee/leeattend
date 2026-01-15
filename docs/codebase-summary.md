# Codebase Summary

## 📁 Project Structure

```
leeattend/
├── run-tests.sh       # Unified Test Orchestrator (Parallel + Docker)
├── agent-main/        # Rust Main Agent (Orchestrator)
│   ├── src/           # Rust Logic (Management, WS, SQLite)
│   └── Cargo.toml     # Rust dependencies
├── agent-zk-worker/   # Python Sidecar (Hardware integration)
│   ├── zk_worker.py   # JSON-RPC Server & ZK Logic
│   └── main.py        # Entry Point
├── apiserver/         # Rust (Axum) Central Server
│   ├── migrations/    # SQLx migrations
│   ├── src/           # API Logic
│   │   ├── handlers/  # Domain-specific API Modules (Auth, Agents, Shifts...)
│   │   ├── services/  # Business logic (PayrollService, etc.)
│   │   ├── repositories/ # Database access (Dept, Roster, Payroll)
│   │   ├── models.rs  # Domain models (Employee, Shift, Roster)
│   │   └── dev_initialization.rs # Automatic seeding for debug mode
│   └── tests/         # Integration tests with TestFactory
├── webdashboard/      # React (Vite) Admin UI
│   ├── src/
│   │   ├── components/ # UI Components (Modals, Switchers)
│   │   ├── hooks/      # React Query hooks (Domain state)
│   │   ├── i18n/       # Internationalization (locales, config)
│   │   ├── pages/      # Route pages
│   │   ├── lib/        # API clients & Utils
│   │   │   └── api/    # Modular API Client (Auth, Workspaces, etc.)
│   │   └── test/       # Vitest setup & Factories
├── docs/              # System documentation
└── plans/             # Feature implementation plans
```

### 🛠 Core Components

1.  **Central API Server (Rust/Axum)**:
    - **Modular Architecture**: Replaced monolithic handler file with domain-driven modules (`handlers/auth.rs`, `handlers/attendance.rs`, etc.) for better maintainability and separation of concerns.
    - Auth: JWT-based authentication & RBAC.
    - Workspace: Organizational isolation.
    - Agent Manager: WebSocket handler for remote agents.
    - **Discovery**: Dynamic port selection (3000+).
    - **Branding Engine**: Support for workspace-specific `theme_primary_color` with persistence and validation.
    - **Temporal Engine**: Logic gán ca (`employee_shifts`) dựa trên thời gian. Hỗ trợ ghi đè ca tạm thời lên ca vĩnh viễn và tự động fallback.
    - **Heartbeat Engine**: Time-based tracking of edge agent connectivity.
    - **Timezone Engine**: Comprehensive timezone management per workspace. Correctly handles local-to-UTC conversion for external data imports (Excel/CSV) using the workspace's configured IANA timezone ID.
    - **Internationalization (i18n) Engine**: Persistent language preferences stored at both user and workspace levels. Backend automatically detects and synchronizes with frontend state via `preferred_language` and `default_language` fields.
    - **Adjustment Engine**: Workflow for attendance confirmation and manual record corrections. Supports **Robust Asynchronous Batch Import** (Excel/CSV) with auto-recovery and real-time progress tracking.
    - **Attendance Persistence Layer**: 
        - **Raw Event Store**: Dedicated `raw_attendance_logs` table for immutable original punch data from hardware and imports.
        - **Pairing Engine**: `PairingService` handles the automatic conversion of flat events into logical attendance sessions.
        - **Accurate Calculation Engine**: `AttendanceCalculator` implements a high-precision **Clamping + Bucket** strategy:
            - **Standard Clamping**: Work is capped within `[ShiftStart, ShiftEnd]`. Early arrivals (e.g., 07:25 for 07:30) are correctly handled as `max(In, Start)`.
            - **Bucket Logic**: Separation of Standard, Early OT, and Late OT time buckets.
            - **Early OT Filter**: Policy-based filtering of early arrivals via `allow_early_ot` toggle.
            - **Boundary Rounding**: Uses `Floor` rounding on total bucket duration to handle "late 1 min = lose 30 mins" policies.
        - **Strict Validation**: High-fidelity file parsing that enforces workspace settings and provides line-by-line detailed error context.
    - **Time Tracking**: Core logic for punch in/out, shift calculation, and payroll-ready duration processing.
        - **Attendance Calculator**: Centralized Domain Service (`attendance_calculator.rs`) implementing pure business logic for:
            - **Grace Period**: Logic "châm chước" đi trễ (ví dụ: cho phép trễ 15p).
            - **Break Deduction**: Tự động trừ giờ nghỉ trưa hoặc trừ theo thực tế.
            - **Rounding**: Làm tròn giờ công (Floor/Ceil/Round) theo Work Unit (15m/30m).
        - **Overtime Engine**: Priority-based calculation logic supporting worked-through breaks and daily OT thresholds.
        - **Leave Management**: Tracking of annual leave entitlements, carry-over balances, and usage history.
        - **Admin Management & Auditing**: High-level workspace overview of all employee balances. Merged history stream of auto-accruals, usage, and manual corrections.
        - **Dynamic Accrual Engine**: Automated leave balance calculation based on configurable rules (Monthly/Quarterly/Yearly) and Seniority Bonuses.
        - **Resilience**: **PortFileGuard** ensures robust resource cleanup (port files) even during unexpected shutdowns or panics.

2.  **Web Dashboard (React/Vite)**:
    - **Premium UI (Violet)**: Zero-distraction layout with a static breadcrumb header and a single-open accordion sidebar.
    - **Dynamic Themeing**: `useTheme` hook for on-the-fly calculation of color palettes from a single HEX code.
    - **Personalization**: User-level theme overrides stored in `localStorage` with cross-tab sync.
    - **Modular API Client**: Refactored monolithic `api.ts` into a structured `lib/api/` directory with domain-specific files (`auth.ts`, `workspaces.ts`, etc.) while maintaining a unified export via `index.ts`.
    - **Resilient API Fetch**: Hardened the central `apiFetch` utility (`lib/api/helpers.ts`) to avoid common URL construction errors like double-prefixing, ensuring stable communication with the backend regardless of path formatting.
    - **Robust API URL Resolution**: Frontend client (`api/config.ts`) intelligently determines API base URL, prioritizing relative `/api` paths for web development environments (to leverage Vite's dynamic proxy) and `VITE_API_URL` for production or Tauri builds.
    - **Common UI Library**: Standardized components (Modal, Combobox, DropdownMenu, etc.) based on Radix UI and React Portals.
    - **Portal-based Modals**: Centralized `Modal` component using `createPortal` to ensure dialogs are always rendered at the top level of the DOM, preventing clipping by parent containers and ensuring full-screen backdrops.
    - **Global Toast System**: Non-blocking notification engine for consistent UX.
    - **Unified Date/Time Formatting**: Centralized formatting via `useDateTimeFormatter` hook, ensuring all displays respect per-workspace `date_format` and `time_format` preferences.
    - **Full-stack Multi-language Support**: Comprehensive i18n implementation using `i18next` for Vietnamese, English, and Chinese. Features automatic synchronization between browser local state and backend user preferences.
    - **Quick Month Filters**: Optimized UX for report and log views with shortcut buttons for "This Month", "Last Month", and "Custom" date ranges.
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
- [x] **Full Employee Lifecycle**: CRUD operations, Department transfers with historical tracking (`employee_department_history`), and **Employment History System** (Re-hire support via `employment_periods`).
- [x] **Department Management**: Full CRUD operations with manager assignment and historical tracking.
- [x] Time-series Salary History.
- [x] **Shift Rostering System (Weekly calendar & Priority logic)**:
    - **Selection-First UX**: Người dùng chọn một hoặc nhiều ngày trên lịch để kích hoạt thanh công cụ (Bulk Action Bar), sau đó mới chọn ca làm việc để áp dụng. Không sử dụng các nút hành động trực tiếp trên từng thẻ ngày để giảm nhiễu giao diện.
    - **Visual Indicators**: Hệ thống sử dụng màu sắc và biểu tượng (Zap cho linh hoạt, Shield cho cố định) để phân biệt nguồn gốc của lịch làm việc.
- [x] Attendance Adjustment Workflow (Manual confirmation requests).
- [x] **Payroll Engine**: Advanced logic for worked-through breaks and configurable early clock-in grace periods.
- [x] **Reporting Suite**: Attendance reports now use the **Attendance Calculator** to show accurate "Payable Minutes" (applying grace period, break deduction, and rounding) instead of just raw duration.
- [x] **Shift Versioning**: Automated tracking of shift configuration changes to maintain historical record integrity.
- [x] Configurable Work Units: Support for 15m/30m/60m rounding blocks with configurable policies (Floor/Ceil/Round).
- [x] **Raw Attendance Audit**: Immutable event-centric storage with smart pairing logic.
- [x] **Workspace Personalization**: Per-workspace preference for `date_format`, `time_format`, and `timezone` with consistent UI and processing application.
- [x] **Full-stack Multi-language Support**: Native support for Vietnamese (vi), English (en), and Chinese (zh) with automatic backend sync.
- [x] Accurate Real-time Status (Heartbeat).
- [x] **RBAC Audit**: Full security audit of all administrative endpoints; strict enforcement of `MemberContext` roles (Owner/Admin).
- [x] **Leave Management**: Comprehensive tracking of leave requests, approvals, and annual balances.
- [x] **Admin Leave Dashboard**: Workspace-wide overview of quotas and manual balance adjustments with audit trail.
- [x] **Dynamic Leave Policies**: Configurable accrual rules (Monthly, Quarterly) with temporal assignment (Effective Date) and seniority bonuses.
- [x] **Overnight Shift Support**: Complete linearization engine for punches spanning multiple calendar days.
- [ ] Automated Deployment scripts (NSSM/Service for production).

### 🛠️ Debugging & Automation Tools

- **Test Factories (Rust & Python):** 
    - `TestFactory` (Rust): Cung cấp Fluent API để tạo Users, Workspaces, Agents với cơ chế **Self-Timeout (5s)** tích hợp sẵn. Extended with builders for `Leaves`, `Adjustments`, `Employees`, `Departments`, and `Policies`.
    - `AsyncTestFactory` (Python): Hỗ trợ bẻ gãy vòng lặp vô hạn và dọn dẹp task bất đồng bộ an toàn.
- **Debug Login:** 
    - Backend: Endpoint `/api/dev/super-user` (debug mode only) tự động sinh token cho admin mặc định.
    - Frontend: Nút "Debug Login" trên `LoginPage` giúp đăng nhập ngay lập tức không cần nhập mật khẩu.
    - **Reliable Dev Seeding**: Seeding logic uses `INSERT ... ON CONFLICT DO UPDATE`, ensuring that "Fast Login" credentials and default workspace settings are always in a valid state for development.
- **Debug Workspace Tools:** Developer-only section in Workspace Settings to quickly clear specific data groups (Attendance, Schedules, Leave, Payroll) or perform a full workspace reset.
- **Playwright Tests (`webdashboard/tests/`):** 
    - `debug-login.spec.ts`: Kiểm tra luồng đăng nhập và tự động hóa việc vào dashboard.
    - `debug-ui.spec.ts`: Script dùng để kiểm tra luồng Auth, Loading loop và lấy logs từ trình duyệt.
- **Port Discovery:**
    - `apiserver/.api_port`: File tạm chứa port hiện tại của backend.

### Testing Infrastructure
- **Unified 4-Phase Pipeline (`run-tests.sh`)**: A high-performance test runner designed for extreme reliability and speed.
    - **Phase 0 (Linting)**: Mandatory code quality check using `clippy` and `eslint`. Halts pipeline on any warnings.
    - **Phase 1 (Parallel Unit)**: Simultaneously executes unit tests for all modules (`apiserver`, `agent-main`, `agent-zk-worker`, `webdashboard`).
    - **Phase 2 (Orchestration)**: Provisions isolated databases for parallel E2E and pre-compiles the backend binary.
    - **Phase 3 (Parallel Integration)**: Runs Rust integration tests and Playwright E2E concurrently.
- **Real-time Sequential Logging**: Sophisticated output engine that streams parallel logs in real-time sequential blocks (e.g., Worker 2's logs start exactly when Worker 1 finishes), providing "parallel speed with sequential clarity."
- **Massively Parallel E2E Isolation**:
    - **Multi-DB Backend**: Automatically spawns dedicated databases (`test_database_e2e_0-3`) for each Playwright worker to eliminate state pollution and race conditions.
    - **Pre-compiled Performance**: Workers execute the pre-built `apiserver` binary directly, avoiding `cargo run` compilation overhead and resolving Rust file-lock contention.
    - **Scalability**: Optimized for 4 concurrent workers with strictly enforced 1-minute per-test timeouts.
- **Mandatory Quality Gate**: Bắt buộc 100% các bài test (Unit, Integration, Frontend) phải PASS trước khi merge.
- **Comprehensive Auth Testing**: Added production-ready E2E suites for standard login flows, ensuring correct i18n handling, credential validation, and redirection logic.
- **Coverage Goal**: Achieved >83% coverage for core business logic.
- **Automated DB Cleanup**: The `apiserver` uses a specialized `TestDatabase` helper that:
    - Creates a unique database for each test session.
    - Automatically runs migrations.
    - Uses a **Detached Background Cleanup** mechanism (via `Drop` trait) to physically remove the database from Postgres after use.
    - Includes a **Signal Trap** (SIGINT/SIGTERM) to ensure orphan databases are cleaned up even if tests are interrupted.
- **Integration Tests**: Comprehensive test suite ensuring 100% reliability of the cleanup logic (`tests/db_trap_test.rs`)

## External Integrations