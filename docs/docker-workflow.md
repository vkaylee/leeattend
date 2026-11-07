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

The repository MCP configuration starts it through `scripts/playwright-mcp.sh`:

```bash
./leedevkit manage up dev
```

Reload the MCP client after changing `.mcp.json`. Build the image once with `./scripts/playwright-auth-role.sh employee` or `podman-compose -p leedevkit-dev -f .compose/docker-compose.dev.yml --profile mcp build playwright-mcp`. Navigate to `http://caddy/` from Playwright MCP; do not use `http://localhost:8080` from inside the MCP browser. Role wrappers use the existing image and mount only the selected role state.

The MCP server runs headless with isolated browser state and enables every capability supported by the pinned CLI: `vision`, `pdf`, and `devtools`, plus all core navigation, interaction, snapshot, screenshot, console, and network-inspection tools. It also includes element bounding boxes in snapshots and exposes informational console messages. Browser requests remain restricted to `http://caddy`; no credentials or storage state are included in the image.

### Role-isolated authenticated MCP sessions

One storage state cannot safely represent five roles. The repository exposes one MCP entry per role, each with its own read-only state file:

| MCP server | Role | State |
|---|---|---|
| `playwright` / `playwright-employee` | `employee` | `.secrets/playwright-auth/employee.json` |
| `playwright-team-lead` | `team_lead` | `.secrets/playwright-auth/team-lead.json` |
| `playwright-shift-manager` | `shift_manager` | `.secrets/playwright-auth/shift-manager.json` |
| `playwright-hr-manager` | `hr_manager` | `.secrets/playwright-auth/hr-manager.json` |
| `playwright-admin` | `admin` | `.secrets/playwright-auth/admin.json` |

Use the least-privileged profile that covers the scenario. `playwright` defaults to `employee`; use `playwright-admin` only when the scenario requires administrator permissions. Never switch roles inside one browser session. For authorization tests, run the same flow separately with the expected-deny and expected-allow profiles.

Create all required states from a local ignored credentials file. The file must contain only these role-specific keys as needed:

```dotenv
PLAYWRIGHT_EMPLOYEE_EMAIL=employee@example.invalid
PLAYWRIGHT_EMPLOYEE_PASSWORD=change-me
PLAYWRIGHT_TEAM_LEAD_EMAIL=team.lead@example.invalid
PLAYWRIGHT_TEAM_LEAD_PASSWORD=change-me
PLAYWRIGHT_SHIFT_MANAGER_EMAIL=shift.manager@example.invalid
PLAYWRIGHT_SHIFT_MANAGER_PASSWORD=change-me
PLAYWRIGHT_HR_MANAGER_EMAIL=hr.manager@example.invalid
PLAYWRIGHT_HR_MANAGER_PASSWORD=change-me
PLAYWRIGHT_ADMIN_EMAIL=admin@example.invalid
PLAYWRIGHT_ADMIN_PASSWORD=change-me
```

Generate only the role needed:

```bash
./scripts/playwright-auth-role.sh employee
./scripts/playwright-auth-role.sh team-lead
./scripts/playwright-auth-role.sh shift-manager
./scripts/playwright-auth-role.sh hr-manager
./scripts/playwright-auth-role.sh admin
```

Every role wrapper performs a session preflight before starting MCP. It checks the selected state against `GET http://caddy/api/v1/me`; valid state is reused immediately. Missing or invalid state triggers `scripts/playwright-auth-role.sh <role>`, which recreates only that role's state from the ignored credentials file. If credentials are unavailable or login fails, MCP fails closed with an actionable error instead of starting with an anonymous or wrong-role session.
### MCP process and browser timeouts

The role wrapper limits each MCP and auth Compose process to 15 minutes by default. It sends `TERM` when the limit is reached, waits 15 seconds for cleanup, then sends `KILL` if the process remains alive. Compose runs keep `--rm --no-deps`, so exited MCP containers are removed.

Override only for unusually long local flows:

```bash
PLAYWRIGHT_MCP_MAX_RUNTIME=1800 PLAYWRIGHT_MCP_KILL_GRACE=30 \
  ./scripts/playwright-mcp-role.sh team-lead
```

`PLAYWRIGHT_MCP_MAX_RUNTIME` accepts `1..3600` seconds. `PLAYWRIGHT_MCP_KILL_GRACE` accepts `1..300` seconds. Invalid values fail before Compose starts. A timeout reports `Playwright process timed out` and returns non-zero.

Browser-level defaults are explicit: action `5s`, navigation `30s`, and settle `1s`. These stop individual browser operations from waiting indefinitely; they do not end an otherwise healthy MCP session. The `/api/v1/me` session preflight remains bounded to `10s`.

```bash
rm -f .secrets/playwright-auth/team-lead.json
```

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
