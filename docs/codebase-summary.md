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
│   │   ├── domain/    # Core Domain Logic (Policy, Payroll)
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
        - **Multi-Segment Calculation**: The `PayrollService` employs a sophisticated segmentation engine that detects "Change Points" (e.g., salary updates, department transfers) within a payroll period.
            - **Prorated Accuracy**: Each segment is calculated independently with its own configuration (Standard Days, Base Salary), ensuring mathematically precise prorating.
            - **Granular Snapshots**: All calculation parameters for every segment are persisted in the `payslip_segments` table, creating an immutable audit trail of exactly *how* the total was derived.
        - **Policy Resolver**: A domain service (`PolicyResolver`) responsible for resolving the active configuration at any given date, respecting the hierarchy: **Department > Workspace**.
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
            - **Flexible Adjustments**: Support for authorized adjustments (e.g., Early Leave for Childcare) via `working_conditions`. Can be configured as Paid or Unpaid time.
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
...
