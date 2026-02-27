# LeeAttend — Source Base

## Repository Structure

```
leeattend/
├── apiserver/                    # Rust API Server
│   ├── Cargo.toml               # Dependencies and build config
│   ├── src/
│   │   ├── main.rs              # Entry point: DB pool, migrations, server startup
│   │   ├── lib.rs               # App state, router construction, auth middleware
│   │   ├── errors.rs            # Error types and HTTP error responses
│   │   │
│   │   ├── handlers/            # HTTP request handlers (17 modules)
│   │   │   ├── auth.rs          # Login, register, profile, refresh
│   │   │   ├── employees.rs     # Employee CRUD + bulk operations
│   │   │   ├── shifts.rs        # Shift CRUD, rosters, schedules
│   │   │   ├── holidays.rs      # Holiday calendar CRUD
│   │   │   ├── leaves.rs        # Leave requests, balances, approvals
│   │   │   ├── leave_types.rs   # Leave type + policy configuration
│   │   │   ├── overtime.rs      # OT summaries, claims, approvals
│   │   │   ├── report_export.rs # Async Excel export jobs
│   │   │   ├── report_websocket.rs  # WebSocket progress streaming
│   │   │   ├── agents.rs        # Agent registration and heartbeat
│   │   │   ├── ess.rs           # Employee self-service endpoints
│   │   │   ├── working_patterns.rs  # Working pattern CRUD
│   │   │   ├── assignment_rules.rs  # Auto shift assignment
│   │   │   ├── debug.rs         # Dev-only debug endpoints
│   │   │   ├── worker_monitor.rs    # Background job monitoring
│   │   │   ├── common.rs        # Shared types (MemberContext, pagination)
│   │   │   └── attendance/      # Sub-module for attendance handlers
│   │   │       └── overtime.rs  # OT-specific handlers
│   │   │
│   │   ├── models/              # Data structures (17 modules)
│   │   │   ├── attendance.rs    # DailyAttendanceReport, AttendanceDailySummary
│   │   │   ├── employee.rs      # Employee, EmploymentPeriod
│   │   │   ├── shift.rs         # Shift, ShiftRoster
│   │   │   ├── holiday.rs       # Holiday, HolidayOvernightStrategy
│   │   │   ├── payroll.rs       # Payslip, PayrollSegment, IncentiveTier
│   │   │   ├── leave.rs         # LeaveType, LeavePolicy, LeaveBalance
│   │   │   ├── workspace.rs     # Workspace configuration
│   │   │   ├── auth.rs          # User, AuthResponse, JWT claims
│   │   │   ├── overtime.rs      # OvertimeRule, OTPayResult
│   │   │   ├── schedule.rs      # EffectiveSchedule, ScheduleTrace
│   │   │   ├── department.rs    # Department, hierarchy
│   │   │   ├── agent.rs         # AuthorizedAgent, DeviceStatus
│   │   │   ├── ess.rs           # EmployeeProfile, EmergencyContact
│   │   │   ├── time_tracking.rs # TimeEntry
│   │   │   ├── enums.rs         # Shared enums
│   │   │   └── common.rs        # Pagination, shared traits
│   │   │
│   │   ├── services/            # Business logic (24 modules)
│   │   │   ├── auth_service.rs
│   │   │   ├── attendance_log_service.rs
│   │   │   ├── attendance_calculator.rs
│   │   │   ├── attendance_summary_service.rs
│   │   │   ├── attendance_import_service.rs
│   │   │   ├── attendance_adjustment_service.rs
│   │   │   ├── pairing_service.rs / pairing_algo.rs
│   │   │   ├── report_service.rs
│   │   │   ├── report_coordinator.rs
│   │   │   ├── excel_writer.rs
│   │   │   ├── payroll_service.rs
│   │   │   ├── payroll_evaluator.rs
│   │   │   ├── leave_service.rs / leave_accrual_service.rs
│   │   │   ├── shift_service.rs
│   │   │   ├── working_pattern_service.rs
│   │   │   ├── rule_evaluator.rs
│   │   │   ├── employee_service.rs
│   │   │   ├── workspace_service.rs
│   │   │   ├── agent_service.rs / agent_log_buffer.rs
│   │   │   └── time_tracking_service.rs
│   │   │
│   │   ├── repositories/        # Database access layer
│   │   │   └── holiday_repository.rs  # Holiday CRUD + batch lookup
│   │   │
│   │   ├── domain/              # Pure business logic
│   │   │   └── policy/
│   │   │       └── overtime.rs  # OvertimeEngine with holiday rules
│   │   │
│   │   ├── cache/               # In-memory caching (Moka)
│   │   ├── mw/                  # Custom middleware
│   │   ├── queue/               # Redis job queue
│   │   ├── worker/              # Background job worker
│   │   ├── utils/               # Utility functions
│   │   ├── agent_manager/       # Agent lifecycle management
│   │   └── dev_initialization/  # Dev mode seeding
│   │
│   ├── migrations/              # SQL migrations (89 files)
│   └── tests/
│       └── integration/         # Integration tests
│           ├── main.rs          # Test entry point
│           ├── common/          # Test helpers (TestApp, setup_test_db)
│           ├── holiday/         # Holiday integration tests
│           ├── attendance/      # Attendance tests
│           └── leave/           # Leave tests
│
├── agent-main/                  # Rust Agent (On-Premise)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # Agent entry point
│       ├── bin/                 # Binary entry points
│       ├── central_client/      # API server communication
│       ├── database/            # SQLite local storage
│       └── memory_store/        # In-memory attendance store
│
├── webdashboard/                # React Frontend
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   └── src/
│       ├── main.tsx             # React entry point
│       ├── App.tsx              # Root component with providers
│       ├── AppRoutes.tsx        # Route definitions
│       ├── lib/
│       │   ├── api/             # API client layer (17 modules)
│       │   │   ├── index.ts     # Re-exports
│       │   │   ├── types.ts     # TypeScript interfaces
│       │   │   ├── config.ts    # API base URL
│       │   │   ├── helpers.ts   # apiFetch wrapper
│       │   │   ├── holidays.ts  # Holiday CRUD client
│       │   │   ├── attendance.ts / auth.ts / employees.ts / ...
│       │   │   └── payroll.ts / shifts.ts / workspaces.ts
│       │   └── utils/           # Utility functions
│       ├── pages/               # Page components (20 pages)
│       │   ├── DashboardPage.tsx
│       │   ├── ReportsPage.tsx
│       │   ├── PayrollPage.tsx
│       │   ├── EmployeesPage.tsx
│       │   ├── ShiftsPage.tsx / WorkingPatternsPage.tsx
│       │   ├── settings/        # Settings sub-pages (8 pages)
│       │   │   ├── SettingsLayout.tsx
│       │   │   ├── GeneralSettings.tsx
│       │   │   ├── HolidaysPage.tsx
│       │   │   └── ...
│       │   └── ess/ / approvals/  # ESS and approval pages
│       ├── components/          # Reusable components
│       │   ├── common/          # AuthGuard, WorkspaceGuard
│       │   ├── layout/          # MainLayout, InfraLayout
│       │   └── features/        # Feature-specific components
│       │       └── reports/     # DailyReportTable, etc.
│       ├── hooks/               # Custom React hooks
│       ├── i18n/                # Internationalization (en/vi)
│       └── styles/              # Global CSS
│
├── scripts/                     # Automation scripts (17 files)
├── docs/                        # Documentation (29 files)
├── docker-compose.yml           # Base Docker config
├── docker-compose.dev.yml       # Dev overlay (hot-reload)
├── docker-compose.test.yml      # Test overlay (isolated)
├── docker-compose.ci.yml        # CI overlay
├── run-tests.sh                 # Unified test orchestrator (33KB)
└── .github/                     # GitHub Actions CI/CD
```

## Entry Points

| Component | Entry Point | Description |
|---|---|---|
| API Server | `apiserver/src/main.rs` | DB pool → migrations → server startup |
| Agent | `agent-main/src/main.rs` | Device connection → sync loop |
| Web Dashboard | `webdashboard/src/main.tsx` | React root render |
| Test Runner | `./run-tests.sh` | Docker-based test orchestration |

## Key Modules

| Module | Lines | Importance | Description |
|---|---|---|---|
| `lib.rs` | 661 | ⭐⭐⭐ | Router construction, app state, auth middleware |
| `report_service.rs` | ~600 | ⭐⭐⭐ | Daily/summary attendance reports with holiday logic |
| `payroll_service.rs` | ~400 | ⭐⭐⭐ | Monthly payslip calculation engine |
| `attendance_summary_service.rs` | ~400 | ⭐⭐⭐ | Daily summary recalculation with OT detection |
| `excel_writer.rs` | 1041 | ⭐⭐ | Multi-sheet Excel generation |
| `pairing_service.rs` | ~300 | ⭐⭐ | Punch-in/out pairing algorithms |
| `ReportsPage.tsx` | ~500 | ⭐⭐ | Main reports UI with filtering and export |
| `DailyReportTable.tsx` | 355 | ⭐⭐ | Data table with status badges |
| `run-tests.sh` | 33KB | ⭐⭐⭐ | Test infrastructure orchestrator |
