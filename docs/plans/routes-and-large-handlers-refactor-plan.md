# LeeAttend Routes and Large Handler Refactor Plan

Status: Proposed; no source refactor started.

## 1. Objective

Reduce route/bootstrap and handler-file complexity without changing runtime behavior.

Preserve exactly:

- dependency ownership, construction order, shared repository instances, and lifecycle ownership;
- global middleware order and protected/public router split;
- route literals, HTTP methods, route precedence, handler binding, authorization, status codes, JSON/error contracts, database queries/schema, audit events, logs, metrics, Redis behavior, storage behavior, SSE/WebSocket behavior;
- current working-tree changes outside approved refactor scope.

## 2. Non-goals

- No API path, method, status, auth, JSON, database, migration, observability, Redis, storage, stream, or lifecycle behavior change.
- No generic route table, route DSL, factory, service locator, or DI framework.
- No new abstraction only to remove a few `registerRoute` lines.
- No repository refactor.
- No edits to pre-existing user changes or unrelated untracked files.
- No Rust/Cargo tooling.
- No broad handler rewrite before characterization tests and an incremental boundary decision.

## 3. Mandatory rule compliance

Read: `CLAUDE.md`.
Read: `.leedevkit/templates/CLAUDE.base.md`.
Read: `.agent/rules/project-structure.md`.
Read: `.agent/rules/coding-standards.md`.
Read: `.agent/rules/development-workflow.md`.
Read: `.agent/rules/testing-standards.md`.
Read: `.agent/rules/execution-wrappers.md`.
Read: `.agent/rules/api-governance.md`.
Read: `.agent/rules/architecture-governance.md`.
Read: `.agent/rules/ai-agent-governance.md`.
Read: `.agent/rules/database-rules.md`.
Read: `.agent/rules/access-control.md`.
Read: `.agent/rules/data-governance.md`.

Relevant rules: preserve dependency direction and ownership; introduce abstractions only at meaningful substitution boundaries; protect authorization before business logic; preserve public contracts; run focused and full integration verification; use repository wrappers.

## 4. Working-tree protection

Initial `git status --short --branch` showed:

- deleted: `.git_commit_msg`, `docs/operations/AGENTS.md`;
- modified: `apiserver/internal/repository/payroll.go`, `apiserver/internal/repository/work_policy.go`, `webdashboard/missing-keys.json`, payroll calculation-trace tests/component/locales;
- untracked: `docs/plans/handler-pointer-receiver-refactor-plan.md`.

Keep all above files unchanged. Also preserve `apiserver/internal/repository/shift_swaps.go` if it becomes modified before implementation. Do not reset, revert, stage, or commit any file outside this plan. New plan file is the only requested artifact for this planning task; do not modify the existing pointer-receiver plan.

## 5. Current-state inventory

### 5.1 Router bootstrap

`apiserver/routes.go:20-228` contains `buildRouter` at 229 lines. It currently owns:

1. chi root router creation;
2. revocation/rate-limiter selection from one Redis client;
3. protected router creation with `AuthMiddlewareWithRevocation`;
4. organization authorizer/repository construction;
5. crypto service optional initialization and warning behavior;
6. health setup;
7. repository and handler construction for organization, people, employee/attendance, scheduling, leave, payroll/ESS, workflows/notifications, reports/files/documents/lifecycle, system, monitor, auth, dashboard;
8. feature-flag middleware closure for ESS;
9. public/protected/WS route registration order;
10. outer middleware chain.

Important shared instances:

- `organizationAuthorizer` is reused by many handlers.
- `organizationRepo` is reused by organization routes and the ESS feature-flag lookup closure.
- `attendanceRepo` is reused by attendance and logs handlers and injected into ESS.
- `attendancePolicyRepo` is reused by attendance policy, overtime, and leave behalf behavior.
- `employeeRepo` is reused by employee and assignment handlers.
- `payrollRepo` is reused by payroll and ESS.
- `leaveRepo` is reused by leave and ESS.
- `swapRepo` is reused by scheduling and ESS.
- `systemSettingsRepo` is reused by system admin and `service.EmailService`.
- `scheduleRepo` is reused as repository, lifecycle, shadow, and activation implementation and later in schedule-office worker construction in `main.go`.
- Redis client is shared by auth, rate limiting, revocation, imports, reports, system admin, monitor, and worker lifecycle.
- `cfg.StorageDir` is shared by employee import, attendance logs, and leave import.

Construction order is observable through route precedence and feature/middleware wiring. Preserve order unless a phase proves equivalence with route snapshots.

Optional resources must keep behavior: crypto initialization logs a warning and leaves `Crypto` nil on invalid/empty encryption configuration; S3 client failure leaves file storage nil without changing route registration; Redis absence leaves optional backends nil.

Outer middleware order at `apiserver/routes.go:223-228` is:

`RequestIDMiddleware` → `RateLimitMiddleware` → `LocaleMiddleware` → `CORSMiddleware`.

Protected routes use `AuthMiddlewareWithRevocation` at `apiserver/routes.go:30`. ESS adds `FeatureFlagMiddleware` only to its protected sub-router at `apiserver/routes.go:139-145`. Public and protected route registration currently occurs in the sequence shown at `apiserver/routes.go:56-216`; preserve it.

### 5.2 Route registration

`apiserver/router.go:10-39` defines `registerRoute`, which parses `METHOD path`, calls chi `MethodFunc`, and copies chi URL params into `http.Request.PathValue`. Existing tests protect path values, methods, 404/405 behavior, auth wrapper behavior, and parameter extraction (`apiserver/router_test.go:13-128`). Treat this helper as a compatibility boundary.

There are 424 `registerRoute` calls across 19 production `apiserver/routes_*.go` files:

- `routes_attendance.go` 40
- `routes_auth.go` 10
- `routes_dashboard.go` 14
- `routes_employees.go` 17
- `routes_employment.go` 32
- `routes_ess.go` 18
- `routes_files.go` 20
- `routes_leave.go` 37
- `routes_lifecycle.go` 30
- `routes_monitor.go` 12
- `routes_organization.go` 36
- `routes_overtime.go` 19
- `routes_payroll.go` 45
- `routes_people.go` 22
- `routes_reports.go` 6
- `routes_scheduling.go` 41
- `routes_system.go` 4
- `routes_workflows.go` 12
- `routes_work_policy.go` 9

The files are already domain-oriented. Route literals are explicit and searchable. Current tests cover the shared helper, employment route registration, payroll period/salary-management registration, organization settings alias rejection, and work-policy authorization. No duplicate production route literal was found by exact-literal scan.

Conclusion: do not mass-convert route files. First inventory route signatures and precedence. Only make local, evidence-backed edits where a file contains a real repeated semantic group and explicit literals remain clearer. Likely outcome: keep most or all route registration files unchanged; route readability is currently stronger than line-count reduction.

### 5.3 Large handlers

- `apiserver/internal/api/payroll.go`: 1,708 lines, `PayrollHandler` at `:509-513`, 53 methods in this file; additional `ListSalaryManagement` method in `salary_management.go:67-105`. `PayrollRepository` at `payroll.go:462-508` has a broad 46-method contract. Handler spans payslips/salary management, salary components, settings, incentive policies, rules, periods, calculation, input batches, formula rules, impact comparison, authorization, audit, and error mapping.
- `apiserver/internal/api/monitor.go`: 1,686 lines, `MonitorHandler` at `:348-353`. It combines auth helpers, operational snapshot assembly, Redis reads, durable repository reads, alert CRUD, audit logs, job status, SSE stream, and JWT/WebSocket pre-upgrade auth. `Stream` is a long-lived request loop at `:1030-1079`; `ReportStreamWS` starts at `:1286` and must keep token/auth/upgrade behavior unchanged.
- `apiserver/internal/api/leaves.go`: 1,098 lines, `LeaveHandler` at `:256-262`. It combines leave types/requests/balances/policies/accrual, behalf assignments, Excel import/upload/template, filesystem handling, authorization, and response/error mapping. `ImportRepository` and `BehalfRepository` are optional overrides with fallback type assertions at `:543-560`; preserve this behavior.

Natural boundaries to evaluate, not assume:

- Payroll: transport parsing/validation helpers; salary-management endpoints; formula/input/impact endpoints; period/calculation endpoints. Keep handler method names and fields or use thin forwarding methods only if route binding and public package tests remain stable. Do not split the broad repository interface unless each new module gets a genuinely independent contract and migration cost is justified.
- Monitor: auth/claim extraction; snapshot/query mapping; alert/audit CRUD; streaming adapters. Do not move Redis/client ownership or alter stream timing, cancellation, token precedence, status/error body, or observability.
- Leave: leave type/request/balance/policy operations; import/file operations; behalf operations; shared auth/error mapping. File writes and import repository fallback make import extraction higher risk.

Existing tests are substantial but unevenly grouped:

- payroll: `apiserver/internal/api/payroll_test.go` (~1,610 lines), `payroll_periods_test.go`, `salary_management_test.go`; route tests in `routes_payroll_test.go` and `router_test.go`;
- monitor: `apiserver/internal/api/monitor_test.go` (~624 lines), including auth, snapshot, validation, SSE, alert CRUD/conflicts, audit logs, durable jobs, and WS pre-upgrade auth;
- leave: `apiserver/internal/api/leaves_test.go` (~747 lines), including submit validation/auto-approval, balances/history, policies, accrual/jobs, behalf, import endpoints;
- no test currently exercises full `buildRouter` construction;
- route tests are not a complete route matrix, so characterization coverage is required before moving route bindings or handler methods;
- no single test currently proves global middleware ordering and public/protected/ESS scope together;
- no router-construction test protects optional S3 wiring, readiness dependency wiring, or Redis-degraded construction.

Compatibility registrations that must remain explicit and covered if route files change:

- duplicate attendance adjustment approval registrations in `apiserver/routes_attendance.go`;
- paper overtime endpoint aliases in `apiserver/routes_attendance.go`;
- `PUT` and `PATCH` holiday update aliases in `apiserver/routes_leave.go`.

These may be intentional compatibility behavior. Do not deduplicate without evidence and approval.

## 6. Dependency direction and resource risks

Preserve current direction:

`routes.go`/transport wiring → `internal/api` handlers → repository/service/infra interfaces → concrete repositories/Redis/storage.

Do not make handlers import route/bootstrap code. Do not pass DB connections into new pure domain helpers. Keep authorization in handler boundary before repository calls. Keep audit emission at existing request points. Keep repositories and their shared instances owned by router bootstrap.

Risks requiring explicit review per phase:

- method expressions and compile-time interface assertions, especially `routes_payroll.go:10-12`;
- pointer/value receiver method sets and zero-value handlers used in route tests;
- route precedence for overlapping paths such as `/api/v1/monitor/jobs/{job_id}` versus `/api/v1/jobs/{job_id}`, payroll period routes, and leave/job routes;
- middleware scope: protected/public split, ESS feature flag, global rate limit, revocation, CORS, locale, request ID;
- mutable or long-lived behavior: Redis queues/keys, storage directory writes, SSE ticker/cancellation, WebSocket JWT checks, S3 optional storage;
- audit and redaction: payroll salary-management audit and monitor summary sanitization;
- optional nil dependencies and fallback repository type assertions.

## 7. Risk classification

| Work item | Risk | Reason | Recommendation |
|---|---:|---|---|
| Inventory and characterization | Low | Read-only; establishes contracts | Do first |
| Extract local constructor groups in `routes.go` without changing ownership/order | Medium | Easy to accidentally duplicate repositories or reorder side effects | Do only after dependency map; prefer named local blocks/functions only if they return concrete handler bundle and measurable readability gain |
| Route registration formatting/local grouping | Low–Medium | 424 bindings; route precedence and literal drift possible | Mostly skip; retain explicit calls |
| Extract pure parsing/validation helpers | Low–Medium | Testable if no auth/repo/response behavior moves | Good first handler phase |
| Split Payroll handler modules | High | Broad repository contract, audit, auth, many endpoints/tests | Last; one bounded domain at a time |
| Split Monitor handler modules | High | Redis, SSE, WebSocket, JWT, long-lived request lifecycle | Last; extract pure mapping/auth helpers first |
| Split Leave handler modules | High | filesystem, imports, optional repository fallbacks, PII/reasons | Last; leave import/file path together until characterized |
| Change route helper or generic route abstraction | High/low benefit | Central method/path/PathValue compatibility surface | Skip |

## 8. Test Impact Matrix

| Dimension | Preserve/check |
|---|---|
| Changed behavior | Intended observable change is source organization only; no endpoint behavior change |
| Existing behavior | Every moved endpoint keeps exact path, method, precedence, handler binding, status, body, headers, and errors |
| Inputs/boundaries | malformed UUIDs, query/body limits, missing fields, invalid dates, pagination limits, duplicate payroll inputs, invalid monitor trace/status/state, import files |
| State | nil optional dependencies, existing repositories, partial import, Redis unavailable, stale monitor jobs, already acknowledged/resolved alerts, closed/reopened payroll periods |
| Failures | repository errors, `pgx.ErrNoRows`, authorization failures, crypto/S3/Redis unavailable, storage write/parse failure, SSE cancellation, WS invalid/missing JWT |
| Security | missing user context, forbidden permissions, member fallback, payroll sensitive read, monitor infra/audit permissions, JWT precedence, no auth bypass from extraction |
| Compatibility | route literals and parameter names; `http.Request.PathValue`; JSON/error envelopes; legacy duplicated aliases currently retained; no DB/schema changes |
| Operations | request IDs, rate limiting, locale/CORS order, audit events, redaction, stream cancellation, resource cleanup, worker/lifecycle ownership |

## 9. Incremental implementation phases

No phase starts until prior phase passes its acceptance criteria. Each phase is one reviewable change and must leave a rollback point.

### Phase 0 — Baseline and characterization

**Files expected:** `apiserver/router_test.go`; route-specific tests under `apiserver/routes_*_test.go`; handler tests under `apiserver/internal/api/*_test.go`; no production source change unless a test-only fixture is required.

**Smallest change:** add only missing behavior characterization for route inventory, middleware scope/order, and the highest-risk handler paths. Prefer table-driven tests over broad refactoring. Capture route method/path/handler reachability without asserting private implementation. Add a full `buildRouter` test using existing test configuration/dependency fixtures if available; otherwise document the smallest deterministic fixture needed rather than inventing a new runtime harness.

**Dependency:** none; must respect existing working tree.

**Acceptance criteria:** route inventory has one source-backed record per production registration; protected/public/ESS/WS route scopes are explicit; payroll, monitor, leave success/validation/forbidden/dependency-failure/stream-or-file cases have protection; middleware order and health/readiness dependency wiring are characterized where the current constructor permits; compatibility aliases are preserved; no API behavior changes.

**Focused tests:** use repository-supported `leedevkit test api --unit-only --pattern ...` patterns for route, router, payroll, monitor, and leave tests as applicable. Do not treat focused result as final proof.

**Rollback point:** revert only new test lines; leave current source and user files untouched.

### Phase 1 — Router construction inventory and low-risk readability pass

**Files expected:** `apiserver/routes.go`; optionally one new `apiserver/router_*.go` file only if a cohesive domain construction block materially improves readability; related router tests.

**Smallest change:** group existing local repository/handler construction by domain while preserving the exact existing statements, shared variable reuse, construction order, route registration order, and nil/optional initialization. Prefer no helper function. If a helper is proposed, it must own a cohesive domain, return concrete values, introduce no new interface/factory, and show reduced cognitive load rather than hiding dependencies. Candidate slices are organization, attendance, scheduling, reporting, and system/monitor wiring; each slice must be checked for shared instances before extraction. Keep route registration calls unchanged in this phase.

**Dependency:** Phase 0.

**Acceptance criteria:** `buildRouter` has shorter, clearer domain blocks or remains unchanged with a documented skip; all shared repository identities remain the same; ESS feature-flag closure still uses the same `organizationRepo`; optional crypto/S3/Redis behavior unchanged; health handler dependency wiring is unchanged unless separately approved; middleware order unchanged; no route literal changes; public/protected/WS registration scopes remain unchanged.

**Focused tests:** router tests; build/type checks; route registration tests; exact compile-time assertions. Then required gates listed in Phase 5.

**Rollback point:** revert only Phase 1 files.

**Likely decision:** skip extraction if it merely moves 100 lines into functions with large parameter lists. A small reduction is not worth hiding ownership.

### Phase 2 — Route registration duplication review

**Files expected:** `apiserver/routes_*.go` and corresponding route tests only for a proven local improvement; `apiserver/router.go` remains unchanged unless characterization exposes a defect.

**Smallest change:** keep direct `registerRoute(router, "METHOD /literal", handler.Method)` calls. Only group registrations through an existing domain function when the group makes route ownership clearer and all literals/methods remain visible. Do not introduce route slices, generic registration helpers, path constants, or method/middleware metadata abstractions solely to reduce lines. Preserve known compatibility registrations: duplicate attendance approval binding, paper-OT aliases, and holiday `PUT`/`PATCH` aliases until route-contract evidence proves removal safe.

**Dependency:** Phase 0; Phase 1 if router construction changed.

**Acceptance criteria:** exact route set, methods, parameter names, precedence, and handler bindings match baseline; `registerRoute` still preserves `PathValue`, 404, 405, and protected middleware behavior; known compatibility aliases remain; no duplicate or retired route is introduced; any intentionally unreachable existing handler (including registration-related handlers) is documented as an inventory finding, not silently exposed by refactor.

**Focused tests:** `router_test.go`; `routes_employment_test.go`; `routes_payroll_test.go`; `routes_work_policy_test.go`; organization settings alias test; add table-driven route contract tests only where baseline is missing.

**Rollback point:** revert changed route files/tests only.

**Likely decision:** skip most/all route-file edits. Current explicit literals are already the safer readable form, and exact duplicate scan found no production duplicates.

### Phase 3 — Low-risk handler boundary extraction

**Files expected:** one of `apiserver/internal/api/payroll.go`, `salary_management.go`, `monitor.go`, `leaves.go`, plus colocated tests; optional new same-package files such as a domain-specific handler file if Go package organization improves without changing exported names.

**Smallest change:** extract pure parsing/validation and response mapping helpers first. Keep exported/request handler methods as thin entry points with existing names, receiver type, auth check order, repository calls, audit calls, and response writes. Do not move permission checks into services or generic middleware.

**Dependency:** Phase 0; handler-specific baseline tests.

**Acceptance criteria:** helper inputs/outputs are explicit and testable; no DB/Redis/storage access in pure helpers; authorization remains before business/repository calls; JSON/error/status/header behavior unchanged; no PII is added to logs.

**Focused tests:** existing handler tests plus new table-driven boundary tests for malformed input, forbidden access, repository error mapping, empty collections, and audit/no-store behavior where applicable.

**Rollback point:** revert only the single handler domain and tests changed in the phase.

### Phase 4 — One large-handler domain split at a time

**Files expected:** one domain's existing handler file plus new same-package files and tests; never Payroll, Monitor, and Leave together.

**Smallest change:** choose the boundary with the clearest independent responsibility after Phase 3 evidence:

1. Payroll salary-management or formula/input/impact cluster; keep `PayrollHandler` route-facing methods as wrappers if needed.
2. Monitor alert/audit query mapping only; do not first split SSE/WS.
3. Leave policy/type cluster; keep import/filesystem/behalf behavior in place until separate characterization exists.

Each extracted module must use explicit existing interfaces or a narrowly scoped interface already justified by independent testing. No broad repository-interface rewrite. Keep response DTOs and package-visible types stable unless tests prove no external use.

**Dependency:** Phase 3 and a written before/after route/contract inventory for the chosen cluster.

**Acceptance criteria:** all callers and method expressions compile; route registration still binds same methods; exact API contracts/auth/audit/observability/resource behavior remain unchanged; no copied mutable state; no lifecycle owner moves; test coverage does not regress.

**Focused tests:** chosen domain unit tests, route tests, authorization negative tests, dependency-error tests, and stream/file tests if touched. Run race-enabled tests where the extracted code touches shared state or long-lived streams, using repository-supported wrappers only.

**Rollback point:** revert the chosen domain's files as one unit; do not revert user files.

**Recommended skip:** if an extraction needs a new façade, broad interface split, forwarding layer across most methods, or changes more than one resource/lifecycle boundary, stop and keep the current handler. Complexity reduction is not proven.

### Phase 5 — Verification and completion gate

**Files expected:** no source changes; plan/test files only if a failing acceptance test must be corrected within approved scope.

**Smallest change:** run verification in required order and inspect phase logs, not only summary lines.

**Acceptance criteria and mandatory commands:**

1. focused tests for each completed phase;
2. `git diff --check`;
3. `./scripts/_go-gates.sh --lint-only </dev/null`;
4. `leedevkit test api </dev/null`;
5. if a gate fails, read relevant `.test_logs/` entries and report each phase as pass/fail; never rely on the trailing summary line;
6. verify `git status --short` still shows pre-existing user changes untouched and only approved plan/refactor files changed.

**Rollback point:** phase-specific rollback before merging any later phase. No commit in this planning task.

## 10. Recommended implementation order

1. Baseline/characterization.
2. Router construction readability review; likely no-op or small local grouping.
3. Route registration review; likely no-op.
4. Pure helper extraction for one handler domain.
5. Payroll or Leave policy cluster only if tests show a clean boundary.
6. Monitor alert/query cluster only after stream/WS contracts are separately protected.
7. Leave import/filesystem or Monitor stream/WS last, and only with explicit benefit.
8. Full verification and diff/status review.

## 11. Items to skip unless evidence changes

- Generic route abstraction or route table: no proven duplication benefit; hides literals and precedence.
- `registerRoute` rewrite: existing compatibility helper has direct tests and a PathValue bridge.
- Moving all handler methods into separate files without boundary: file splitting alone does not reduce responsibility or test coupling.
- Splitting broad repository interfaces: likely expands contract and mock churn without clear ownership benefit.
- Extracting Redis/SSE/WebSocket/storage lifecycle into generic modules: high behavior risk and no current measured need.
- Changing route order or middleware placement: explicitly prohibited by scope.

## 12. Findings

1. `routes.go` is long because it is the composition root, not because it contains duplicated business logic. Safe reduction comes from domain-local construction blocks, but helper extraction can hide shared-instance ownership. Treat no-op as valid outcome.
2. Route files already separate domains and use explicit literals. 424 calls are repetitive by nature, but removing them through generic metadata would reduce reviewability. Exact duplicate scan found no production duplicate route literals. Existing compatibility registrations must not be mistaken for safe duplication cleanup: attendance approval, paper overtime aliases, and holiday `PUT`/`PATCH` are deliberate review points.
3. `PayrollHandler`, `MonitorHandler`, and `LeaveHandler` are large for different reasons. Payroll has broad endpoint and repository surface; Monitor owns Redis plus long-lived streaming/JWT behavior; Leave mixes policy/request operations with filesystem import fallback. One shared extraction pattern is unsafe.
4. Existing handler tests are stronger than route matrix tests. Characterization must precede structural changes, especially for route precedence, status/error mapping, authorization, optional dependencies, SSE cancellation, WebSocket pre-upgrade auth, and import/file behavior.
5. Source is ground truth. Graph coverage is best-effort; `apiserver/internal/infra/external`, storage, repository internals, and some parsed ranges are incomplete or excluded. Any implementation must re-read affected source ranges and not infer behavior solely from graph results.
6. Existing working-tree changes include payroll/work-policy repository work and frontend artifacts. They remain out of scope and must not be staged or altered.

## 13. Plan completion condition

This document is complete as a plan only. No source was edited, no refactor was run, no user file was staged/reset/reverted, and no commit was created.
