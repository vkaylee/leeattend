# Playwright MCP zombie-process timeout

Status: Implemented — verification in progress

## Goal
Prevent stalled Playwright MCP/auth containers from surviving indefinitely while preserving multi-step browser sessions and Caddy-only browser routing.

## Decision
Use layered timeouts:

- MCP browser actions: explicit action/navigation/settle limits passed to the pinned MCP image.
- Interactive MCP lifetime: the watchdog starts before the role wrapper's first external operation, including Compose startup. Startup and pending-request silence have a 10-second deadline with up to 2 seconds reserved for termination. Healthy sessions without pending requests may remain idle.
- Interactive authentication: no automatic credentials or saved state. The browser context is isolated and starts at login UI.
- Container cleanup: preserve `--rm --no-deps`; timeout returns `124` and process-group cleanup is bounded.

Only MCP server stdout progress resets a pending-request deadline. Client notifications cannot keep a stuck request alive. The project Python environment is required before startup.

## Constraints and interfaces

- Keep browser requests on `http://caddy`; do not expose or use the host Caddy port from the browser container.
- Start every interactive session unauthenticated; do not mount role state or invoke auth preflight/login.
- Never log credentials, cookies, tokens, or raw PII.
- Use existing Compose/wrapper commands and repository validation targets.
- No changes to React application behavior or production services.
- Keep MCP stdio transport intact; do not convert to an HTTP server.

## Tasks

- [x] Define the startup/pending-response timeout contract in `scripts/playwright-mcp-role.sh` and the stdio watchdog. Preserve standalone auth job limits.
  - Verify: role wrapper startup is supervised; login-first smoke reaches Caddy, and JSON-RPC fixtures cover startup silence, pending requests, progress, healthy idle, stdio forwarding, and cleanup.

- [x] Start supervision before Compose startup; forward MCP stdio unchanged, track request IDs and replies, preserve healthy idle sessions, and terminate silent startup or pending requests within the configured deadline.
  - Verify: focused fixtures pass 9 tests, including forced termination.

- [x] Bound auth image build/run operations with the existing hard runtime timeout; preserve temporary credential file cleanup on all exits.
  - Verify: success, missing credentials, invalid credentials, Compose failure, and forced timeout all remove temporary credential material and do not create partial role state.


- [x] Pass explicit pinned-MCP browser timeout options in `.compose/docker-compose.dev.yml` only after confirming `@playwright/mcp@0.0.84` accepts them; use conservative values: action 5s, navigation 30s, settle 1s.
  - Verify: image help/config accepts options; normal navigation and API-backed SPA actions work; a deliberately stalled navigation fails within the configured bound.

- [x] Update `docs/operations/docker-workflow.md` with defaults, environment overrides, signal behavior, expected timeout errors, and the distinction between request timeout and process lifetime.
  - Verify: documented commands match wrapper variables and no secret material appears.

- [ ] Commit as one focused change after validation; do not mix unrelated MCP or frontend changes.
  - Verify: staged diff contains only timeout/watchdog files and documentation.

## Test impact matrix

- Changed behavior: startup and unanswered JSON-RPC requests are bounded to 10 seconds; healthy idle sessions remain alive and interactive sessions begin at login UI.
- Existing behavior: Caddy-only routing and MCP stdio contract remain unchanged.
- Boundaries: missing, zero, negative, non-numeric, minimum, default, and maximum timeout values.
- Failures: Compose unavailable, image build failure, API timeout, login timeout, MCP idle timeout, graceful termination, forced kill.
- Security: credentials remain mode `600`, temporary files are deleted, no state/token/password logging.
- Compatibility: Docker Compose and Podman Compose paths; pinned MCP version; Python watchdog availability.
- Operations: exit code, logs, orphan container/process check, and cleanup after client disconnect.

## Done when

- [x] Interactive MCP startup and pending-request silence are bounded to 10 seconds; unauthenticated sessions begin at the login UI.
- [x] Graceful shutdown precedes forced termination.
- [x] Browser action/navigation limits remain explicit and documented.
- [x] MCP stdio reaches Caddy: initialize response observed at 1.15s, `/login` navigation response at 2.00s.
- [ ] Full LeeDevKit suite: blocked by existing Go compile/lint failures and web TypeScript/E2E errors; focused watchdog tests and Ruff pass.
- [ ] Working tree contains only the reviewed commit or intentionally documented user changes.

## Review focus

- Whether the stdio watchdog forwards all MCP bytes without changing protocol behavior.
- Whether response progress resets a pending deadline without client notifications masking a stalled request.
- Whether auth build timeout accidentally removes a valid state or a credential temp file.
