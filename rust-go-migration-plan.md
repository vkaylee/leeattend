# Rust-to-Go API Server Migration

Status: Awaiting approval
Execution ledger: `rust-go-migration-task-ledger.md` contains restartable task IDs, dependencies, evidence, and verification for context compaction recovery.

Approval required before implementation. Ledger task `P0-01` is the first execution task.
Current checkout prerequisite: the previously untracked `attendance-compute/`, `proto/`, `buf.yaml`, `buf.gen.yaml`, and ADR `0002-attendance-compute-grpc-cutover.md` are absent after untracked-file cleanup. The execution ledger therefore starts with a source/contract recovery checkpoint; AI MUST NOT infer or recreate that service contract from prose alone.

## Goal
Migrate `apiserver/` from Rust/Axum to Go without changing consumer-visible behavior, database semantics, security boundaries, or operational contracts.

## Scope and defaults
- In scope: `apiserver/`, its HTTP API, PostgreSQL/Redis/S3/SMTP/ClamAV integrations, background jobs owned by the API, OpenAPI contract, and integration tests.
- `attendance-compute/` is an existing Go gRPC dependency, not a migration target. Preserve its protobuf contract, mTLS/JWT, `CalculateDay` behavior, timeout, digest, and error semantics while replacing the Rust client/orchestration in `apiserver/`.
- Out of scope initially: `agent-main/`, `webdashboard/`, and database redesign.
- Default cutover: Go runs beside Rust, then domain-by-domain route cutover with rollback to Rust.
- Compatibility target: same HTTP method/path/status/headers/body/error codes, auth/RBAC behavior, pagination, side effects, audit records, and transaction boundaries.

## Proposed technology contract — awaiting approval

| Concern | Choice | Verification gate |
|---|---|---|
| Language/runtime | Go 1.24.x; `go.mod` and container toolchain pinned to one patch line | `go version`, reproducible container build, `go mod verify` |
| HTTP | Standard `net/http` plus `chi/v5` only for routing/middleware composition | `oapi-codegen` strict server interface compile-checks every route implementation |
| API types | Frozen existing OpenAPI as canonical; `oapi-codegen` v2 generates models, request/response types, and strict server interfaces | regenerate in CI and require zero `git diff`; OpenAPI compatibility diff must be empty |
| PostgreSQL | `pgx/v5` + `pgxpool`; `sqlc` generated query layer; no ORM | `sqlc generate` zero diff, `go vet`, integration tests against PostgreSQL, explicit transaction tests |
| Redis | `go-redis/v9` behind a narrow internal adapter | key/TTL/serialization golden tests and Redis integration tests |
| Protobuf/gRPC | Canonical existing `attendance-compute` protobuf; Buf plus generated Go gRPC client | `buf lint`, `buf breaking`, generated-code zero diff, mTLS/JWT/deadline/retry contract tests |
| Validation/types | Generated OpenAPI types, strict JSON decoding, explicit constructors/validators; `uuid` type; `time.Time`/IANA locations; `shopspring/decimal` for exact payroll values; no `float64` for money | boundary tests for missing/null/empty, UUID, enum, decimal, UTC/DST, pagination |
| Errors | Internal typed error code/status/retryability; one HTTP response mapper; wrapped causes never exposed | golden error JSON/status/header tests and internal-error redaction tests |
| Auth/security | `golang-jwt/jwt/v5`, `golang.org/x/crypto/argon2`; Redis revocation; handler-layer permission checks | auth matrix, 401/403/429 tests, rotation/revocation, tenant-isolation, PII log scan |
| Observability | `log/slog` JSON plus OpenTelemetry Go for traces/metrics; request-ID middleware | required fields, propagation, masking, `/health`, `/health/ready`, and alert smoke tests |
| External systems | Interfaces in service layer; adapters for S3, SMTP, ClamAV, HTTP, queues; no infrastructure calls from domain logic | deterministic fakes plus timeout/retry/failure-injection integration tests |
| Testing | `testing`, `httptest`, compose-managed PostgreSQL/Redis; no production mocks; generated fakes only where interface size requires it | `go test ./...`, `go test -race ./...`, `go vet ./...`, coverage, differential Rust/Go fixtures |
| Static/security gates | `gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`; pinned tool versions in build image | every Go CI run fails on any gate; generated-file diff is mandatory |
| Packaging | multi-stage Go container, `CGO_ENABLED=0` release binary, non-root distroless runtime; race tests use builder image | image smoke test, liveness/readiness, SBOM/vulnerability scan, rollback drill |

Rejected defaults: GORM (implicit query/transaction behavior), `float64` payroll math, runtime `map[string]any` API DTOs, untyped `error` responses, direct handler-to-DB calls, and a second API contract. These make parity and review harder.

Required devkit targets: `api-go` for format/vet/staticcheck/unit/integration/coverage and `all` only after Go service is independently green. The existing `api` target remains Rust/agent until cutover.

## Constraints and interfaces
- Canonical API: existing OpenAPI plus generated frontend types; no contract drift without explicit versioning.
- Layering: middleware → handler → service → repository → storage. Handlers do authorization before business logic.
- Tenant isolation: every tenant query scopes `workspace_id`; no query-in-loop; multi-table mutations use one transaction.
- Security: preserve JWT claims/rotation/revocation, rate limits, PII masking, audit logging, and service-token rules.
- Types: preserve UUID, nullable-vs-missing, decimal precision, enum, JSON, UTC/timezone/DST, and pagination semantics.
- Tooling: use project wrappers and `./leedevkit test`; never call raw `cargo`, `npm`, `diesel`, or direct test commands.
- Rule IDs: `project-structure.md`, `coding-standards.md`, `database-rules.md`, `access-control.md`, `data-governance.md`, `observability-rules.md`, `testing-standards.md`.

## Tasks
- [ ] 1. Freeze scope and contracts: inventory all routes, middleware, permissions, error responses, env vars, external dependencies, Redis keys, queues, migrations, and health endpoints. Deliver `docs/migration/rust-go/endpoint-matrix.csv` and `dependency-inventory.md`. Verify every route in `apiserver/src/routes/` has an owner and parity row.
- [ ] 2. Capture Rust behavior: turn existing integration coverage into reusable fixtures for auth, attendance, payroll, shifts/timezones, employees, leave, permissions, pagination, files, notifications, and dashboard. Record permitted nondeterministic fields. Verify baseline with `./leedevkit test api`.
- [ ] 3. Define Go contracts: write type mapping, error contract, database/transaction contract, auth-permission matrix, observability contract, and Go package boundaries. Verify OpenAPI schema and generated frontend types remain unchanged.
- [ ] 4. Build Go foundation: application state, config validation, logging/tracing/request ID, health/readiness, DB/Redis clients, typed errors, middleware, auth/RBAC, and graceful shutdown. Verify startup, unauthorized access, readiness failure, and request-ID propagation.
- [ ] 5. Migrate one low-risk vertical slice end-to-end using existing schema and dependencies. Prefer workspace/settings or another bounded domain. Verify unit, integration, unauthorized, tenant-isolation, transaction-rollback, and error-contract cases.
- [ ] 6. Migrate remaining domains in dependency order: auth/workspaces → employees/organization → attendance/time tracking/shifts → leave/ESS/notifications → payroll/reports/dashboard → files/documents/system. After each domain, run differential Rust-vs-Go tests and update the parity matrix.
- [ ] 7. Add shadow/canary cutover: route selected traffic to Go, compare responses and side effects, monitor 5xx, p99 latency, DB pool, auth failures, queue failures, and audit logs. Verify rollback to Rust without schema rollback.
- [ ] 8. Remove Rust API only after all rows reach `verified`/`cutover`, operational dashboards and runbooks are updated, and rollback has been exercised. Do not delete shared migrations or contracts.

## Done when
- 100% in-scope routes and side effects have verified parity rows.
- OpenAPI, frontend behavior, auth/RBAC, PII policy, DB/Redis semantics, and observability contracts show no unapproved drift.
- Differential, integration, E2E, security, and full-suite checks pass.
- Canary and rollback evidence exists.
- Run `./leedevkit test api`, then `./leedevkit test all` after final cutover.

## Approval gate
No Go implementation starts until this plan and the migration scope are approved. Approval may revise scope, cutover strategy, or compatibility target before Task 1 begins.
