# Rust-to-Go API Server Migration

Status: Awaiting approval
Execution ledger: `rust-go-migration-task-ledger.md` contains restartable task IDs, dependencies, evidence, and verification for context compaction recovery.

Approval required before implementation. Ledger task `P0-01` is the first execution task.
Current checkout prerequisite: required migration sources are the tracked `apiserver/`, existing OpenAPI contract, deployment definitions, and integration-test fixtures. Execution ledger starts with a source/contract recovery checkpoint; AI MUST NOT infer missing behavior from prose alone.

## Goal
Migrate `apiserver/` from Rust/Axum to Go without changing consumer-visible behavior, database semantics, security boundaries, or operational contracts.

## Scope and defaults
- In scope: `apiserver/`, its HTTP API, PostgreSQL/Redis/S3/SMTP/ClamAV integrations, background jobs owned by the API, OpenAPI contract, and integration tests.
- No separate compute service is in scope. Attendance pairing, matching, timezone, and payroll calculations move into typed in-process Go domain packages with no database or network access.
- Out of scope initially: `agent-main/`, `webdashboard/`, and database redesign.
- Default cutover: Go runs beside Rust, then domain-by-domain route cutover with rollback to Rust.
- Compatibility target: same HTTP method/path/status/headers/body/error codes, auth/RBAC behavior, pagination, side effects, audit records, and transaction boundaries.

## Proposed technology contract — awaiting approval

| Concern | Choice | Verification gate |
|---|---|---|
| Language/runtime | Go 1.24.x; `go.mod` and container toolchain pinned to one patch line | `go version`, reproducible container build, `go mod verify` |
| HTTP routing | Go standard `net/http.ServeMux` (Go 1.22+ method + wildcard path patterns) + strict generated adapter | `oapi-codegen` strict server interface compile-checks 100% of routes |
| API types | Existing OpenAPI frozen as canonical; `oapi-codegen` v2 generates models, request/response types, and strict server interface | CI regeneration check: zero `git diff` against checked-in generated code |
| PostgreSQL | `pgx/v5` (`pgxpool`) + `sqlc` compile-time generated queries; no ORM | `sqlc generate` zero diff; tenant isolation compile check; integration tests |
| Redis | `go-redis/v9` behind narrow internal adapter trait | Golden serialization tests, TTL compliance, live Redis integration tests |
| Domain calculation | In-process pure Go packages (`internal/domain/pairing`, `internal/domain/matcher`, `internal/domain/payroll`) | Table-driven parity fixtures matching Rust behavior; property tests; zero DB/network in domain |
| Validation/types | Strict JSON decoding; `google/uuid`; `time.Time` with IANA tz; `shopspring/decimal` for money/payroll (zero `float64`) | Fuzzing on decoders; boundary tests for null/missing/empty/tz/DST |
| Errors | Internal typed `AppError` enum; single HTTP error mapper; wrapped causes never leak to client | Golden error response tests; redaction tests |
| Auth/security | `golang-jwt/jwt/v5`, `golang.org/x/crypto/argon2`; Redis blocklist; handler-layer permission checks | Auth matrix integration tests (401/403/429); tenant isolation check; PII masking linter |
| Observability | `log/slog` structured JSON + OpenTelemetry Go (W3C trace context, spans, metrics); request ID middleware | Log field validation tests; trace propagation integration tests; `/health` and `/health/ready` |
| External systems | Go interfaces in service layer; adapters for S3, SMTP, ClamAV; pure domain never imports adapters | Interface mock tests + live container integration tests |
| Testing | `testing`, `httptest`, compose-managed Postgres/Redis; test fixtures directly from Rust suite | `go test ./...`, `go test -race ./...`, `go vet ./...`; coverage thresholds enforced |
| Static gates | `gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify` | All gates must pass in container; CI fails on any warning |
| Packaging | Multi-stage Dockerfile, `CGO_ENABLED=0` static binary, distroless non-root runtime | Container vulnerability scan, minimal image size, healthcheck verification |

Rejected defaults (with explicit rationale):
- **GORM / Ent / other ORMs:** Rejected. Hidden N+1 queries, implicit transactions, unpredictable SQL generation. `sqlc` generates type-safe Go from raw SQL verified against Postgres schema.
- **Chi / Gin / Fiber / Echo:** Rejected. Go 1.22+ standard library `net/http.ServeMux` supports method matching and wildcard path patterns natively. Third-party routers add external dependency risk without architectural benefit when using generated `oapi-codegen` adapters.
- **`float64` for money / hours:** Rejected. Precision loss in payroll/overtime calculations is unacceptable in enterprise HR. `shopspring/decimal` provides arbitrary-precision fixed-point arithmetic matching Rust's `rust_decimal` / `bigdecimal`.
- **`map[string]any` in handlers / DTOs:** Rejected. Circumvents compiler type checking. All request/response bodies must use structs generated by `oapi-codegen`.
- **Separate compute microservice:** Rejected per user instruction. In-process pure domain packages reduce operational complexity, eliminate network serialization latency, avoid distributed failure modes, and keep transaction boundaries simple.

Required devkit target: `api-go` running `gofmt`, `go vet`, `staticcheck`, `govulncheck`, unit tests with `-race`, and integration tests.

## Enterprise stack rationale

The requirements are: compile-time type safety, deterministic behavior parity, verifiable build/test gates, and low operational complexity. Each choice is driven by that, not by popularity.

| Decision | Why it wins for this project | Cost accepted |
|---|---|---|
| Go standard `net/http.ServeMux` | Go 1.22+ supports method + wildcard patterns. Zero third-party routing dependency. Generated `oapi-codegen` adapter enforces every route signature at compile time. | No batteries-included middleware ecosystem; middleware is written explicitly. |
| `oapi-codegen` strict server interface | The existing OpenAPI becomes executable specification. A handler that omits a route, returns wrong type, or drifts from schema **fails compilation**, not review. | Requires keeping OpenAPI authoritative and regenerating. |
| `pgx/v5` + `sqlc` | `sqlc` compiles SQL against the real schema and emits typed Go. Prevents untyped row scanning, prevents silent column drift, and keeps tenant `workspace_id` filters visible in SQL review. | No runtime ORM convenience; every query is explicit. |
| `shopspring/decimal` | Payroll, overtime, and leave balances must not use binary floating point. Matches Rust `rust_decimal`/`bigdecimal` precision guarantees. | Slightly more verbose arithmetic; no native literal syntax. |
| Pure in-process domain packages | Pairing/matcher logic has no DB or network dependency by design. Keeping it in-process removes serialization, network failure modes, mTLS/JWT config, cross-service versioning, and distributed debugging. | Larger single binary; needs strict package boundaries to stay honest. |
| `log/slog` + OpenTelemetry Go | Standard-library structured logging; vendor-neutral traces/metrics that match existing observability contract. | Requires OTel SDK wiring and exporter configuration. |
| `CGO_ENABLED=0` static binary | Reproducible builds, distroless non-root runtime, smaller attack surface, no glibc coupling. | `-race` tests need the builder image, not the release image. |

Verification philosophy: **every claim in this plan must be provable by a command that can fail.** Compile gates prove type safety; `sqlc`/`oapi-codegen` diff checks prove no hand-edited generated code; seeded-violation demonstrations prove gates are not false-green; differential fixtures prove behavior parity against the Rust reference.


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
