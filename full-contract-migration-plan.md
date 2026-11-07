# Full Contract Migration Plan: Complete 403-Route Port from Rust to Go

Status: Approved

## Goal
Sequentially implement, verify, and cut over all remaining 269 contract routes and domain operations to complete ledger tasks `P6-01` through `P9-10` with zero contract, database, security, or behavioral drift.

---

## Current State & Metrics
- **Total Contract Routes:** 403 (SSOT: `endpoint-matrix.csv`)
- **Go Registered Routes:** 157
- **Covered Contract Routes:** 134 (33.2%)
- **Missing Contract Routes:** 269 (66.8%)
- **Gates Status:** 100% Passing (`gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, `go test -race`, `./leedevkit test api-go`).
- **Known Parity Gaps to Backfill:**
  - Work location audit events (`location.created`, `location.updated`, `location.deleted`)
  - Global vs department-scoped `managed_department_ids` in `/me/permissions`

---

## Global Constraints & Interfaces
1. **Zero External Compute / Microservice:** Attendance pairing, matching, timezone, and payroll rules remain deterministic pure Go packages in `internal/domain/`.
2. **Strict Financial & Time Arithmetic:** `shopspring/decimal` for money, rates, and balances. `float32`/`float64` strictly forbidden in domain calculation paths.
3. **No ORM:** PostgreSQL access exclusively via `pgx/v5` connection pools (`pgxpool`) and SQL queries.
4. **Tenant Isolation:** Every tenant database query MUST include explicit `workspace_id = $...` predicates and execute against the resolved workspace connection pool (`TenantDB`).
5. **Quality Gates:** Every slice MUST pass `./scripts/_go-gates.sh` and `./leedevkit test api-go` before commit.
6. **Parity Evidence:** Every domain slice must update `docs/migration/rust-go/evidence-*.md`, refresh `endpoint-coverage-gap.md`, and record verification results.

---

## Staged Implementation Plan (Batched Slices)

### Batch 1: Core Foundation & Master Data (45 routes)
Target domains: P6-02, P6-03, P6-11
- [ ] **Task 1.1: Employers & Recruitment Sources (10 routes)**
  - Routes: `GET/POST/PUT/DELETE /api/v1/workspaces/{ws_id}/employers[/{id}]`, `GET/POST/PUT/DELETE /api/v1/workspaces/{ws_id}/recruitment-sources[/{id}]`, `GET /api/v1/dashboard/recruitment-sources`.
  - Interfaces: `internal/api/employers.go`, `internal/api/recruitment_sources.go`, `internal/repository/employers.go`, `internal/repository/recruitment_sources.go`.
  - Constraints: Circular parent employer prevention, active employee reference checks prior to deletion, audit logging.
  - Verify: `./scripts/_go-gates.sh` passes; handler tests verify parent loop rejection and 404/409 handling.
- [x] **Task 1.2: System Admin & SMTP (Retained routes per ADR 0004)**
  - Routes: `GET/PUT/POST /api/v1/system/smtp/*`, `GET /api/v1/system/health`, `GET /api/v1/monitor/*`.
  - Note: Per ADR 0004 and Standalone On-Prem Plan, legacy SaaS surfaces (`/api/v1/system/tenants/*`, `/api/v1/system/roles`, `/api/v1/system/users/{user_id}/roles`, `/api/v1/system/db-clusters/*`) are removed. Retained system routes are guarded by standalone organization `admin` RBAC.
  - Interfaces: `internal/api/system_admin.go`, `internal/repository/system_admin.go`.
  - Verify: Unauthenticated or non-admin calls return 401/403.
- [ ] **Task 1.3: Workspace Roles, Members & Settings Remaining (19 routes)**
  - Routes: `/workspaces/{ws_id}/roles/*`, `/workspaces/{ws_id}/members/*`, `/workspaces/{ws_id}/settings/*`.
  - Interfaces: `internal/api/workspaces.go`, `internal/repository/workspaces.go`.
  - Verify: Member invite/removal, role assignments, and workspace setting JSON round-trip tests pass.

### Batch 2: Employee Lifecycles, Contracts & Assignments (63 routes)
Target domains: P6-03, P6-11
- [ ] **Task 2.1: Working Conditions & Employee Departments (8 routes)**
  - Routes: `GET/POST/DELETE /employees/{id}/conditions[/{cond_id}]`, `GET/POST /employees/{id}/departments`, `GET /employees/{id}/compensation-history`.
  - Interfaces: `internal/api/employee_conditions.go`, `internal/repository/employee_conditions.go`.
  - Verify: Condition type validation, department transfer audit log, and compensation history ordering.
- [ ] **Task 2.2: Contracts & Employments CRUD (15 routes)**
  - Routes: `GET /contracts`, `GET/POST/GET/PATCH/DELETE /employees/{employee_id}/contracts[/{contract_id}]`, `GET/POST/GET/PATCH/DELETE /employees/{employee_id}/employments[/{employment_id}]`.
  - Interfaces: `internal/api/contracts.go`, `internal/api/employments.go`, `internal/repository/employments.go`.
  - Constraints: Temporal employment date ranges, status enum parity (`active`, `terminated`, `on_leave`).
  - Verify: Date conflict rejection, soft-deletion handling, and employment transition tests pass.
- [ ] **Task 2.3: Secondments / Assignments & Assignment Rules (14 routes)**
  - Routes: `GET/POST/GET/PATCH/DELETE /employees/{employee_id}/assignments[/{assignment_id}]`, `/assignment-rules/*`, `/assignment-history`.
  - Interfaces: `internal/api/employee_assignments.go`, `internal/repository/employee_assignments.go`.
  - Verify: Secondment overlap checks, primary vs secondary assignment resolution.
- [ ] **Task 2.4: Employment Phases, Lifecycles & Templates (18 routes)**
  - Routes: `/employees/{user_id}/phases/*`, `/employments/{employment_id}/phases/*`, `/lifecycle-templates/*`, `/employments/{emp_id}/lifecycle/*`.
  - Interfaces: `internal/api/lifecycle.go`, `internal/repository/lifecycle.go`.
  - Verify: Phase status transition (`pending` -> `in_progress` -> `completed`), checklist validation.
- [ ] **Task 2.5: Employee Excel Upload & Termination Import (8 routes)**
  - Routes: `/employees/upload`, `/employees/import-termination/preview|confirm|template`.
  - Interfaces: `internal/api/employee_import.go`, `internal/service/employee_import.go`.
  - Constraints: Max upload body limit, header validation, atomic batch insertion.
  - Verify: Malformed Excel error mapping, preview vs confirm side-effect test.

### Batch 3: Attendance, Time Tracking & Roster / Shifts (48 routes)
Target domains: P6-04, P6-05
- [ ] **Task 3.1: Attendance Logs, Imports & Paper OT Uploads (12 routes)**
  - Routes: `/attendance/upload`, `/attendance/logs/upload`, `/attendance/bulk-upload-paper-ot`, `/attendance/paper-forms/*`.
  - Interfaces: `internal/api/attendance_import.go`, `internal/repository/attendance_import.go`.
  - Constraints: In-process duplicate punch deduplication, staging log claiming on employee update.
  - Verify: Duplicate punch rejection, staging log ingestion tests pass.
- [ ] **Task 3.2: Attendance Rules & Policy Assignments (10 routes)**
  - Routes: `GET/PATCH /attendance/rules`, `GET/POST/DELETE /attendance/policies/assignments[/{assignment_id}]`, `/attendance/provenance/*`.
  - Interfaces: `internal/api/attendance_policies.go`, `internal/repository/attendance_policies.go`.
  - Verify: Overlapping policy assignment exclusion constraint, policy inheritance.
- [ ] **Task 3.3: Overtime Policies, Requests & Calculations (12 routes)**
  - Routes: `/overtime/policies/*`, `/overtime/requests/*`, `/overtime/bulk-requests`.
  - Interfaces: `internal/api/overtime.go`, `internal/repository/overtime.go`.
  - Constraints: Overtime rate calculation via pure domain arithmetic.
  - Verify: Multi-tier overtime calculation unit tests and approval workflow triggers pass.
- [ ] **Task 3.4: Shifts, Shift Swaps & Schedules (14 routes)**
  - Routes: `/shifts/*` (remaining), `/shift-swaps/*`, `/schedules/*`.
  - Interfaces: `internal/api/shift_swaps.go`, `internal/repository/shift_swaps.go`.
  - Constraints: Shift swap conflict detection, roster cache invalidation outbox.
  - Verify: Swap state machine transitions and cache invalidation event generation.

### Batch 4: Leaves, ESS, Holidays & Approvals (35 routes)
Target domains: P6-06
- [x] **Task 4.1: Holidays Management & Leave Processing (7 routes)**
  - Routes: `GET/POST/GET/PUT/PATCH/DELETE /holidays[/{id}]`, `POST /holidays/{id}/process-leave`.
  - Interfaces: `internal/api/holidays.go`, `internal/repository/holidays.go`.
  - Constraints: Overlapping holiday date validation, automated leave deduction on holiday processing.
  - Verify: Holiday calendar queries, leave accrual adjustments on holiday.
- [ ] **Task 4.2: Leave Balances, Adjustments & Excel Upload (10 routes)**
  - Routes: `GET /leave/balances`, `POST /leave/upload-excel`, `GET/POST /employees/{id}/leave/adjust`, `GET /employees/{id}/leave/history`, `/leave/policies/assignments/*`.
  - Interfaces: `internal/api/leave_balances.go`, `internal/repository/leave_balances.go`.
  - Constraints: Exact `shopspring/decimal` balance accrual and rounding.
  - Verify: Decimal balance calculation without float inaccuracies, audit event for manual adjustments.
- [ ] **Task 4.3: ESS (Employee Self-Service) Endpoints (18 routes)**
  - Routes: `/ess/leave/*`, `/ess/shift-swaps/*`, `/ess/payslips/*`, `/ess/signatures/*`, `/ess/profile/*`.
  - Interfaces: `internal/api/ess.go`, `internal/repository/ess.go`.
  - Constraints: User context strictly resolved to linked employee record; cross-employee access denied.
  - Verify: Self-access isolation tests (User A cannot view User B's ESS records).

### Batch 5: Payroll, Work Buckets & Salary Components (25 routes)
Target domains: P6-08
- [ ] **Task 5.1: Salary Bases & Salary Components (8 routes)**
  - Routes: `GET/POST /salary-bases`, `/salary-bases/{base_id}/members/*`, `GET/POST /employments/{id}/salary-components`.
  - Interfaces: `internal/api/salary_components.go`, `internal/repository/salary_components.go`.
  - Constraints: Non-negative monetary amounts, decimal rate validation.
  - Verify: Salary base component summation and formula validation tests.
- [ ] **Task 5.2: Payroll Settings & Department/Employee Configs (9 routes)**
  - Routes: `GET/PATCH/GET /payroll-settings[/*]`, `GET/POST /employees/{id}/payroll-config`, `/payroll-rules/{rule_id}/assignments/*`.
  - Interfaces: `internal/api/payroll_config.go`, `internal/repository/payroll_config.go`.
  - Verify: Configuration hierarchy resolution (Workspace -> Department -> Employee).
- [ ] **Task 5.3: Payroll Calculation Execution & Export (8 routes)**
  - Routes: `POST /employees/{id}/calculate-payroll`, `/payroll/runs/*`, `/payroll/export/*`.
  - Interfaces: `internal/api/payroll_execution.go`, `internal/repository/payroll_execution.go`.
  - Constraints: Pure domain payroll package calculation, zero DB/network I/O during calculation step.
  - Verify: Golden fixtures match Rust payroll calculation down to the cent.

### Batch 6: Analytics, Reports, Files & Agent Devices (43 routes)
Target domains: P6-07, P6-09, P6-10, P6-11
- [ ] **Task 6.1: Dashboard Analytics & Metric Summaries (10 routes)**
  - Routes: `/dashboard/executive-overview`, `/dashboard/overtime-breakdown`, `/dashboard/workspace-stats`, `/dashboard/department-attendance`, `/dashboard/unexcused-absences`, etc.
  - Interfaces: `internal/api/dashboard_analytics.go`, `internal/repository/dashboard_analytics.go`.
  - Constraints: Bound query count (no N+1), date range bounds enforcement.
  - Verify: Empty workspace vs populated workspace aggregation tests.
- [ ] **Task 6.2: Document Folders & Template Management (16 routes)**
  - Routes: `/documents/folders/*`, `/documents/templates/*`, `/documents/acknowledgements/*`.
  - Interfaces: `internal/api/documents.go`, `internal/repository/documents.go`.
  - Verify: Folder hierarchy loops prohibited, template variable substitution.
- [ ] **Task 6.3: Monitor, Stream & Alert Management (9 routes)**
  - Routes: `/monitor/overview`, `/monitor/jobs`, `/monitor/correlations`, `/monitor/stream`, `/monitor/alerts/*`.
  - Interfaces: `internal/api/monitor.go`, `internal/repository/monitor.go`.
  - Verify: Stream SSE/WebSocket connection lifecycle, alert acknowledgement state changes.
- [ ] **Task 6.4: Hardware Devices, Agents & WS Hub (8 routes)**
  - Routes: `/devices/*`, `/authorized-agents/*`, `/agents`, `/agent/{agent_id}/command`, `/ws`.
  - Interfaces: `internal/api/devices.go`, `internal/repository/devices.go`, `internal/infra/hub/`.
  - Constraints: Mutual TLS or token-based device authentication, heartbeat tracking.
  - Verify: Agent registration, command dispatch, and WebSocket heartbeat tests pass.

### Batch 7: Quality Gate Closure & Ledger Sign-off (Tasks P6-12 -> P9-10)
- [ ] **Task 7.1: P6-12 Domain Completion Review**
  - Verify: `endpoint-matrix.csv` has 0 missing routes (403/403 covered); `endpoint-coverage-gap.md` reports 0 missing.
- [x] **Task 7.2: Phase 7 Differential, Security & Performance Proof (P7-01 -> P7-09)**
  - Run all fixture requests against Rust and Go; verify 0 unexplained mismatches (`scripts/run_differential_tests.py`, `docs/migration/rust-go/evidence-p7-verification.md`).
  - Run database mutation comparison and Redis TTL/invalidation comparison (`apiserver/internal/api/resilience_test.go`).
  - Run tenant isolation security suite (`./scripts/security_audit.sh`, `apiserver/internal/api/security_verification_test.go`).
  - Run load/latency comparison against Rust baseline (`scripts/run_performance_benchmarks.py`, `apiserver/internal/api/performance_benchmark_test.go`).
- [ ] **Task 7.3: Phase 8 Deployment, Shadow & Rollback Drills (P8-01 -> P8-09)**
  - Multi-stage Go production container build (`.compose/go/Dockerfile`).
  - Verify staging shadow comparison and failure drills (DB down, Redis down, timeout fail-closed).
  - Verify zero-schema-change rollback back to Rust.
- [ ] **Task 7.4: Phase 9 Cutover, Retirement & Archive (P9-01 -> P9-10)**
  - Execute `./leedevkit test all` across entire repository.
  - Remove obsolete Rust `apiserver/` route modules post-rollback window.
  - Final ledger archive in `docs/migration/rust-go/final-evidence.md`.

---

## Review Focus & Critical Risks
1. **Scope Bleed:** Avoid reimplementing frontend or adding unrequested helper endpoints; implement ONLY exact contract rows.
2. **Nullable Field Semantics:** Rust uses `Option<Option<T>>` for partial updates where `null` means unsetting and omitted means keeping existing. Go handlers must preserve this using pointers to pointers (`**T`).
3. **Audit Event Parity:** Every mutating endpoint in Rust that emits `AuditHelper::log` must be mapped to Go `audit.Writer.Emit`.
4. **Tenant Connection Leaks:** Ensure every repository method releases its acquired transaction or pool connection back to the manager on error.

---

## Done When
1. All 403 contract routes in `endpoint-matrix.csv` are implemented and registered in Go `apiserver/main.go`.
2. `endpoint-coverage-gap.md` shows 0 missing routes.
3. All build gates (`./scripts/_go-gates.sh`) pass with 0 errors and 0 race conditions.
4. Full integration test suite `./leedevkit test all` passes completely.
5. All tasks in `rust-go-migration-task-ledger.md` are marked complete with accompanying evidence.
