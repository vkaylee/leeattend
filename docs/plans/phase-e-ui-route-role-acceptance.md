# Phase E — UI route × role acceptance matrix

Status: **Incomplete / Pending full live capture** (updated 2026-11-25). Representative checks were previously noted, but full route/action, console, network, and state evidence has not been captured cleanly across all 5 roles. Frontend unit/component tests (`leedevkit test web`) do not satisfy live UI acceptance.

Viewport: `desktop-large` — `1920x1080` (plus mobile smoke where required). Before each scenario: announce viewport, call `browser_resize`, verify `window.innerWidth` / `window.innerHeight`. Mask credentials, tokens, IDs, and PII.

| Role | Allowed routes/actions | Expected denied routes/actions | Console | Network | Live status |
|---|---|---|---|---|---|
| `employee` | `/ess/clock`, `/ess/attendance`, `/ess/ot`, `/ess/profile`; own permitted `/reports` and `/attendance-logs` views; own schedule/closure read if exposed | `/shifts`, `/shifts/new`, `/shifts/$shiftId/edit`, `/shifts/rules`, `/working-patterns`, `/assignment-rules`, `/shift-swaps` approval, `/attendance-policy-assignments`, `/payroll`, `/settings/*`, `/infra/monitor`; close/reopen/reclose mutations | pending live capture | pending live capture | unverified in current session |
| `team_lead` | `/reports`, `/attendance-logs`, `/approvals`, `/approvals/ot`, `/attendance-adjustments`, team-scoped schedule read | closure close/reopen/reclose; shift/pattern/rule/policy admin mutations; `/payroll`, `/settings/*`, `/infra/monitor` | pending live capture | pending live capture | blocked (`playwright-team-lead` MCP `CONNECTION_CLOSED`) |
| `shift_manager` | `/reports`, `/attendance-logs`, `/shifts`, `/shifts/new`, `/shifts/$shiftId/edit`, `/shifts/rules`, `/working-patterns`, `/assignment-rules`, `/shift-swaps` (schedule/swap routes within scope) | attendance closure close/reopen/reclose; work-policy/payroll mutations; `/payroll`, `/settings/payroll`, `/settings/overtime`, `/infra/monitor` | pending live capture | pending live capture | blocked (`playwright-shift-manager` MCP `CONNECTION_CLOSED`) |
| `hr_manager` | `/reports`, `/attendance-logs`, `/attendance-policy-assignments`, `/attendance-adjustments`, `/approvals`, `/approvals/ot`, `/payroll`, closure status/close/reopen/reclose within granted employer scope | `/infra/monitor`, `/system/smtp`, out-of-scope employer mutations, unauthorized shift-only admin | pending live capture | pending live capture | blocked (`playwright-hr-manager` MCP `CONNECTION_CLOSED`) |
| `admin` | all lifecycle routes and actions (`/reports`, `/attendance-logs`, `/shifts*`, `/working-patterns*`, `/assignment-rules*`, `/shift-swaps`, `/attendance-policy-assignments`, `/payroll`, `/settings/*`, `/infra/monitor`) | only explicit forbidden/conflict business cases (stale version, locked payroll, missing coverage, open punch, self-reopen if enforced) | pending live capture | pending live capture | blocked (`playwright-admin` MCP `CONNECTION_CLOSED`) |

Required state cases per authorized role: loading, empty, error, provisional vs published/superseded/closed, stale version, locked payroll, missing coverage, open punch. Denied direct URLs/actions must yield expected `403`/redirect, not unexpected `404`/`5xx`/crash/wrong navigation.

Evidence fields required per executed scenario: role, route, action, final URL, visible state, console messages, network request/status, expected-vs-unexpected authorization result.

## Untested routes and blockers (2026-11-25)

- **Runtime blocker:** Local dev stack is not serving API (`leedevkit-dev-apiserver-1` exited, `leedevkit-dev-redis-1` exited; `GET http://127.0.0.1:8088/api/v1/health` returned `502`).
- **Browser MCP blocker:** Four role MCP servers (`playwright-admin`, `playwright-hr-manager`, `playwright-shift-manager`, `playwright-team-lead`) failed with `CONNECTION_CLOSED`; only `playwright-employee` connected.
- **Untested routes (all 5 roles pending live evidence):** `/reports`, `/attendance-logs`, `/shifts`, `/shifts/new`, `/shifts/$shiftId/edit`, `/shifts/rules`, `/working-patterns`, `/assignment-rules`, `/shift-swaps`, `/attendance-adjustments`, `/attendance-policy-assignments`, `/approvals`, `/approvals/ot`, `/payroll`, `/ess/clock`, `/ess/attendance`, `/ess/ot`, `/settings/time`, `/settings/overtime`, `/settings/payroll`, `/settings/feature-flags`, `/infra/monitor`.
- **Precondition to complete Phase E:** Start dev stack (`leedevkit dev up`), verify `/api/v1/health` returns `200`, reconnect all 5 Playwright MCP servers, then execute and record each route × role row with masked console/network evidence.
