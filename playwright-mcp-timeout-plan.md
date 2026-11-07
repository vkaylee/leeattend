# Playwright MCP zombie-process timeout

Status: Implemented — verification in progress

## Goal
Prevent stalled Playwright MCP/auth containers from surviving indefinitely while preserving multi-step browser sessions and Caddy-only browser routing.

## Decision
Use layered timeouts:

- MCP browser actions: explicit action/navigation/settle limits passed to the pinned MCP image.
- Auth preflight/login: retain the existing 10-second `/api/v1/me` request limit and add bounded auth-container execution.
- MCP process lifetime: wrap the Compose foreground process with GNU `timeout`, default 15 minutes, `TERM` first and `KILL` after a 15-second grace period.
- Container cleanup: retain `docker/podman compose run --rm --no-deps`; timeout must return a distinct failure and leave no MCP container behind.

The 15-minute limit is a hard maximum, not an idle timeout. It avoids zombie processes without guessing whether a quiet browser session is still intentional. Make it configurable through an environment variable with validation and a safe upper bound.

## Constraints and interfaces

- Keep browser requests on `http://caddy`; do not expose or use the host Caddy port from the browser container.
- Preserve role-isolated storage state and session preflight behavior.
- Never log credentials, cookies, tokens, or raw PII.
- Use existing Compose/wrapper commands and repository validation targets.
- No changes to React application behavior or production services.
- Keep MCP stdio transport intact; do not convert to an HTTP server.

## Tasks

- [x] Define timeout contract in `scripts/playwright-mcp-role.sh` and `scripts/playwright-auth-role.sh`: defaults, positive-integer validation, maximum, exit-code/error wording, and `TERM`/`KILL` grace period.
  - Verify: invalid values fail before Compose; defaults resolve deterministically; timeout failure is distinguishable from login failure.

- [x] Add process watchdog around the MCP Compose foreground command using `timeout --foreground --signal=TERM --kill-after=<grace>`; keep `--rm --no-deps` and propagate caller arguments unchanged.
  - Verify: a controlled short-timeout fixture receives `TERM`, then `KILL` only if needed; wrapper returns timeout status and does not orphan the child process/container.

- [x] Bound auth image build/run operations so a missing Docker/Podman response cannot leave auth setup running indefinitely; preserve temporary credential file cleanup on all exits.
  - Verify: success, missing credentials, invalid credentials, Compose failure, and forced timeout all remove temporary credential material and do not create partial role state.

- [x] Pass explicit pinned-MCP browser timeout options in `.compose/docker-compose.dev.yml` only after confirming `@playwright/mcp@0.0.84` accepts them; use conservative values: action 5s, navigation 30s, settle 1s.
  - Verify: image help/config accepts options; normal navigation and API-backed SPA actions work; a deliberately stalled navigation fails within the configured bound.

- [x] Update `docs/docker-workflow.md` with defaults, environment overrides, signal behavior, expected timeout errors, and the distinction between request timeout and process lifetime.
  - Verify: documented commands match wrapper variables and no secret material appears.

- [ ] Commit as one focused change after validation; do not mix unrelated MCP or frontend changes.
  - Verify: staged diff contains only timeout/watchdog files and documentation.

## Test impact matrix

- Changed behavior: stalled MCP/auth processes terminate instead of running indefinitely.
- Existing behavior: valid role state reuse, invalid-state refresh, Caddy routing, and MCP stdio interaction remain unchanged.
- Boundaries: missing, zero, negative, non-numeric, minimum, default, and maximum timeout values.
- State: no state, valid state, expired state, partial state, and state creation interrupted by timeout.
- Failures: Compose unavailable, image build failure, API timeout, login timeout, MCP timeout, graceful termination, forced kill.
- Security: credentials remain mode `600`, temporary files are deleted, no state/token/password logging.
- Compatibility: Docker Compose and Podman Compose paths; pinned MCP version; Linux coreutils `timeout` availability.
- Operations: exit code, logs, orphan container/process check, and cleanup after client disconnect.

## Done when

- [ ] No MCP or auth wrapper path can run indefinitely by default.
- [ ] Graceful shutdown occurs before forced kill.
- [ ] Browser action/navigation/preflight limits are explicit and documented.
- [ ] Role auth/session behavior remains intact.
- [ ] Shell, Node, JSON, Compose, focused watchdog, and full applicable LeeDevKit test gates pass.
- [ ] Working tree contains only the reviewed commit or intentionally documented user changes.

## Review focus

- Whether `timeout` signal propagation reaches the Compose-run container under both Docker Compose and Podman Compose.
- Whether the hard lifetime is long enough for intended manual browser workflows.
- Whether auth build timeout accidentally removes a valid state or leaves a credential temp file.
- Whether the pinned MCP version supports the selected timeout flags.
