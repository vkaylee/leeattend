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
│   │   ├── AppRoutes.tsx # Centralized Route Definitions
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
        - **Automated Quality Suite**: Enhanced `translation-usage.test.ts` provides comprehensive codebase auditing:
            - **100% Key Parity**: Enforces identical key structures across all supported languages (en, vi, zh).
            - **Missing Key Detection**: Scans all `.ts/.tsx` files for `t()` calls and verifies their existence in locales.
            - **Untranslated Value Guard**: Warns if a translation value matches the key path (indicative of auto-generated stubs).
            - **Hardcoded String Scanner**: Identifies raw text in JSX that should be moved to translation files.
    - **Settings Hub (Centralized Configuration)**:
        - **Unified Navigation**: Redesigned sidebar and routing to provide a single entry point for all workspace, personnel, and system configurations.
        - **Categorized Management**: Logical grouping into General, People, Time & Attendance, Payroll, and Connectivity settings.
        - **Responsive Layout**: Optimized mobile-friendly grid system for setting cards and detailed configuration panels.
    - **Unified Employee Search**: High-fidelity search subsystem used across all administrative and reporting modules.
        - **Server-Side Filtering**: Optimized SQL queries using `ILIKE` for case-insensitive matching across first names, last names, and employee ID numbers.
        - **Intelligent Debouncing**: Integrated `useDebounce` hook (400ms delay) to minimize server load while providing a responsive "search-as-you-type" experience.
        - **Multi-Select Workflow**: Support for building complex filters using a dismissal chip-based UI, allowing users to build a "basket" of employees for targeted reporting.
    - **Assignment Rule Engine (Enterprise)**: Advanced scheduling automation based on flexible conditions.
        - **Condition Evaluation**: `rule_evaluator.rs` resolves complex logic (Day of week, Department, Location) to determine the correct shift pattern for an employee.
        - **Schedule Traceability (New)**: The engine now records the "why" behind every assignment. Each generated schedule entry includes a `traceability` object identifying the matching rule, evaluation timestamp, and decision path.
        - **Priority Logic**: Rules are evaluated in descending order of priority, allowing global defaults to be overridden by department or individual-specific rules.
    - **Adjustment Engine**: Workflow for attendance confirmation and manual record corrections. Supports **Robust Asynchronous Batch Import** (Excel/CSV) with auto-recovery, duplicate skipping (Silent Skip), and real-time progress tracking.
        - **Repositories**:
            - **RosterRepository**: Manages shift rosters and implements the core schedule resolution logic (`get_effective_shift`) with traceability.
            - **EmployeeRepository**: Handles employee CRUD, history, and optimized bulk creation with duplicate handling (`create_employee_tx_skip_duplicates`).
            - **PayrollRepository**: Handles salary components, payslip generation, and segment storage.
            - **TimeEntryRepository**: Stores raw punches and paired time entries with BSON metadata.
            - **AgentRepository**: Manages authorized agents and device trust.
        - **Services**:
            - **AttendanceImportService**: Manages asynchronous file processing, validation, and silent-skipping of duplicate records.
            - **RuleEvaluator**: Determines the "winning" assignment rule based on priority (P100 Roster > P80 Pattern > P60 Rule > P40 Dept Pattern > P20 Fixed > P10 Dept Fixed).
            - **PolicyResolver**: Hierarchical configuration resolution (Individual > Department > Workspace) with full traceability for payroll parameters (Standard Days, Hours/Day).
            - **PairingService**: Core engine for converting raw punches into time entries using FSM or Anchor strategies.
            - **PayrollService**: Orchestrates multi-segment payroll calculation, tax logic, and high-performance batch processing for large teams.
        - **Tests**:
            - **fallback_system_tests.rs**: Comprehensive verification of the schedule priority hierarchy (P100-P10).
            - **schedule_traceability_tests.rs**: Validates that trace metadata is correctly generated for every assignment decision.
            - **payroll_traceability_tests.rs**: Verifies the hierarchical resolution and source tracking for payroll "Standard Days".
            - **payroll_batch_tests.rs**: Ensures reliable parallel calculation for multiple employees simultaneously.
3.  **Admin Dashboard (React/Vite)**:
    - **Premium UI Aesthetics**: Modern "Metro-style" interface using Tailwind CSS v4 and glassmorphism.
    - **Decoupled Routing**: Routes are defined in a standalone `AppRoutes.tsx` component, separating navigation logic from the main application shell (`App.tsx`) to enable isolated integration testing via `MemoryRouter`.
    - **Global Modernization**: All administrative pages (Departments, Shifts, Reports, Payroll) follow a consistent card-based layout with high-precision information density.
    - **Payroll Management**: Integrated full-width data grid with sliding detail drawers for granular traceability.
    - **Internationalization**: Full support for English, Vietnamese, and Chinese with centralized i18n management.

4.  **Employee Self-Service (ESS)**:
    - **Dedicated Portal**: Streamlined interface for employees to manage their own data.
    - **Time Tracking**: Real-time clock-in/out via `PunchCard` component with integrated location/status detection.
    - **Attendance & Profile**: Interactive attendance history and profile management with traceability metadata.
    - **Shared Components**: Leverages core design system for visual consistency between Admin and ESS roles.


...
