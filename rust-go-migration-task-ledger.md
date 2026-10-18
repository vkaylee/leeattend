# Rust-to-Go `apiserver` Migration — Complete Task Ledger

Status: Awaiting approval
Parent plan: `rust-go-migration-plan.md`

This ledger is the restartable execution source. After context compaction, continue from the first unchecked task whose dependencies are complete. Do not infer completion from code existence; mark a task complete only after its listed verification passes.

## Fixed scope

- Replace Rust/Axum `apiserver/` with Go.
- Preserve `attendance-compute/`; it is already a Go gRPC dependency and is not a migration target.
- Preserve `agent-main/`, `webdashboard/`, PostgreSQL schema/migrations, Redis contracts, protobuf, OpenAPI behavior, and deployment contracts unless an approved decision changes scope.
- Keep Rust and Go deployable in parallel until cutover is proven.

## Global completion rules

- Every task records evidence in `docs/migration/rust-go/` or the relevant test log.
- Every endpoint, worker, job, consumer, producer, and external side effect gets one parity row.
- No task may silently change API status codes, error codes, JSON nullability, pagination, auth semantics, tenant filters, transaction boundaries, decimal/time behavior, Redis keys, queue payloads, or audit events.
- No database migration is required for the initial port. Any schema change requires a separate approved decision and reversible migration.
- Use project wrappers and redirect output to `logs/`; use `./leedevkit test` for repository tests.

## Phase -1 — Context-loss and prerequisite recovery

- [ ] R-01 Verify required migration sources exist: `apiserver/`, current OpenAPI/protobuf contracts, `attendance-compute/` source or approved service artifact, deployment definitions, and existing integration-test fixtures. Depends: none. Evidence: `docs/migration/rust-go/recovery-check.md`. Verify: paths and commit SHAs are recorded.
- [ ] R-02 Recover missing `attendance-compute/`, `proto/`, and ADR sources from an approved remote/tag/artifact, or obtain an explicit scope decision to treat the gRPC contract as an external immutable dependency. Depends: R-01. Evidence: source commit/tag or approved external-contract record. Verify: protobuf descriptors, mTLS/JWT metadata, timeout/retry, digest/rules, and error mapping are testable.
- [ ] R-03 Recover or regenerate the migration ledger and plan if context compaction occurs. Depends: R-01. Evidence: current ledger SHA plus task-status row. Verify: resume from the first unchecked task; never infer completed work from prose or filesystem timestamps.
- [ ] R-04 Block implementation if any required source, contract, fixture, or rollback artifact is unavailable. Depends: R-01 through R-03. Evidence: blocker entry in `decisions.md`. Verify: no Go implementation begins while status is `blocked`.

## Phase 0 — Governance and restart checkpoint

- [ ] P0-01 Approve scope: `apiserver/` only; preserve existing `attendance-compute/`; preserve schema and consumers. Depends: none. Evidence: approved plan status. Verify: scope matches this ledger.
- [ ] P0-02 Choose Go HTTP/DB stack and record ADR. Default: standard `net/http` or existing approved lightweight router plus `pgx`/`sqlc`; avoid ORM behavior drift. Depends: P0-01. Evidence: ADR with rejected alternatives. Verify: package boundary and transaction strategy are explicit.
- [ ] P0-03 Create migration ownership map: task ID, owner, status, files, dependency, evidence, rollback note. Depends: P0-01. Evidence: `docs/migration/rust-go/task-status.csv`. Verify: every task ID in this ledger appears once.
- [ ] P0-04 Define status vocabulary: `not-started`, `in-progress`, `blocked`, `implemented`, `verified`, `cutover`, `retired`. Depends: P0-03. Verify: no ambiguous statuses such as `probably-done`.
- [ ] P0-05 Define decision log and escalation rule for contract mismatches. Depends: P0-01. Evidence: `docs/migration/rust-go/decisions.md`. Verify: every mismatch has owner, impact, decision, and test consequence.

## Phase 1 — Repository and runtime inventory

- [ ] P1-01 Inventory all route modules under `apiserver/src/routes/` and map each route to handlers, services, repositories, models, permissions, tests, and frontend consumers. Depends: P0-01. Evidence: `endpoint-matrix.csv`. Verify: every route has exactly one row per HTTP operation.
- [ ] P1-02 Inventory every handler and extractor, including middleware ordering and rejection behavior. Depends: P1-01. Evidence: `middleware-contract.md`. Verify: auth, workspace, member, rate-limit, i18n, request-ID, and tracing stages are ordered.
- [ ] P1-03 Inventory services and repository traits; record allowed dependencies and transaction ownership. Depends: P1-01. Evidence: `layer-map.md`. Verify: no handler-to-repository or service-to-Diesel shortcut is omitted.
- [ ] P1-04 Inventory models, DTOs, Diesel entities, enum values, serde names, defaults, and private/public field boundaries. Depends: P1-03. Evidence: `type-mapping.md`. Verify: every API-visible type has a Rust source and Go target.
- [ ] P1-05 Inventory PostgreSQL tables, views, functions, indexes, migrations, soft-delete filters, workspace filters, and transaction boundaries. Depends: P1-03. Evidence: `database-contract.md`. Verify: every repository query family is assigned a Go implementation strategy.
- [ ] P1-06 Inventory Redis keys, value formats, TTLs, namespaces, locks, cache invalidation, rate-limit buckets, blocklists, and pub/sub. Depends: P1-03. Evidence: `redis-contract.md`. Verify: no key is regenerated from guessed naming.
- [ ] P1-07 Inventory background jobs, queues, outbox records, scheduled tasks, retry policies, dead-letter behavior, and shutdown behavior. Depends: P1-03. Evidence: `job-contract.md`. Verify: each producer has a consumer and payload schema.
- [ ] P1-08 Inventory external integrations: S3, SMTP, ClamAV, HTTP services, gRPC, WebSocket, and filesystem paths. Depends: P1-03. Evidence: `dependency-inventory.md`. Verify: timeout, retry, auth, and failure mapping are recorded.
- [ ] P1-09 Inventory configuration and secrets: env names, required/optional status, defaults, validation, rotation, and startup fail-closed behavior. Depends: P1-08. Evidence: `configuration-contract.md`. Verify: no secret value is committed or copied into logs.
- [ ] P1-10 Inventory deployment, health checks, readiness, resource limits, ports, certificates, service identities, and compose/test profiles. Depends: P1-08. Evidence: `deployment-contract.md`. Verify: Rust and Go can coexist without port or service-name collision.
- [ ] P1-11 Inventory data classification, PII fields, masking rules, retention, audit events, export events, and erasure behavior. Depends: P1-04, P1-07. Evidence: `privacy-contract.md`. Verify: T1/T2 fields have explicit log/API restrictions.

## Phase 2 — Contract freeze and baseline

- [ ] P2-01 Export or otherwise snapshot current OpenAPI and compare generated frontend API types. Depends: P1-01, P1-04. Evidence: `openapi-baseline.*`. Verify: generated types are unchanged before implementation.
- [ ] P2-02 Complete endpoint matrix fields: method/path, auth, permission, inputs, status, headers, body, errors, pagination, rate limit, timeout, side effects, transaction, and audit. Depends: P1-01, P1-02. Verify: no blank REQUIRED column for an in-scope endpoint.
- [ ] P2-03 Define canonical error contract from `AppError`: HTTP status, error code, message localization, field errors, log classification, and retryability. Depends: P1-02. Evidence: `error-contract.md`. Verify: representative errors have golden JSON.
- [ ] P2-04 Define type compatibility rules: UUID, nullable/missing/empty, enums, JSON, decimal, timestamp, timezone, DST, binary, arrays, and pagination. Depends: P1-04. Verify: fixtures cover each ambiguous representation.
- [ ] P2-05 Define authentication and authorization matrix: JWT claims, expiry, refresh rotation, revocation, workspace context, resource:action permissions, default deny, service identity, and audit. Depends: P1-02, P1-11. Evidence: `auth-permission-matrix.csv`. Verify: every protected endpoint has a permission row.
- [ ] P2-06 Define observability contract: request ID, trace propagation, structured JSON fields, log levels, PII masking, metrics, health response, and audit events. Depends: P1-10, P1-11. Evidence: `observability-contract.md`. Verify: required fields and redaction rules are testable.
- [ ] P2-07 Define `attendance-compute` integration contract: protobuf version, gRPC metadata, mTLS, scoped JWT, 2-second deadline, 1 retry only for `UNAVAILABLE`, digest/rules validation, response validation, and no DB transaction held over RPC. Depends: P1-08. Evidence: `attendance-compute-contract.md`. Verify: contract matches `.ai/ADRs/0002-attendance-compute-grpc-cutover.md` and proto.
- [ ] P2-08 Define permitted nondeterminism for comparisons: request IDs, trace IDs, generated IDs, timestamps, presigned URL expiry, ordering where contract permits it. Depends: P2-02, P2-04. Evidence: `comparison-normalization.md`. Verify: every ignored field has an explicit reason.
- [ ] P2-09 Freeze baseline runtime behavior and test result. Depends: P2-01 through P2-08. Evidence: baseline logs and artifact hashes. Verify: run `./leedevkit test api`; record result without modifying source.

## Phase 3 — Behavior characterization and parity oracle

- [ ] P3-01 Build fixture format for HTTP request, identity, workspace, database seed, Redis seed, external stubs, expected response, expected side effects, and normalized fields. Depends: P2-08. Evidence: `fixtures/README.md`. Verify: one fixture executes deterministically.
- [ ] P3-02 Add/gather auth and authorization fixtures: missing token, invalid token, expired token, revoked token, wrong workspace, missing permission, service token, and rate limit. Depends: P2-05. Verify: expected 401/403/429 and no unauthorized side effect.
- [ ] P3-03 Add/gather workspace/settings/feature-flag fixtures. Depends: P3-01. Verify: tenant isolation, validation, optimistic/conflict behavior, and audit output.
- [ ] P3-04 Add/gather employee/employment/assignment/custom-field fixtures. Depends: P3-01. Verify: uniqueness, soft delete, workspace scope, bulk operations, and PII response restrictions.
- [ ] P3-05 Add/gather attendance/time-tracking fixtures. Depends: P3-01, P2-07. Verify: duplicate punches, reprocess, overnight, idempotency, gRPC failure, and no-write-on-compute-failure.
- [ ] P3-06 Add/gather shift/roster fixtures. Depends: P3-01. Verify: timezone, DST gap/ambiguity, overnight offsets, break windows, and stable pagination.
- [ ] P3-07 Add/gather leave/ESS/approval/notification fixtures. Depends: P3-01. Verify: state transitions, authorization, balance updates, notification/outbox behavior, and transaction rollback.
- [ ] P3-08 Add/gather payroll/report/dashboard fixtures. Depends: P3-01. Verify: decimal precision, rounding, aggregation, exports, audit, and no N+1 behavior.
- [ ] P3-09 Add/gather files/documents/e-signature fixtures. Depends: P3-01. Verify: upload limits, content validation, ClamAV/S3 failures, signed URL behavior, and PII handling.
- [ ] P3-10 Add/gather lifecycle, condition types, system, notifications, and remaining route fixtures. Depends: P3-01. Verify: each remaining endpoint has success and failure examples.
- [ ] P3-11 Add database-state and Redis-state comparators. Depends: P3-02 through P3-10. Verify: comparator catches changed rows, TTLs, key values, audit entries, outbox events, and rollback.
- [ ] P3-12 Add Rust-vs-Go differential harness interface without Go implementation dependency. Depends: P3-11. Verify: same fixture can target either base URL and produce a normalized comparison report.

## Phase 4 — Go repository and build foundation

- [ ] P4-01 Create Go module/package layout matching project layering, with no business behavior yet. Depends: P0-02, P2-03. Verify: package dependency direction is enforceable.
- [ ] P4-02 Implement configuration loading and fail-closed validation for all required settings. Depends: P1-09. Verify: missing/malformed settings fail startup without leaking secrets.
- [ ] P4-03 Implement structured logger, request ID, trace propagation, and PII redaction. Depends: P2-06. Verify: logs contain required fields and never raw T1/T2 data.
- [ ] P4-04 Implement application state, lifecycle, graceful shutdown, signal handling, and bounded goroutine cleanup. Depends: P4-01, P4-02. Verify: shutdown drains requests/jobs within configured deadline.
- [ ] P4-05 Implement `/health` liveness and `/health/ready` dependency checks with exact response contract. Depends: P1-10, P2-06. Verify: liveness works with DB down; readiness fails when required dependency is unavailable.
- [ ] P4-06 Implement PostgreSQL pools for system, tenant transaction, and tenant session use-cases. Depends: P1-05, P0-02. Verify: pool limits, timeouts, transaction release, and context cancellation.
- [ ] P4-07 Implement Redis client, namespaces, timeout, serialization, TTL, and failure policy. Depends: P1-06. Verify: key/TTL compatibility fixtures pass.
- [ ] P4-08 Implement typed application errors and response mapping. Depends: P2-03. Verify: golden error responses and internal error redaction.
- [ ] P4-09 Implement HTTP middleware order: request ID, recovery, access logging, rate limit, auth, workspace/member context, tracing, and route dispatch. Depends: P2-05, P2-06. Verify: unauthorized requests stop before service logic.
- [ ] P4-10 Implement JWT, refresh, blocklist, service-token, workspace, and permission primitives. Depends: P2-05, P4-07. Verify: all P3-02 auth fixtures pass against isolated Go components.
- [ ] P4-11 Implement OpenAPI generation/validation integration without changing existing schema. Depends: P2-01. Verify: schema diff is empty or approved.
- [ ] P4-12 Implement test harness, fixtures, database reset, Redis reset, external stubs, and differential runner. Depends: P3-12, P4-06, P4-07. Verify: one fixture runs end-to-end against Go foundation.

## Phase 5 — Shared adapters and cross-cutting behavior

- [ ] P5-01 Implement transaction helper with explicit commit/rollback and context cancellation. Depends: P4-06. Verify: injected failure leaves no partial mutation.
- [ ] P5-02 Implement tenant-scoped query helpers or review checks that prevent missing `workspace_id` filters. Depends: P1-05, P4-06. Verify: cross-tenant fixture fails closed.
- [ ] P5-03 Implement pagination, filtering, sorting, cursor/offset, and stable ordering exactly as current API. Depends: P2-02, P2-04. Verify: boundary, empty, duplicate, and last-page fixtures.
- [ ] P5-04 Implement validation and decoding rules, including body size, UUID, enum, decimal, date, timezone, multipart, and unknown fields. Depends: P2-04. Verify: invalid input status/error parity.
- [ ] P5-05 Implement audit event writer and PII-safe application event logging. Depends: P1-11, P2-06. Verify: required events appear once with no restricted data.
- [ ] P5-06 Implement S3, SMTP, ClamAV, HTTP, and filesystem adapters with timeouts and failure mappings. Depends: P1-08. Verify: stubbed success, timeout, auth failure, retry, and cleanup behavior.
- [ ] P5-07 Implement queue/outbox and background worker adapters. Depends: P1-07. Verify: payload parity, retry/visibility timeout, idempotency, shutdown, and audit behavior.
- [ ] P5-08 Implement Go `attendance-compute` client and mapping layer. Depends: P2-07, P5-01. Verify: mTLS/JWT, metadata, deadline, one retry, digest/rules validation, response validation, and no RPC while holding DB transaction.
- [ ] P5-09 Implement shared export/import and file response behavior. Depends: P5-04, P5-06. Verify: row counts, audit records, content headers, and masking.

## Phase 6 — Domain migration slices

Each slice requires: handler, service, repository, DTOs, permissions, audit events, metrics, OpenAPI check, unit tests, integration fixtures, differential comparison, performance/N+1 review, and rollback note.

- [ ] P6-01 Auth/session/token domain. Depends: P4-10, P5-05. Verify: all auth fixtures, token rotation/revocation, login rate limit, session invalidation, and audit.
- [ ] P6-02 Workspaces/settings/feature flags/permissions domain. Depends: P6-01, P5-02. Verify: workspace isolation, RBAC, settings transaction, flag behavior, and P3-03.
- [ ] P6-03 Employees/employments/assignments/custom fields domain. Depends: P6-02, P5-02. Verify: PII access rules, soft delete, uniqueness, bulk operations, and P3-04.
- [ ] P6-04 Attendance/time tracking domain orchestration. Depends: P6-03, P5-08. Verify: Rust-equivalent prefetch, gRPC request, result validation, transaction writes, reprocess, and P3-05.
- [ ] P6-05 Shifts/roster/pattern/compliance domain. Depends: P6-03, P6-04. Verify: overnight/DST/timezone/break semantics, cache behavior, and P3-06.
- [ ] P6-06 Leave/ESS/approvals domain. Depends: P6-03, P5-07. Verify: transition rules, balances, permissions, notifications, rollback, and P3-07.
- [ ] P6-07 Notifications/domain delivery domain. Depends: P5-06, P5-07. Verify: templates, localization, deduplication, retry, delivery failure, and P3-07.
- [ ] P6-08 Payroll/work buckets/domain calculations. Depends: P6-04, P6-05, P6-06. Verify: decimal/rounding invariants, persistence, authorization, export, and P3-08.
- [ ] P6-09 Reports/dashboard/analytics domain. Depends: P6-03 through P6-08. Verify: aggregation parity, pagination, query count, export audit, and P3-08.
- [ ] P6-10 Files/documents/e-signature domain. Depends: P5-06, P5-09. Verify: upload/download, malware rejection, S3 failure, signed URL, authorization, and P3-09.
- [ ] P6-11 Lifecycle/condition types/system/admin domain. Depends: P6-02, P5-05. Verify: remaining route fixtures and P3-10.
- [ ] P6-12 Domain completion review. Depends: P6-01 through P6-11. Verify: endpoint matrix has no `unmapped` or `implemented` rows; all are `verified`.

## Phase 7 — Differential, security, and performance proof

- [ ] P7-01 Run all fixture requests against Rust and Go with response normalization. Depends: P6-12. Verify: zero unexplained mismatches.
- [ ] P7-02 Compare database mutations and rollback state after every mutating fixture. Depends: P7-01. Verify: row, column, soft-delete, audit, and outbox parity.
- [ ] P7-03 Compare Redis keys, values, TTLs, locks, invalidation, and rate-limit state. Depends: P7-01. Verify: zero unexplained cache/session differences.
- [ ] P7-04 Run authorization and tenant-isolation security suite. Depends: P7-01. Verify: unauthorized access always denied before business logic and no cross-tenant data leak.
- [ ] P7-05 Run PII/log/export audit review. Depends: P7-01. Verify: no T1 logs/responses and required T2 access/export audit exists.
- [ ] P7-06 Run concurrency/idempotency/retry/deadline/failure-injection suite. Depends: P7-02, P7-03. Verify: no duplicate mutation, deadlock, leaked connection, or partial transaction.
- [ ] P7-07 Run N+1 and query-count review for list/report/dashboard paths. Depends: P6-12. Verify: query count remains bounded and bulk paths use joins/IN/batch operations.
- [ ] P7-08 Run load/latency/resource comparison against Rust baseline. Depends: P7-01, P7-07. Verify: agreed p95/p99, memory, CPU, pool, and error budgets are met or approved.
- [ ] P7-09 Run OpenAPI/frontend compatibility checks. Depends: P7-01. Verify: generated types and webdashboard behavior require no unapproved changes.

## Phase 8 — Deployment, shadow, canary, and rollback

- [ ] P8-01 Add Go image/build/test target and compose/deployment service without replacing Rust. Depends: P4-12. Verify: reproducible container build and isolated health check.
- [ ] P8-02 Add routing switch and per-domain/workspace feature flags. Depends: P6-12. Verify: default route remains Rust; flag-off rollback is immediate.
- [ ] P8-03 Deploy staging with mTLS/JWT, certificates, service identity, logs, metrics, alerts, and readiness. Depends: P8-01. Verify: startup fail-closed and gRPC connectivity.
- [ ] P8-04 Run non-production shadow comparisons using identical immutable snapshots. Depends: P8-02, P8-03. Verify: zero mismatch across approved fixture/canary set.
- [ ] P8-05 Run staging canary for read-only endpoints. Depends: P8-04. Verify: status/body/latency/log/metric parity and no unexpected writes.
- [ ] P8-06 Run staging canary for mutating endpoints with isolated tenants. Depends: P8-05. Verify: DB/Redis/audit/outbox parity and rollback.
- [ ] P8-07 Exercise failure drills: Go down, gRPC unavailable, DB unavailable, Redis unavailable, timeout, invalid certificate/JWT, queue failure, and partial dependency failure. Depends: P8-06. Verify: fail-closed/no-write behavior and alert delivery.
- [ ] P8-08 Exercise deployment rollback to last approved Rust/Go images. Depends: P8-07. Verify: no schema rollback required and traffic returns to Rust.
- [ ] P8-09 Roll out domains progressively with explicit go/no-go evidence. Depends: P8-08. Verify: thresholds for 5xx, p99, auth failures, pool utilization, queue failures, and audit lag remain satisfied.

## Phase 9 — Final verification and retirement

- [ ] P9-01 Run `./leedevkit test api` after all domain code and adapter changes. Depends: P7-09, P8-09. Verify: pass and archive log.
- [ ] P9-02 Run Go service tests through project wrapper, including race/static checks defined by repository. Depends: P8-09. Verify: pass and archive log.
- [ ] P9-03 Run `./leedevkit test all`. Depends: P9-01, P9-02. Verify: full backend/frontend/integration/E2E suite passes.
- [ ] P9-04 Run deployment validation and health/readiness smoke checks against the cutover topology. Depends: P9-03. Verify: all services healthy and request-ID/tracing visible.
- [ ] P9-05 Reconcile parity matrix, endpoint matrix, dependency inventory, ADRs, runbooks, dashboards, and rollback instructions. Depends: P9-04. Verify: no open `unmapped`, `implemented`, or unexplained mismatch rows.
- [ ] P9-06 Obtain explicit cutover completion approval. Depends: P9-05. Verify: approval references evidence artifacts and rollback window.
- [ ] P9-07 Retire Rust API route/runtime only after approved rollback window. Depends: P9-06. Verify: shared migrations/contracts remain; no remaining traffic or dependency points to retired Rust API.
- [ ] P9-08 Remove temporary flags, shadow code, comparison scaffolding, and migration-only adapters according to ADR retention rules. Depends: P9-07. Verify: no dead code, no hidden fallback, no stale migration TODO.
- [ ] P9-09 Re-run full suite and smoke checks after retirement. Depends: P9-08. Verify: same acceptance criteria still pass.
- [ ] P9-10 Mark ledger complete and archive final evidence index. Depends: P9-09. Verify: every task is `verified`, `cutover`, or explicitly `retired` with evidence.

## Required evidence index

- `docs/migration/rust-go/task-status.csv`
- `docs/migration/rust-go/endpoint-matrix.csv`
- `docs/migration/rust-go/dependency-inventory.md`
- `docs/migration/rust-go/type-mapping.md`
- `docs/migration/rust-go/error-contract.md`
- `docs/migration/rust-go/database-contract.md`
- `docs/migration/rust-go/redis-contract.md`
- `docs/migration/rust-go/job-contract.md`
- `docs/migration/rust-go/auth-permission-matrix.csv`
- `docs/migration/rust-go/observability-contract.md`
- `docs/migration/rust-go/attendance-compute-contract.md`
- `docs/migration/rust-go/fixtures/`
- `docs/migration/rust-go/parity-reports/`
- `docs/migration/rust-go/rollback-drills/`
- `docs/migration/rust-go/final-evidence.md`

## Stop conditions

Stop and request a decision when any of these occur:

- Rust behavior is undocumented and multiple interpretations change the public contract.
- Go cannot preserve a status/error/body/header contract without changing consumers.
- Existing data contains ambiguous timezone, decimal, enum, or null semantics.
- A dependency requires schema changes, distributed transactions, or a new trust boundary.
- Differential mismatch remains unexplained after two focused investigations.
- Security, tenant isolation, PII, transaction integrity, or rollback evidence is incomplete.
