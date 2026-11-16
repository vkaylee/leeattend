# Phase E — UI route × role acceptance matrix

Viewport: `desktop-large` — 1920x1080. Before each scenario: resize, verify innerWidth/innerHeight.

| Role | Allowed routes/actions | Expected denied routes/actions | Console | Network |
|---|---|---|---|---|
| employee | `/reports` own permitted views; `/attendance-logs`; own closure status if exposed | close/reopen/reclose; management/settings/infra direct URLs | capture | capture |
| team_lead | `/reports`; `/attendance-logs`; team-scoped approval views | closure mutation without permission; HR/admin/settings/infra | capture | capture |
| shift_manager | `/reports`; `/attendance-logs`; schedule/swap routes | attendance closure mutation; HR/payroll/admin/infra | capture | capture |
| hr_manager | `/reports`; `/attendance-logs`; closure status/close/reopen/reclose where permission granted | infra/system admin; unrelated shift administration | capture | capture |
| admin | all lifecycle routes and actions | only explicit forbidden cases | capture | capture |

Required state cases per authorized role: loading, empty, error, stale version, locked payroll, missing coverage, open punch. Denied direct URLs/actions must yield expected 403/redirect, not unexpected 404/5xx/crash/wrong navigation.

Evidence fields: role, route, action, final URL, visible state, console messages, network request/status, expected-vs-unexpected authorization result. Mask credentials, tokens, IDs, and PII.

Untested routes: list after execution with concrete reason.
