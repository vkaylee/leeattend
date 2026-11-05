# Rust Residue Retirement Record

Status: Completed

## Boundary

LeeAttend runtime is Go 1.26+ `net/http` with React/Vite/TypeScript, PostgreSQL, Redis, and MinIO. Rust/Axum migration material is historical evidence only.

## Retained

- `docs/migration/rust-go/` remains immutable migration evidence.
- Generic Rust templates, agents, containers, and tests under ignored `.leedevkit/` remain available for other projects.
- Database schema and compatibility metadata remain when required by the active Go migration path.

## Removed

- Duplicate `.attendagent/` context and local Claude settings.
- Stale LeeAttend Rust/Cargo/Diesel tooling and migration parity helpers.
- Obsolete smoke-mode plans targeting removed Rust test paths.
- Active documentation and wrapper references to removed `run-tests.sh`, Cargo, Diesel, Axum, Tokio, and SQLx workflows.

## Verification boundary

Searches may still find Rust terms in `docs/migration/rust-go/`, archived material, database compatibility metadata, and generic `.leedevkit/` support. These matches are non-operational for LeeAttend and must not be treated as active runtime instructions.

Use `./leedevkit validate-compose`, `./leedevkit test api`, `./leedevkit test web`, and `./leedevkit test all` for active LeeAttend verification.
