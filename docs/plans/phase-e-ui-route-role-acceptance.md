# Phase E — UI route × role acceptance matrix

Status: **Live route/role audit verified for primary allowed/denied routes — dev-DB schema drift (`500` on `/payroll`) and interactive state mutations remain open** (updated 2026-11-25). Live dev stack (`GET /api/v1/health` = `200 OK`) and all 5 Playwright MCP role servers (`playwright-employee`, `playwright-team-lead`, `playwright-shift-manager`, `playwright-hr-manager`, `playwright-admin`) connected and executed against masked dev accounts from `docs/dev-accounts.md`.

Viewports verified via `browser_resize` + `window.innerWidth`/`window.innerHeight`:
- Desktop: `desktop-large` — `1920x1080` across all 5 roles.
- Mobile smoke: `mobile` — `390x844` across all 5 roles (`scrollWidth = 390`, 0 console errors, no horizontal overflow).

## Summary Matrix (Live Re-Verification After Route/Query Guard Fixes — 2026-11-25)

| Role (masked) | Tested allowed routes/actions | Tested denied routes/actions | Console (errors) | Network summary | Live status |
|---|---|---|---|---|---|
| `employee` (`emp***@example.invalid`) | `/`, `/ess/clock` (redirects `/` Quick Clock), `/ess/attendance` (redirects `/`), mobile smoke `/` (`390x844`) | `/shifts`, `/attendance-logs`, `/payroll`, `/infra/monitor` -> expected redirect `/` | 0 errors | All requests `200 OK` (`/me`, `/me/permissions`, `/lifecycle/my-tasks`); zero unauthorized `/shifts` or `/attendance/raw-logs` requests | **Passed for tested routes** |
| `team_lead` (`tl***@example.invalid`) | `/approvals` (`200`, 0 errors), mobile smoke `/approvals` (`390x844`) | `/reports`, `/attendance-adjustments`, `/payroll`, `/infra/monitor` -> expected redirect `/` | 0 errors | All requests `200 OK`; zero repeated `403` calls after route guards added | **Passed for tested routes** |
| `shift_manager` (`sm***@example.invalid`) | `/shifts` (`200`), `/shift-swaps` (`200`), `/working-patterns` -> `/shifts/rules?tab=patterns` (`200`), mobile smoke `/shifts` (`390x844`) | `/reports`, `/payroll`, `/settings/payroll` -> expected redirect `/` | 0 errors | `200 OK` on `/shifts`, `/working-patterns`, `/departments`, `/shift-swaps`, `/shift-swaps/stats`; zero `403` on `/employees` after permission gate | **Passed for tested routes** |
| `hr_manager` (`hr***@example.invalid`) | `/reports` (`200`), `/attendance-policy-assignments` (`200`), `/attendance-logs` (`200`), `/payroll` (`payslips` `200`, `adjustments?status=pending` **`500`** due to dev-DB drift), mobile smoke `/reports` (`390x844`) | `/infra/monitor` -> expected redirect `/` | `/reports`, `/attendance-policy-assignments`, `/attendance-logs`: 0; `/payroll`: 3 | `200 OK` on `/reports/attendance`, `/attendance/raw-logs`, `/attendance/policies/assignments`; zero `403` on `/staging-logs` after permission gate; **`500`** on `GET /api/v1/attendance/adjustments?status=pending` at `/payroll` | **Dev-DB drift blocker on `/payroll`** (all other tested routes passed) |
| `admin` (`adm***@example.invalid`) | `/reports`, `/shifts`, `/attendance-logs`, `/settings/feature-flags`, `/infra/monitor`, mobile smoke `/reports` (`390x844`) | N/A for route denial (business conflict mutations pending interactive capture) | 0 errors across all 5 desktop routes + mobile smoke | All API requests `200 OK` including `/attendance/staging-logs?page=1&limit=1` | **Passed for tested read routes** |

## Defects Remediated & Verified Live (2026-11-25)

1. **`employee` -> `/shifts` missing route guard (FIXED & VERIFIED):** added permission `beforeLoad` guard in `webdashboard/src/routes/_authenticated/_organization/shifts.tsx`; live re-check redirects `employee` to `/` with `0` console errors and `0` `/shifts` API calls.
2. **`team_lead` -> `/reports`, `/attendance-adjustments`, and `employee` -> `/attendance-logs` noisy `403` renders (FIXED & VERIFIED):** added `beforeLoad` permission guards in `reports.tsx`, `attendance-adjustments.tsx`, and `attendance-logs.tsx`; live re-check redirects cleanly to `/` with `0` console errors.
3. **`shift_manager` -> `/shifts/rules?tab=patterns` unconditional `/employees` `403` (FIXED & VERIFIED):** gated `fetchEmployees` in `use-working-patterns.ts` with `hasPermission("employees:read")`; live re-check loads with `0` console errors and `0` `403`s.
4. **`hr_manager` -> `/attendance-logs` unconditional `/staging-logs` `403` (FIXED & VERIFIED):** gated `stagingPages` and `stagingCount` in `use-attendance-logs-page.ts` with `hasPermission("attendance:staging:manage")`; live re-check loads with `0` console errors for `hr_manager` while `admin` still receives `200 OK`.

## Remaining Open Defect & Untested Scope

- **Dev-DB schema drift (`hr_manager` -> `/payroll` `500`):** `leedevkit-dev-apiserver-1` logged `ERROR: column a.adjustment_type does not exist (SQLSTATE 42703)` on `GET /api/v1/attendance/adjustments?status=pending` because the local `leedevkit-dev-db-1` volume predates migration `20261123000000_typed_attendance_adjustments`. Both incremental and phased fresh migrations include `adjustment_type`; re-verify `/payroll` after re-migrating the local dev DB.
- **Untested direct routes:** `/shifts/new`, `/shifts/$shiftId/edit`, `/assignment-rules`, `/approvals/ot`, `/ess/ot`, `/ess/profile`, `/settings/time`, `/settings/overtime`, `/system/smtp`.
- **Untested interactive state mutations:** UI trigger verification of provisional vs published/superseded/closed badges, stale version, locked payroll, missing coverage, open punch, and self-reopen error presentation (dev apiserver image predates branch commits; backend repository/API contracts are verified by DB integration tests in this branch, and live UI re-audit is required after rebuilding dev apiserver + re-migrating dev DB).
