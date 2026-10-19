# P6-01 Auth/session/token slice evidence

Status: in progress; implementation slice verified, full fixture parity pending.

## Implemented

- `internal/service/auth.go`: typed login, Argon2id password verification, 15-minute JWT issuance, 7-day refresh session rotation, reuse-triggered session revocation.
- `internal/repository/auth.go`: pgx-backed user/session reads and session revocation using existing `users` and `sessions` schema.
- `internal/infra/redis/login_limiter.go`: Redis login attempt/lockout keys and exponential lockout policy; Redis failures remain fail-open for login availability.
- `internal/infra/redis/revocation.go`: JWT signature and user cutoff reads using existing Redis key contracts.
- `internal/infra/http/middleware.go`: bearer and `access_token` cookie extraction, JWT validation, optional Redis revocation checks, workspace context propagation.
- `internal/api/auth.go`: login, refresh, logout, and current-user handlers; cookie issuance/clearing and typed error envelopes.
- `main.go`: system pool, Redis client, auth routes, and auth middleware wiring.

## Verification

- `./scripts/_go-gates.sh`: PASS. Includes `gofmt`, `go vet`, `staticcheck`, `govulncheck`, `go mod verify`, and `go test -race` across Go packages.
- `./scripts/_go-gates.sh --package ./internal/api`: PASS.
- `./scripts/_python.sh scripts/validate_migration_fixtures.py`: PASS (`59 fixtures validated; 59 unique IDs; all sections present`).
- Auth handler tests cover login success/cookies, missing refresh token, and logout session revocation/cookie clearing.
- `./leedevkit test api`: BLOCKED by pre-existing Rust/agent-main workspace lint/build failures under current toolchain (`Atomic::fetch_update` and widespread `clippy::double_must_use`/`unused_async` errors). Go gates remain green.

## Remaining P6-01 proof

- Run auth fixtures against Rust and Go with response/body/header/side-effect normalization.
- Add live PostgreSQL/Redis integration evidence for session rotation, Redis blocklist, and lockout threshold.
- Add audit-event sink wiring and evidence for login/logout/security events.
