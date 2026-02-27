# LeeAttend — System Architecture

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Web Dashboard                             │
│                   (React 19 + Vite + Tauri)                      │
│          Pages: Dashboard, Reports, Payroll, Settings            │
└───────────────────────────┬─────────────────────────────────────┘
                            │ HTTPS / WebSocket
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Nginx Reverse Proxy                           │
│               (dev: nginx-dev.conf, port 80)                     │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    API Server (Rust/Axum)                         │
│                     Port 3000                                    │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  ┌────────────────┐ │
│  │ Handlers │→ │ Services │→ │  Repos    │→ │  PostgreSQL    │ │
│  │ (REST)   │  │ (Logic)  │  │ (SQL)     │  │  (sqlx)        │ │
│  └──────────┘  └──────────┘  └───────────┘  └────────────────┘ │
│                                                                  │
│  ┌──────────────────┐  ┌────────────────┐  ┌────────────────┐  │
│  │ Domain Policies   │  │ Job Worker     │  │ Redis          │  │
│  │ (OT, Pairing)    │  │ (Background)   │  │ (Queue/Cache)  │  │
│  └──────────────────┘  └────────────────┘  └────────────────┘  │
└───────────────────────────┬─────────────────────────────────────┘
                            │ WebSocket / HTTP
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Agent Main (Rust)                              │
│               On-premise device gateway                          │
│                                                                  │
│  ┌──────────────────┐  ┌────────────────┐  ┌────────────────┐  │
│  │ Central Client    │  │ Memory Store   │  │ SQLite DB      │  │
│  │ (API sync)       │  │ (in-memory)    │  │ (local cache)  │  │
│  └──────────────────┘  └────────────────┘  └────────────────┘  │
│                            │                                     │
│                            ▼                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │            ZKTeco Biometric Devices                        │   │
│  │         (TCP/UDP via rustzk protocol)                     │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Layered Architecture (API Server)

### Layer 1: Handlers
HTTP request/response layer. Each handler module maps to a REST resource.

| Module | Endpoints | Auth |
|---|---|---|
| `auth.rs` | Login, register, profile, refresh token | Public / JWT |
| `employees.rs` | Employee CRUD, department history | Admin |
| `shifts.rs` | Shift CRUD, rosters, schedules | Admin |
| `holidays.rs` | Holiday CRUD | Admin |
| `leaves.rs` / `leave_types.rs` | Leave requests, balances, policies, types | Admin / Self |
| `overtime.rs` | OT summaries, claims, approvals | Admin / Self |
| `report_export.rs` | Excel report generation (async jobs) | Admin |
| `report_websocket.rs` | Real-time report progress via WebSocket | Admin |
| `agents.rs` | Agent registration, heartbeat | Agent token |
| `ess.rs` | Employee Self-Service (profile, attendance) | Employee |
| `worker_monitor.rs` | Background job monitoring | Admin |

### Layer 2: Services
Business logic layer. Services orchestrate multiple repositories and domain policies.

| Service | Responsibility |
|---|---|
| `auth_service` | JWT generation/validation, password hashing |
| `attendance_log_service` | Raw log ingestion, device sync |
| `pairing_service` | Punch-in/punch-out pairing algorithms |
| `attendance_calculator` | Work unit calculation, rounding |
| `attendance_summary_service` | Daily summary recalculation, OT detection |
| `report_service` | Daily/summary attendance reports, holiday-aware status |
| `report_coordinator` | Async Excel export job coordination |
| `payroll_service` | Monthly payslip calculation, OT pay, segments |
| `payroll_evaluator` | Incentive tier matching |
| `excel_writer` | Multi-sheet Excel generation |
| `leave_service` / `leave_accrual_service` | Leave balance, accrual, carry-over |
| `shift_service` / `working_pattern_service` | Schedule resolution |
| `agent_service` | Agent lifecycle, device management |
| `employee_service` | Employee CRUD, employment periods |
| `workspace_service` | Multi-workspace management |

### Layer 3: Domain Policies
Pure business logic without I/O dependencies.

| Policy | Location | Purpose |
|---|---|---|
| `OvertimeEngine` | `domain/policy/overtime.rs` | Rule-based OT calculation with holiday multipliers |
| `RuleEvaluator` | `services/rule_evaluator.rs` | Assignment rule matching |
| `PairingAlgo` | `services/pairing_algo.rs` | Multiple pairing strategies (greedy, optimal) |

### Layer 4: Repositories
Database access layer using `sqlx` with compile-time verified queries.

### Layer 5: Models
Shared data structures used across all layers. 17 model modules covering all domain entities.

## Design Patterns

1. **Compile-time SQL verification** — `sqlx::query!` / `sqlx::query_as!` macros verify queries against live DB during compilation
2. **Middleware-based auth** — `UserId` extractor and `MemberContext` for role-based access
3. **Background job queue** — Redis-backed worker pool for async report generation and batch operations
4. **WebSocket streaming** — Real-time progress updates for long-running export jobs
5. **Temporal data** — Effective dates on shifts, salary components, and department assignments
6. **Multi-workspace isolation** — All queries scoped by `workspace_id`
7. **In-memory caching** — Moka cache for frequently accessed data (workspace settings, shift lookups)

## Data Flow: Attendance Processing

```
ZKTeco Device → Agent (rustzk) → API Server (log ingestion)
                                        ↓
                                Raw Attendance Logs
                                        ↓
                                Pairing Service (punch in/out matching)
                                        ↓
                                Time Entries
                                        ↓
                                Attendance Calculator (work units, rounding)
                                        ↓
                                Attendance Summary (daily aggregation)
                                        ↓
                              ┌─────────┴──────────┐
                              ↓                    ↓
                         Report Service      Payroll Service
                         (daily/summary)     (monthly payslip)
                              ↓                    ↓
                         Excel Export        Payslip Generation
```
