# Gate ESS Behind Runtime Feature Flag

Status: Approved — implementation in progress

## Goal
Disable Employee Self-Service by default without deleting ESS code/data; enable it later through existing runtime feature-flag management.

## Constraints and Interfaces
- **Flag:** `ff_ess`; centralized `feature_flags` row; default `false`; no environment-variable or localStorage override for this production behavior.
- **Runtime contract:** authenticated clients receive `ff_ess` through `/api/v1/feature-flags/effective`; backend remains authoritative.
- **Disabled contract:** ESS UI is hidden; direct ESS navigation redirects to dashboard; every `/api/v1/ess/*` request returns structured `403 feature_disabled` before repository/business logic runs.
- **Preserve:** ESS tables, handlers, hooks, translations, and migrations; unrelated attendance, leave, payroll, and admin feature flags stay unchanged.
- **Rules:** `configuration-management.md` §§5–6, `change-management.md` §6, `development-workflow.md` §§2–7, `testing-standards.md` §§1–4, `access-control.md` authorization boundary.

## Tasks

- [ ] **1. Register flag and lifecycle metadata.** Add a forward migration inserting `ff_ess=false`, description, owner, creation date, and removal target; add reversible down migration; add `docs/feature-flags.md` entry. → Verify: migration applies and rolls back on disposable DB; row is disabled by default; no existing migration is rewritten.
- [ ] **2. Add backend feature-gate middleware.** Reuse `OrganizationRepository` flag lookup behind a short TTL cache; add structured `feature_disabled` error response; mount gate around `registerESSRoutes` after authentication and before ESS handlers. Fail closed on missing/error state. → Verify: disabled returns `403` with stable error key and no ESS repository call; enabled reaches handler; unauthenticated requests remain `401`; cache avoids per-request DB reads and expires.
- [ ] **3. Add frontend flag metadata.** Add `ff_ess: false` to `FEATURE_FLAGS` and catalog with owner/removal target; make all ESS app-registry entries and sidebar links use this flag, including command-palette/search results. → Verify: existing feature-flag and registry tests prove disabled filtering and enabled rendering; no ESS navigation item leaks while bootstrap/API fails.
- [ ] **4. Guard direct ESS routes.** Add one ESS route-layout guard using the existing runtime flag loader/subscribe mechanism; wait for flag bootstrap before deciding; redirect disabled deep links to `ROUTES.HOME`/dashboard while preserving normal auth redirects and lazy loading. → Verify: each ESS route redirects when disabled, renders when enabled, and does not flash ESS content during bootstrap/error.
- [ ] **5. Protect client request behavior.** Keep existing ESS hooks unchanged unless required; handle `feature_disabled` as expected disabled state rather than repeated retries/toasts. → Verify: disabled ESS page produces no request loop, no repeated console errors, and no mutation is sent; enabled flow remains unchanged.
- [ ] **6. Add regression coverage.** Backend tests cover flag disabled/enabled, auth ordering, structured error, missing flag, repository failure, and no side effects. Frontend tests cover default false, runtime true/false, menu/search filtering, route guard, bootstrap failure, and all ESS route paths. → Verify: focused API/web tests pass, then full `./leedevkit test all` passes with integration/E2E phases.
- [ ] **7. Verify role matrix and operational behavior.** Test documented roles with least privilege: `employee` ESS denied while flag off and available when on; `team_lead`, `shift_manager`, `hr_manager`, and `admin` get same flag behavior; confirm feature-flag management remains available to authorized admins. Record console/network evidence and untested routes. → Verify: route × role acceptance matrix has no unexpected `404`, `5xx`, crash, wrong redirect, or retry storm.

## Test Impact Matrix

- **Changed behavior:** ESS availability controlled by `ff_ess`; UI and API both fail closed.
- **Existing behavior:** non-ESS routes, auth, flag management, and enabled ESS behavior remain unchanged.
- **Boundaries:** missing row, API failure, stale cache, bootstrap race, direct deep link, malformed/unauthorized requests.
- **Security:** unauthenticated, forbidden role, direct API access, and cross-tenant access cannot bypass gate.
- **Operations:** cache TTL, structured error, no PII logging, audit events for flag changes.
- **Compatibility:** additive migration and existing flag API payload shape preserved.

## Done When

- [ ] ESS is disabled by default from one centralized runtime flag.
- [ ] UI, deep links, and every ESS API endpoint obey same flag.
- [ ] Flag changes remain audit-logged through existing management flow.
- [ ] Docs identify owner and removal target.
- [ ] Focused tests and full LeeDevKit suite pass; results and limitations reported.
