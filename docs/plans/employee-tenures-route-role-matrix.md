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
- Employee direct `/employees`: API 403 is expected. At `1280×800` after commits `a02e79969` and `4a780fb45`, no employee/custom-field API requests were emitted and console errors were 0.
- Admin `/infra/monitor`: monitor alerts/jobs/overview APIs 200, console errors 0 at 1920x1080.
- HR manager direct `/infra/monitor`: redirected to `/`; no monitor API call. After commit `254001f52`, HR landing at `1440×900` emitted no `/api/v1/ess/time-entries` request and console errors were 0.
- HR manager `/employees`: API 200, console 0 errors at 1920×1080.
- Shift manager `/shifts`: API 200 for shifts, working patterns, assignment history; console 0 errors at 1920×1080.
- Team lead `/approvals`: page remained `/approvals`; after commits `8dec9ba35` and `a02e79969`, no employee roster request was emitted and console errors were 0 at `1440×900`.
- Remaining required actions: HR lifecycle actions, reused/new-code rehire, old-tenure denial, clean `1366×768`/`1280×800` sessions for every role (several browser sessions hit `ERR_NETWORK_CHANGED`), and admin direct denial checks.
- HR manager `/employees` and employee detail: 1920×1080, 1440×900, 1366×768, 1280×800 verified; API 200, console errors 0. Existing detail shows `Recruitment tenure history`, `Transfer`, and `Terminate`; no fixture history rows available. Lifecycle mutations not executed against shared fixture.
- Employee `/ess/profile`: 1920×1080, API 200, console errors 0. Direct `/employees`: expected 403; later `1280×800` retest had no forbidden background requests and console errors 0.
- Team lead `/approvals`: 1920×1080 and 1440×900, no employee roster request after permission gating, console errors 0.
- Shift manager `/shifts`: 1920×1080, shifts/working-patterns/assignment-history APIs 200, console errors 0, final route `/shifts?tab=weekly-roster&view=week`.
- Admin `/infra/monitor`: 1920×1080, monitor APIs 200, console errors 0.
- HR landing time-entry noise fixed by active-employment gating in commit `254001f52`; `1440×900` retest produced no request and console errors 0.
- PostgreSQL tenure integration: PASS through `./scripts/_go-gates.sh --integration --package ./internal/repository --test-pattern EmployeeTenures`; both tenure tests passed against disposable migrated DB.
- Historical reader audit slice updated assignment, organization-condition, and organization leave-balance search projections to resolve/search tenure numbers instead of only the person-level legacy field. Characterization tests passed; full API gate passed.
- Employee detail and ESS profile projections now resolve current tenure number with legacy fallback. Characterization tests passed; full API gate passed.
- Document template variable projection is tenure-aware by employment start date; finalized/signed document storage has no mutable employee-number field, so no snapshot rewrite was performed. Characterization test and full API gate passed.
- Disposable PostgreSQL report integration now proves daily attendance report returns `old-1541` for historical employment date after current tenure changed. Integration gate passed.
- Current full web evidence remains failed: unit 326 test files passed/6 failed (3014 tests passed/39 failed); E2E 55 passed/5 failed. Typecheck/i18n and lint emitted no failure status, but the target is not green. Report-export E2E emitted `TypeError: Failed to fetch` and failed progress/background URL assertions; do not infer route audit pass from partial results.
- Browser sessions at 1920×1080 logged 403 `/api/v1/ess/profile` errors for management roles on dashboard; expected API authorization alone does not prove clean route behavior. Existing clean route evidence above remains limited to listed scenarios.
- Lifecycle mutations, old-tenure direct-ID denial, HR old/new number search, and rehire variants remain untested because browser fixture is shared and no disposable reset was available; no destructive action was executed.
- Required clean 1440×900, 1366×768, and 1280×800 sessions for every role remain incomplete. Several sessions previously hit `ERR_NETWORK_CHANGED`; those are not classified as app defects without clean retest.
- Disposable UI lifecycle retest at 1920×1080 was blocked before login: Playwright navigation to `http://127.0.0.1:18082/login`, `http://localhost:18082/login`, and `http://host.docker.internal:18082/login` returned `ERR_BLOCKED_BY_CLIENT`. No lifecycle mutation or credential action ran against disposable data.
- MCP shared-fixture retest at 1440×900 found unexpected authorization defects: `team_lead` direct `/employees` rendered and `/api/v1/employees` returned 200; `shift_manager` direct `/employees` rendered and `/api/v1/employees` returned 200. Both should redirect/403. `employee` and `hr_manager` direct `/infra/monitor` redirected to `/`, but each emitted `/api/v1/ess/profile` 403 and two console errors; expected-denied navigation is not clean.
