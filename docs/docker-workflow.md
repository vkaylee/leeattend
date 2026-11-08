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

The development Compose file includes a `playwright-mcp` profile. Its Chromium browser runs on `frontend_net` beside Caddy, so AI browser checks use `http://caddy/` and do not require `${DEV_PORT}` or any Caddy port published on the host.

The repository MCP configuration starts a single Playwright server through `scripts/playwright-mcp-role.sh`:

```bash
./leedevkit manage up dev
```

The server uses the least-privileged `employee` role by default. To use another role, change the role argument for the single `playwright` entry in `.mcp.json` to `team-lead`, `shift-manager`, `hr-manager`, or `admin`, then reload the MCP client. The wrapper adds role-specific guidance to the MCP `initialize` response and starts a fresh unauthenticated browser context; open the configured endpoint's `/login` first when authentication is needed. MCP instructions guide the model; application authorization remains the enforcement boundary. Never mount persisted role state.

The endpoint is declared with `--endpoint` in `.mcp.json`. The wrapper forwards that origin to Playwright as `--allowed-origins`; navigation remains limited to the configured origin. Set it to the host reachable from the browser container.

### Playwright MCP login-first sessions

Every interactive Playwright MCP session starts unauthenticated. Sign in through the UI only when a scenario requires authentication. Do not create or reuse `.secrets/playwright-auth/*.json` for interactive MCP.

### MCP process and browser timeouts

The role wrapper starts a stdio watchdog before Compose startup. Startup silence and pending JSON-RPC requests are bounded by `PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS` (default `10`, integer `1..60`). Server stdout progress resets the pending-request deadline; client notifications do not. A healthy session with no pending request may remain idle.

The watchdog reserves up to 2 seconds of that deadline for cleanup: it sends `TERM` first, then `KILL` at the deadline if necessary, and returns `124`. It uses the existing project Python environment directly, without dependency installation during MCP startup. If that environment is missing, prepare it with `./scripts/_python.sh --version` first. Restart the MCP server to load wrapper changes; editing files does not replace an already-running server.

Standalone `playwright-auth-role.sh` only provisions test state for non-interactive checks; it is not invoked by the interactive MCP server.

The MCP service uses a read-only container root. Runtime-writable paths (`/tmp`, Chromium cache/config directories, and `/dev/shm`) are memory-backed through `tmpfs`; they disappear when the container exits. Interactive MCP does not mount a persistent browser storage state. Sign-in state exists only in the isolated browser context for that session.

`.secrets/` is ignored by Git. Never copy role states to production or include them in screenshots, reports, logs, or commits.
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
