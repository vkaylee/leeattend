# Playwright MCP zombie-process timeout

Status: Implemented — verification in progress

## Goal
Prevent stalled Playwright MCP/auth containers from surviving indefinitely while preserving multi-step browser sessions and Caddy-only browser routing.

## Decision
Use layered timeouts:

- MCP browser actions: explicit action/navigation/settle limits passed to the pinned MCP image.
- Interactive MCP lifetime: a stdio watchdog resets on every MCP request, starts `TERM` after 45 seconds of silence, then uses a 15-second grace period; the container is gone within 60 seconds after the last request.
- Auth preflight/login: retain the existing 10-second `/api/v1/me` request limit and bounded auth-container execution.
- Container cleanup: retain `docker/podman compose run --rm --no-deps`; idle cleanup must return a distinct failure and leave no MCP container behind.

The 60-second limit is a hard maximum after the last MCP request. Auth build/login jobs retain their separate hard runtime timeout.

## Constraints and interfaces

- Keep browser requests on `http://caddy`; do not expose or use the host Caddy port from the browser container.
- Preserve role-isolated storage state and session preflight behavior.
- Never log credentials, cookies, tokens, or raw PII.
- Use existing Compose/wrapper commands and repository validation targets.
- No changes to React application behavior or production services.
- Keep MCP stdio transport intact; do not convert to an HTTP server.

## Tasks

- [x] Define timeout contract in `scripts/playwright-mcp-role.sh` and `scripts/playwright-auth-role.sh`: MCP idle deadline, auth hard runtime defaults, validation, exit-code/error wording, and cleanup behavior.
  - Verify: invalid values fail before Compose; defaults resolve deterministically; idle cleanup is distinguishable from login failure.

- [x] Add stdio idle watchdog around the MCP Compose foreground command; reset deadline on every MCP request, send `TERM` 15 seconds before the 60-second deadline, then `KILL` at the deadline if needed. Keep `--rm --no-deps` and propagate caller arguments unchanged.
  - Verify: controlled idle fixture receives `TERM`, request activity resets deadline, wrapper returns timeout status, and no orphan process remains.

- [x] Bound auth image build/run operations with the existing hard runtime timeout; preserve temporary credential file cleanup on all exits.
  - Verify: success, missing credentials, invalid credentials, Compose failure, and forced timeout all remove temporary credential material and do not create partial role state.


- [x] Pass explicit pinned-MCP browser timeout options in `.compose/docker-compose.dev.yml` only after confirming `@playwright/mcp@0.0.84` accepts them; use conservative values: action 5s, navigation 30s, settle 1s.
  - Verify: image help/config accepts options; normal navigation and API-backed SPA actions work; a deliberately stalled navigation fails within the configured bound.

- [x] Update `docs/docker-workflow.md` with defaults, environment overrides, signal behavior, expected timeout errors, and the distinction between request timeout and process lifetime.
  - Verify: documented commands match wrapper variables and no secret material appears.

- [ ] Commit as one focused change after validation; do not mix unrelated MCP or frontend changes.
  - Verify: staged diff contains only timeout/watchdog files and documentation.

## Test impact matrix

- Changed behavior: MCP containers terminate within 60 seconds after the last MCP request.
- Existing behavior: valid role state reuse, invalid-state refresh, Caddy routing, and MCP stdio interaction remain unchanged.
- Boundaries: missing, zero, negative, non-numeric, minimum, default, and maximum idle values.
- State: no state, valid state, expired state, partial state, and state creation interrupted by timeout.
- Failures: Compose unavailable, image build failure, API timeout, login timeout, MCP idle timeout, graceful termination, forced kill.
- Security: credentials remain mode `600`, temporary files are deleted, no state/token/password logging.
- Compatibility: Docker Compose and Podman Compose paths; pinned MCP version; Python watchdog availability.
- Operations: exit code, logs, orphan container/process check, and cleanup after client disconnect.

## Done when

- [x] Interactive MCP wrapper terminates within 60 seconds after the last request; auth wrapper retains bounded hard runtime.
- [x] Graceful shutdown occurs before forced kill.
- [x] Browser action/navigation/preflight limits are explicit and documented.
- [x] Role auth/session behavior remains intact.
- [x] Shell, Python, JSON, Compose, focused watchdog, and applicable LeeDevKit gates run; unrelated existing web unit failures remain documented in verification.
- [ ] Working tree contains only the reviewed commit or intentionally documented user changes.

## Review focus

- Whether the stdio watchdog forwards all MCP bytes without changing protocol behavior.
- Whether request activity resets the deadline under Docker Compose and Podman Compose.
- Whether auth build timeout accidentally removes a valid state or a credential temp file.
