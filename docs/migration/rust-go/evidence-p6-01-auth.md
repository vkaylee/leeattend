# P6-01 Auth/session/token slice evidence

Status: in progress; Go implementation and canonical auth fixture contract tests verified; Rust-vs-Go live differential parity remains pending.

## Implemented

- `internal/service/auth.go`: typed login, Argon2id password verification, 15-minute JWT issuance, 7-day refresh session rotation, reuse-triggered session revocation.
- `internal/service/auth.go`: registration with Rust-compatible email/password/full-name validation, duplicate-email rejection, Argon2id hashing, first-user `platform_admin` bootstrap, and session issuance.
- `internal/service/auth.go`: change-password, forgot-password, and single-use reset-password flows with session revocation and user-wide Redis cutoff writes.
- `internal/repository/auth.go`: pgx-backed user/session reads, session revocation, transactional first-user registration with `pg_advisory_xact_lock(1337)`, and password-hash updates.
- `internal/infra/redis/login_limiter.go`: Redis login attempt/lockout keys and exponential lockout policy; Redis failures remain fail-open for login availability.
- `internal/infra/redis/revocation.go`: JWT signature and user cutoff reads plus reset-token create/consume and revocation writes using existing Redis key contracts.
- `internal/infra/http/rate_limit.go`: Redis atomic INCR/EXPIRE rate limiting with bounded in-memory fallback, method tiers, health bypass, identity keys, and exact 429 response contract.
- `internal/infra/redis/rate_limit.go`: Redis Lua counter adapter preserving first-hit TTL semantics.
- `internal/api/auth.go`: register, forgot-password, reset-password, and change-password handlers with anti-enumeration response and Rust-compatible error keys.
- `internal/api/auth_fixture_test.go`, `internal/api/auth_test.go`, `internal/service/auth_test.go`: executable auth contract tests covering login, registration validation, reset-token consumption, password change, logout, revocation, and rate-limit behavior.
- `internal/auth/jwt.go`: explicit timestamp token constructor used to exercise expired-token behavior deterministically.
- `main.go`: rate limiter, auth routes, public-prefix exemptions, and Redis token store wired into the auth service.
## Verification

- `./scripts/_go-gates.sh`: PASS after auth self-service changes. Includes `gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, and `go test -race` across Go packages.
- `./leedevkit test api-go`: ALL GATES PASSED.
- `./scripts/_python.sh scripts/validate_migration_fixtures.py`: PASS (`59 fixtures validated; 59 unique IDs; all sections present`).
- Go canonical auth tests PASS for AUTH-001/002/003/004/006/008 plus register, forgot-password, reset-password, and change-password validation/consumption cases.
- Live Go smoke PASS: `/api/v1/health` returned `200`; Redis-seeded AUTH-008 returned `429`, `Retry-After: 10`, exact JSON body, and Redis count advanced from `2` to `3`; Redis signature blocklist returned `401`.
- Go API integration containers initialized PostgreSQL migrations and Redis successfully.
- Endpoint metrics: contract routes 403; Go route registrations 135; covered 115; missing 288.
- `./leedevkit test api`: BLOCKED by pre-existing Rust/agent-main workspace lint/build failures under current toolchain (`Atomic::fetch_update` and widespread `clippy::double_must_use`/`unused_async` errors).

## Remaining P6-01 proof

- Start Rust reference server in a tracked integration profile and run identical auth fixtures against both servers.
- Add live PostgreSQL session rotation/reuse/expiry assertions and login lockout threshold evidence.
- Add audit-event sink wiring and evidence for login/logout/security events.

