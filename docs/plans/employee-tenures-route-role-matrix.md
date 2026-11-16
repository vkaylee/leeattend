# Employee Tenure Route × Role Acceptance Matrix

Status: verification in progress
Date: 2026-11-16

Credentials: documented development fixtures only; account identifiers masked in evidence.
Baseline viewport: 1920×1080. Required secondary desktop: 1440×900, 1366×768, 1280×800.

| Role | Allowed tenure scenarios | Expected denied scenarios | Console/network evidence |
|---|---|---|---|
| employee | `/ess/profile`; current-tenure ESS profile, leave, attendance | `/employees`, HR settings, payroll administration, `/infra/monitor`; redirect or 403 | capture URL, console errors, API status |
| team_lead | dashboard; team attendance/adjustment and OT approval flows | employee administration, HR settings, payroll administration, `/infra/monitor`; redirect or 403 | capture URL, console errors, API status |
| shift_manager | shifts, working patterns, shift swaps | employee administration, HR settings, payroll administration, `/infra/monitor`; redirect or 403 | capture URL, console errors, API status |
| hr_manager | `/employees`, employee detail/history, hire/transfer/terminate/rehire, leave, attendance, payroll | `/infra/monitor` and system administration; redirect or 403 | capture URL, console errors, API status |
| admin | organization, employees, settings, payroll, `/infra/monitor`; tenure history and rehire flows | only explicitly forbidden routes | capture URL, console errors, API status |

## Scenarios

- Current tenure visible to linked employee account.
- Historical tenure data not visible to newly linked employee account.
- HR hire, transfer, terminate, rehire with reused number, rehire with new number.
- Direct old-tenure resource access denied for employee account.
- Expected authorization failures remain clean 403/redirect; unexpected 404, 5xx, blank screen, crash, repeated toast, wrong navigation are defects.

## Evidence status

- Admin login and `/employees` smoke: 1920×1080, `/employees` API 200, console 0 errors.
- Admin viewport checks: 1440×900, 1366×768, 1280×800 verified on `/employees`; console 0 errors.
- Employee `/ess/profile`: retest after fixture reload succeeded at `1920x1080`; `/api/v1/ess/profile` returned 200, console errors 0, current profile rendered. Earlier 403 was stale session/fixture state, not reproduced.
- Employee direct `/employees`: API 403 as expected, but page emitted 6 console errors from forbidden employee/custom-field queries. Authorization denial is correct; background query error handling is noisy and needs separate cleanup.
- Admin `/infra/monitor`: monitor alerts/jobs/overview APIs 200, console errors 0 at 1920x1080.
- HR manager direct `/infra/monitor`: redirected to `/`; no monitor API call. Dashboard emitted 400 errors for `/api/v1/ess/time-entries` during landing-page preload; unrelated HR route noise but must be tracked before final UI gate.
- HR manager `/employees`: API 200, console 0 errors at 1920×1080.
- Shift manager `/shifts`: API 200 for shifts, working patterns, assignment history; console 0 errors at 1920×1080.
- Team lead `/approvals`: page remained `/approvals`; `/api/v1/employees?page=1&limit=3000` returned expected 403, but console recorded QueryClient 403 error. Route/API contract should suppress expected forbidden background query.
- Remaining required actions: HR lifecycle actions, reused/new-code rehire, old-tenure denial, all secondary viewports for every role, and admin direct denial checks.
- HR manager `/employees` and employee detail: 1920×1080, 1440×900, 1366×768, 1280×800 verified; API 200, console errors 0. Existing detail shows `Recruitment tenure history`, `Transfer`, and `Terminate`; no fixture history rows available. Lifecycle mutations not executed against shared fixture.
- Employee `/ess/profile`: 1920×1080, API 200, console errors 0. Direct `/employees`: expected 403, but management shell logs 6 QueryClient errors from forbidden background requests; fixed approval query suppression/date-only ESS clock bounds in commit `29efd4613`, browser retest pending.
- Team lead `/approvals`: 1920×1080, expected employee-list 403; prior QueryClient 403 noise fixed in `29efd4613`, browser retest pending.
- Shift manager `/shifts`: 1920×1080, shifts/working-patterns/assignment-history APIs 200, console errors 0, final route `/shifts?tab=weekly-roster&view=week`.
- Admin `/infra/monitor`: 1920×1080, monitor APIs 200, console errors 0.
- HR landing still emits two `/api/v1/ess/time-entries` 400 responses before route navigation; date-only hook fix committed, browser retest pending.
- PostgreSQL tenure integration: PASS through `./scripts/_go-gates.sh --integration --package ./internal/repository --test-pattern EmployeeTenures`; both tenure tests passed against disposable migrated DB.
- Historical reader audit slice updated assignment, organization-condition, and organization leave-balance search projections to resolve/search tenure numbers instead of only the person-level legacy field. Characterization tests passed; full API gate passed.
- Employee detail and ESS profile projections now resolve current tenure number with legacy fallback. Characterization tests passed; full API gate passed.
- Document template variable projection is tenure-aware by employment start date; finalized/signed document storage has no mutable employee-number field, so no snapshot rewrite was performed. Characterization test and full API gate passed.
- Disposable PostgreSQL report integration now proves daily attendance report returns `old-1541` for historical employment date after current tenure changed. Integration gate passed.
- Full web gate remains blocked by unrelated concurrent changes; do not infer route audit pass from unit/E2E results.
