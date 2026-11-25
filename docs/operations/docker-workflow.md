# Podman Workflow and Multi-Environment Setup

LeeAttend uses layered Compose files for development, testing, CI, and production. Runtime services are the Go `apiserver`, React `webdashboard`, PostgreSQL, PgBouncer, Redis, and MinIO. Every application and migration connection uses the single PgBouncer endpoint in transaction mode with prepared-statement tracking enabled; PostgreSQL healthchecks and PgBouncer's upstream connection are direct.

## Compose layers

| Layer | File | Purpose |
|---|---|---|
| Production | `docker-compose.yml` | API, dashboard, databases, Redis, MinIO, healthchecks |
| Development | `.compose/docker-compose.dev.yml` | Go API and Vite hot reload with local mounts |
| Test | `.compose/docker-compose.test.yml` | Isolated databases, Redis, Go gates, Bun, Playwright |
| CI | `.compose/docker-compose.ci.yml` | CI-specific cache and resource settings |
| AI browser | `.compose/docker-compose.dev.yml` profile `mcp` | Playwright MCP browser on the internal frontend network |


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

## Playwright MCP without Caddy host port

Playwright MCP runs via the global `leedevkit mcp playwright` subcommand, configured in `leedevkit.toml` under `[mcp.playwright]` and exposed as a single `playwright` server in `.mcp.json`. Its Chromium container attaches to `leedevkit-dev_dev_frontend` beside Caddy, so AI browser checks use `http://caddy/` without needing a published host port.

Build the shared image once (reused across all LeeDevKit repositories):

```bash
leedevkit mcp playwright --build-only
```

Without `--build`/`--build-only`, a missing image fails fast with a remediation message instead of holding the MCP handshake open.

### Login-first sessions & role switching

Every Playwright MCP session starts unauthenticated in an isolated browser context (`--isolated`, `--read-only`, `tmpfs` scratch paths, `--init`). Open `http://caddy/login` before authenticating. To switch roles, always call `browser_close` for a fresh isolated context, then open `http://caddy/login` and sign in as the next role. Viewport policy from `config/ui-viewports.json` is injected into the MCP `initialize` response automatically.

For disposable stacks, override the network/endpoint via flags or env vars:

```bash
leedevkit mcp playwright --network <disposable_frontend_net> --endpoint http://caddy
```

### Lifecycle watchdog

`leedevkit mcp playwright` runs a Go stdio watchdog with three separate bounds configured in `[mcp.playwright]`:
- `startup_timeout` (default `15`s): waiting for the `initialize` handshake.
- `request_timeout` (default `45`s): waiting for a pending JSON-RPC response (exceeds `--timeout-navigation=30000`).
- `session_idle_timeout` (default `300`s): idle after `initialize` (reset by completed `tools/call` responses).

On timeout, EOF, or `TERM`/`INT`, the watchdog closes child stdin, terminates the process group, force-removes the named container (`rm -f`) if still present, and returns exit code `124` on timeout.

`.secrets/` is ignored by Git. Never copy credentials or session tokens to screenshots, reports, logs, or commits.

Development migration gate:

`db-migrate` runs `apiserver/migrations/run-dev.sh` through PgBouncer. It records the baseline migration in `schema_migrations`, skips an already-applied baseline, and exits non-zero on SQL errors. Existing development volumes containing the baseline schema are marked applied without replaying the dump.


## Test gates

./scripts/_compose-validate.sh
leedevkit test api
leedevkit test web
```

The pinned Go gate image in `.compose/go/Dockerfile` provides `gofmt`, `go vet`, `staticcheck`, `govulncheck`, `sqlc`, and `oapi-codegen`. API tests use the Go module under `apiserver/`; web tests use Bun and Playwright.

- Go module and build caches use named volumes.
- Web dependencies use a named Bun cache and isolated `node_modules` volume.
- Test databases run on the internal Compose network.
- All commands that can block or spawn containers run through the repository wrappers with stdin closed and output redirected by the caller.

## Operational contract

Provider tokens, raw attendance payloads, and employee identifiers MUST NOT appear in container logs. Pull cursors advance only after import transaction commit. `shadow` mode is the safe migration default; `write` mode requires reconciliation and rollback approval.
