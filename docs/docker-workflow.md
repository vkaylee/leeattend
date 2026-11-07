# Podman Workflow and Multi-Environment Setup

LeeAttend uses layered Compose files for development, testing, CI, and production. Runtime services are the Go `apiserver`, React `webdashboard`, PostgreSQL, PgBouncer, Redis, and MinIO. Every application and migration connection uses the single PgBouncer endpoint in transaction mode with prepared-statement tracking enabled; PostgreSQL healthchecks and PgBouncer's upstream connection are direct.

## Compose layers

| Layer | File | Purpose |
|---|---|---|
| Production | `docker-compose.yml` | API, dashboard, databases, Redis, MinIO, healthchecks |
| Development | `.compose/docker-compose.dev.yml` | Go API and Vite hot reload with local mounts |
| Test | `.compose/docker-compose.test.yml` | Isolated databases, Redis, Go gates, Bun, Playwright |
| CI | `.compose/docker-compose.ci.yml` | CI-specific cache and resource settings |

## Production

```bash
podman-compose -f docker-compose.yml up --build -d
podman-compose -f docker-compose.yml ps
podman-compose -f docker-compose.yml down
```

The `apiserver` container runs the Go binary. `LOG_PULL_*` configuration is injected only into this service. The scheduler pulls external attendance pages; report WebSockets use Redis Pub/Sub.

## Development

```bash
podman-compose -f .compose/docker-compose.dev.yml up --build
```

The development API skips standalone license verification; production retains mandatory offline license verification. Database and Redis volumes are isolated by the development configuration.
Development migration gate:

`db-migrate` runs `apiserver/migrations/run-dev.sh` through PgBouncer. It records the baseline migration in `schema_migrations`, skips an already-applied baseline, and exits non-zero on SQL errors. Existing development volumes containing the baseline schema are marked applied without replaying the dump.


## Test gates

./leedevkit validate-compose
./leedevkit test api
./leedevkit test web
```

The pinned Go gate image in `.compose/go/Dockerfile` provides `gofmt`, `go vet`, `staticcheck`, `govulncheck`, `sqlc`, and `oapi-codegen`. API tests use the Go module under `apiserver/`; web tests use Bun and Playwright.

- Go module and build caches use named volumes.
- Web dependencies use a named Bun cache and isolated `node_modules` volume.
- Test databases run on the internal Compose network.
- All commands that can block or spawn containers run through the repository wrappers with stdin closed and output redirected by the caller.

## Operational contract

Provider tokens, raw attendance payloads, and employee identifiers MUST NOT appear in container logs. Pull cursors advance only after import transaction commit. `shadow` mode is the safe migration default; `write` mode requires reconciliation and rollback approval.
