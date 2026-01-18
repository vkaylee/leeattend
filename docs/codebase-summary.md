# Codebase Summary

## 📁 Project Structure

```
leeattend/
├── run-tests.sh       # Unified Test Orchestrator (Wrapper)
├── setup-test-env.sh  # Test Environment Setup (Parent Script)
├── scripts/           # Modular Test Scripts
│   ├── test-utils.sh       # Shared utilities (logging, parallel execution)
│   ├── test-lint.sh        # Phase 0: Linting
│   ├── test-unit.sh        # Phase 1: Unit Tests
│   └── test-integration.sh # Phase 3: Integration Tests
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
│   │   ├── utils/     # Shared utilities (DateTime, etc.)
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
    - **Workspace Identification Refactor (Path Priority)**: Modernized the API identification strategy for multi-tenant isolation.
        - **Smart URI Extraction**: The backend `WorkspaceId` extractor now prioritizes extracting the Workspace ID directly from URL path parameters (`/workspaces/:ws_id/...`).
        - **Optional Headers**: Removed the strict requirement for the `X-Workspace-Id` header when the ID is present in the path, simplifying client integrations and improving compatibility with standard browser requests.
        - **Header Fallback**: Maintained header-based identification as a secondary fallback for ESS routes and legacy support.
    - **Branding Engine**: Support for workspace-specific `theme_primary_color` with persistence and validation.
    - **Temporal Engine**: Logic gán ca dựa trên thời gian với cơ chế kế thừa phân cấp (Inheritance). Thứ tự ưu tiên: `Manual Override (Roster)` > `Individual Pattern` > `Department Pattern` > `Fixed Assignment` > `Department Default`. Hỗ trợ gán mẫu lịch làm việc (Working Patterns) hàng loạt.
    - **Heartbeat Engine**: Time-based tracking of edge agent connectivity.
    - **Timezone Engine**: Comprehensive timezone management per workspace. Correctly handles local-to-UTC conversion for external data imports (Excel/CSV) using the workspace's configured IANA timezone ID.
    - **Internationalization (i18n) Engine**: Persistent language preferences stored at both user and workspace levels. Backend automatically detects and synchronizes with frontend state via `preferred_language` and `default_language` fields.
        - **Feature-First Structure**: Locale files (`.json`) are organized with major functional modules (Auth, ESS, Shifts, etc.) at the top-level for concise key access and maintainability.
        - **100% Key Parity**: Enforced 100% key synchronization between all supported languages (en, vi, zh) through an automated audit suite.
    - **Unified Employee Search**: High-fidelity search subsystem used across all administrative and reporting modules.
        - **Server-Side Filtering**: Optimized SQL queries using `ILIKE` for case-insensitive matching across first names, last names, and employee ID numbers.
        - **Intelligent Debouncing**: Integrated `useDebounce` hook (400ms delay) to minimize server load while providing a responsive "search-as-you-type" experience.
        - **Multi-Select Workflow**: Support for building complex filters using a dismissal chip-based UI, allowing users to build a "basket" of employees for targeted reporting.
    - **Assignment Rule Engine (Enterprise)**: Advanced scheduling automation based on flexible conditions.
        - **Condition Evaluation**: `rule_evaluator.rs` resolves complex logic (Day of week, Department, Location) to determine the correct shift pattern for an employee.
        - **Priority Logic**: Rules are evaluated in descending order of priority, allowing global defaults to be overridden by department or individual-specific rules.
    - **Adjustment Engine**: Workflow for attendance confirmation and manual record corrections. Supports **Robust Asynchronous Batch Import** (Excel/CSV) with auto-recovery and real-time progress tracking.
    - **Attendance Persistence Layer**: 
        - **Raw Event Store**: Dedicated `raw_attendance_logs` table for immutable original punch data from hardware and imports.
        - **Pairing Engine (Strategy Pattern)**: `PairingService` handles the conversion of flat events into logical sessions using a pluggable strategy architecture:
            - **Pluggable Algorithms**: Supports `DpPatternStrategy` (intelligent matching), `AnchorFirstStrategy` (first/last), `SequentialStrategy` (chronological/retail), and `FsmStrategy` (state-based).
            - **Morning Lookback (Overnight support)**: Backend automatically reprocesses the previous day's attendance if a log is received before 12:00 PM. This ensures morning check-outs are correctly paired with the previous night's check-ins without requiring manual reprocessing.
            - **Metadata-Aware**: Algorithms use the `LogEvent` structure to access `punch_type` information (Status) from devices, enabling explicit intent-based pairing.
            - **Workspace Selection**: Users can configure the preferred algorithm per workspace to match their specific business environment (Office, Factory, Retail, Professional Hardware).
            - **High-Penalty Matching**: Optimized the DP algorithm with a 300-minute unmatched log penalty to correctly capture extreme overtime scenarios.
        - **Accurate Calculation Engine**: `AttendanceCalculator` implements a high-precision **Integer Work Unit (WU)** strategy:
            - **Atomic Integer Units**: All internal calculations (Standard, OT) are performed in strictly integer units. Fractional units (e.g., 1.5 units) are physically impossible in the data layer.
            - **Unit-Based Rounding**: Uses workspace-specific `work_unit_minutes` (e.g., 15, 30, 60m) to convert raw minutes to units using configurable policies (`Floor`, `Ceil`, `Round`).
            - **Standard Clamping**: Work is capped within `[ShiftStart, ShiftEnd]`. Early arrivals (e.g., 07:25 for 07:30) are correctly handled as `max(In, Start)`.
            - **Bucket Logic**: Separation of Standard Units and Overtime Units.
            - **Early OT Filter**: Policy-based filtering of early arrivals via `allow_early_ot` toggle.
            - **Historical Integrity (Snapshotting)**: Payslips store immutable unit counts (`standard_units`, `overtime_units`) at the moment of calculation, ensuring that future changes to workspace unit settings do not affect past payroll records.
        - **Strict Validation**: High-fidelity file parsing that enforces workspace settings and provides line-by-line detailed error context.
    - **Dynamic Payroll System (Component-Based)**: Flexible architecture for multi-tenant salary calculation.
        - **Salary Component Directory**: Allows defining custom income types (Meal, Gas, Seniority) with configurable taxability and "Base for OT" flags.
        - **Flexible Standard Day Policy**: Support for diverse company cultures via `Fixed` (e.g., always 26 days) or `Actual Month` (calendar-based) standard day modes.
        - **Dynamic Hourly Rate Engine**: Automatically derives OT unit rates based on the sum of all "Base for OT" components divided by the calculated standard units.
        - **Historical Tracking**: Time-series storage of component assignments ensuring payroll integrity even when salaries change mid-period.
    - **Employee Self-Service (ESS) Portal**: Dedicated subsystem for employee empowerment and data autonomy.
        - **Secure Personal Access**: Direct access to personal data (Profile, Contacts, Documents) without requiring workspace administrative roles.
        - **Emergency Contact Management**: Self-service tools for maintaining up-to-date emergency notification information.
        - **Document Repository**: Infrastructure for storing and accessing employment-related documents (IDs, contracts).
        - **Personal Insights**: On-demand viewing of payslips, attendance history, and real-time leave balances.
    - **API Standardization Framework**: Mandatory security and stability layer for all outbound requests.
        - **Fetch Wrapper**: Centralized `apiFetch` utility (`lib/api/helpers.ts`) implemented with a strict 10s timeout using `AbortController`.
        - **Header Injection**: Automatically attaches `Authorization` (JWT), `X-Workspace-Id`, and `Content-Type: application/json` to every call.
        - **Regression Enforcement**: Automated `standardization.test.ts` suite that scans the API layer to forbid direct use of `window.fetch`.
    - **Time Tracking**: Core logic for punch in/out, shift calculation, and payroll-ready duration processing.
        - **Attendance Calculator**: Centralized Domain Service (`attendance_calculator.rs`) implementing pure business logic for:
            - **Grace Period**: Logic "châm chước" đi trễ (ví dụ: cho phép trễ 15p).
            - **Break Deduction**: Tự động trừ giờ nghỉ trưa hoặc trừ theo thực tế.
            - **Work Unit Calculation**: Quy đổi từ phút thực tế sang số Work Unit (Số nguyên) và áp dụng quy tắc làm tròn (Floor/Ceil/Round) theo cấu hình Workspace (15m/30m/60m).
        - **Overtime Engine**: Priority-based calculation logic supporting worked-through breaks and daily OT thresholds. Result is converted to integer Units.
        - **Leave Management**: Tracking of annual leave entitlements, carry-over balances, and usage history.
        - **Admin Management & Auditing**: High-level workspace overview of all employee balances. Merged history stream of auto-accruals, usage, and manual corrections.
        - **Dynamic Accrual Engine**: Automated leave balance calculation based on configurable rules (Monthly/Quarterly/Yearly) and Seniority Bonuses.
        - **Hierarchical Assignment Engine**: Flexible policy distribution system supporting Workspace, Department, and Employee levels.
            - **Priority Logic**: Strictly enforces Employee > Department > Workspace override order.
            - **Transactional Auto-Close**: Seamlessly manages policy transitions by automatically closing previous assignments when a new one is created for the same category.
            - **Multi-Category Support**: Added `leave_type` field to definitions to allow distinct accrual rules for different leave categories (Vacation, Sick, etc.).
        - **Resilience**: **PortFileGuard** ensures robust resource cleanup (port files) even during unexpected shutdowns or panics.
    - **Advanced Deserialization**: Implemented `serde_helpers` to accurately distinguish between missing fields and explicit `null` values in partial update APIs (e.g., clearing a department's default pattern).

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
    - **Unified Date/Time Formatting**: Centralized formatting via `useDateTimeFormatter` hook, ensuring all displays respect per-workspace `date_format` and `time_format` preferences. Includes helpers for `formatDate`, `formatTime`, `formatDateTime`, and `formatMonthYear`.
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
- [x] **Shift Rostering System (Weekly calendar & Priority logic)**:
    - **Selection-First UX**: Người dùng chọn một hoặc nhiều ngày trên lịch để kích hoạt thanh công cụ (Bulk Action Bar), sau đó mới chọn ca làm việc để áp dụng. Không sử dụng các nút hành động trực tiếp trên từng thẻ ngày để giảm nhiễu giao diện.
    - **Visual Indicators**: Hệ thống sử dụng màu sắc và biểu tượng (Zap cho linh hoạt, Shield cho cố định) để phân biệt nguồn gốc của lịch làm việc.
- [x] **Auto Assignment Rules**: Set up recurring schedules automatically based on department, location, or day-of-week criteria.
- [x] **Bulk Schedule Assignment**: Cho phép gán mẫu lịch (Working Patterns) cho hàng loạt nhân viên và thiết lập mẫu lịch mặc định cho từng Phòng ban để tự động hóa khâu onboarding.
- [x] Attendance Adjustment Workflow (Manual confirmation requests).
- [x] **Payroll Engine**: Advanced logic for worked-through breaks and configurable early clock-in grace periods.
- [x] **Dynamic Payroll System**: Fully implemented component-based salary management with flexible standard day policies and historical tracking.
- [x] **Payroll UI Suite**: Integrated management tools for workspace payroll settings and employee-specific component assignments.
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
- [x] **Employee Self-Service (ESS) Portal**: Personal profile management, emergency contacts, and document vault.
- [x] **Advanced Pairing Algorithms**: Industrial-grade FSM (State-based) and Sequential (Retail) matching strategies.
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
- **Refactored Modular Test Runner (`run-tests.sh` + `scripts/`)**: A modular and robust test execution system.
    - **`run-tests.sh`**: Acts as an orchestrator, delegating to specific phase scripts.
    - **`setup-test-env.sh`**: "Parent" script that handles the entire DB infrastructure lifecycle (Up/Migration/Down) and executes child scripts (like integration tests) within that environment.
    - **Modular Scripts (`scripts/`)**:
        - `test-utils.sh`: Shared logic for parallel logging, worker calculation, and cleanup traps.
        - `test-lint.sh`: Phase 0 (Linting).
        - `test-unit.sh`: Phase 1 (Unit Tests).
        - `test-integration.sh`: Phase 3 (Integration/E2E), automatically pre-compiles backend before running.
- **Resilient Stability Suite**:
    - **Initialization Hardening**: Standardized test timeouts to **30 seconds** across all frameworks to accommodate the high resource load of parallel setup.
    - **Regression Protection**: Implemented `stability-regression.spec.ts` to verify that critical functional fixes (like Workspace Context headers and UI icon presence) remain intact.
- **Unified 4-Phase Pipeline**:
    - **Phase 0 (Linting)**: Mandatory code quality check using `clippy` and `eslint`. Halts pipeline on any warnings (`--max-warnings 0`).
    - **Phase 1 (Unit Tests)**: Simultaneously executes unit tests for all modules (`apiserver`, `agent-main`, `agent-zk-worker`, and `webdashboard`).
    - **Phase 2 (Environment Setup)**: Provisions isolated databases for parallel E2E and pre-compiles the backend binary. Strict error checking ensures the environment is perfect before testing starts.
    - **Phase 3 (Parallel Integration)**: Runs Rust integration tests and Playwright E2E concurrently using the pre-provisioned environment.
- **API Standardization Coverage**: `standardization.test.ts` ensures 100% of API implementations use the secure `apiFetch` wrapper.
- **Automated i18n Audit Suite**: `translation-usage.test.ts` enforces 100% key parity across all locale files and scans the codebase to ensure all utilized keys exist in all languages. It also proactively detects hardcoded JSX strings.
- **Timeout Verification**: `apiFetch.test.ts` unit tests verify the integrity of the 10s request abort logic.
- **Fail-Fast Engine**: Orchestrator immediately terminates all sibling tasks in a phase if a single task fails, providing instantaneous feedback and saving system resources.
- **Real-time Sequential Logging**: Sophisticated output engine that streams parallel logs in real-time sequential blocks (e.g., Worker 2's logs start exactly when Worker 1 finishes), providing "parallel speed with sequential clarity."
- **Reliable Resource Cleanup**: Implemented a comprehensive `trap` mechanism in the orchestrator that automatically stops containers, drops Docker volumes (`-v`), and removes temporary signal files on `EXIT`, `INT`, or `TERM`.
- **Massively Parallel E2E Isolation**:
    - **Multi-DB Backend**: Automatically spawns dedicated databases (`test_database_e2e_0-N`) for each Playwright worker to eliminate state pollution.
    - **Dynamic Worker Scaling**: Automatically calculates optimal concurrency based on system CPU cores (Half of total cores, min 1).
    - **Balanced Parallelism**: Synchronizes Playwright workers and Rust test threads (`--test-threads`) to prevent CPU oversubscription, achieving peak throughput without system lag.
    - **Pre-compiled Performance**: Workers execute the pre-built `apiserver` binary directly, avoiding `cargo run` compilation overhead and resolving Rust file-lock contention.
    - **Determinism**: Strictly enforced 1-minute per-test timeouts.
- **Mandatory Quality Gate**: Bắt buộc 100% các bài test (Unit, Integration, Frontend) phải PASS trước khi merge.
- **Comprehensive Auth Testing**: Added production-ready E2E suites for standard login flows, ensuring correct i18n handling, credential validation, and redirection logic.
- **Leave Policy Assignment**: Added comprehensive integration test (`leave-policy-assignment.spec.ts`) covering multi-level policy allocation (Employee/Department) and transactional auto-closing logic.
- **UI Interaction Robustness**: Hardened portal-based components (Combobox, Modal) with high z-index stacking (`z-[10000]`) and standard ARIA roles (`role="status"`) to ensure reliable automated interaction and accessibility. Verified via `z-index-stacking.spec.ts`.
- **Payroll Testing**: Comprehensive suite covering both legacy flat history and new dynamic component logic. Includes verification of hourly rate derivation, standard day modes (Fixed/Actual), and break deductions.
- **UI Interaction Verification**: Dedicated suite for Sidebar navigation (`sidebar-navigation.spec.ts`) ensuring correct default expansion and accordion behavior.

## External Integrations