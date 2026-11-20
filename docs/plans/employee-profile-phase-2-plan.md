# Employee Profile Phase 2 — Centralized Personnel Pages

Status: Approved

## Goal
Move **Career Phases** and **Attendance Policy Assignments** from employee profile tabs into permission-guarded, sidebar-backed organization pages. Keep Documents, Contracts, Salary, and Compensation in profile for a later security-reviewed batch.

## Rule compliance
- `PROFILE-UI-01` — `.agent/rules/design-rules.md`: responsive table/card layout, keyboard access, focus states, skeleton, empty/error states, i18n.
- `PROFILE-AUTH-02` — `.agent/rules/access-control.md`: handler authorization before business logic, fail closed, explicit resource permissions.
- `PROFILE-DATA-03` — `.agent/rules/data-governance.md`: collection responses stay T3; no salary, medical reason, email, or other T2 fields.
- `PROFILE-API-04` — `.agent/rules/api-governance.md`: contract-first, bounded collection, stable ordering, additive compatibility.
- `PROFILE-TEST-05` — `.agent/rules/testing-standards.md`: behavior tests plus integration/E2E through LeeDevKit; focused tests do not replace full regression.

## Existing constraints
- Existing phase APIs are employee/employment scoped in `lib/api/employee-phases.ts`; no organization collection endpoint exists yet.
- Existing policy assignment API is organization-scoped, but current UI loads all assignments and maps employee/department names client-side.
- Existing permissions: `employment_phases:read/manage`; policy assignment handlers currently use `organization:settings:read/write`.
- Existing employee drawer uses `employee-phases-tab.tsx` and `employee-attendance-policy-tab.tsx`.
- Preserve old bookmarks with redirects such as `?tab=phases` and `?tab=attendance-policy`.

## Tasks

- [ ] **1. Freeze contracts and authorization matrix.** Define routes `/employment-phases` and `/attendance-policy-assignments`; define URL search (`employeeId`, `employmentId`, `q`, status/date filters only where supported); decide collection columns and CRUD parity. Keep Salary/Compensation/Documents/Contracts out of scope. Verify: written route × role matrix for employee, team lead, shift manager, HR manager, admin; no UI implementation yet.

- [ ] **2. Add phase collection contract.** Add bounded, workspace-scoped organization list endpoint with stable ordering, employee/employment identifiers, phase definition, dates, status, and pagination/filter support. Reuse existing employee-scoped mutation endpoints only if authorization and invalidation remain correct; otherwise add additive collection mutation contract. Verify: API schema/client types, invalid UUID/empty page, tenant isolation, missing permission `403`, no N+1 query.

- [ ] **3. Split policy assignment permission and scope.** Prefer dedicated `attendance_policy_assignments:read/manage` permissions rather than exposing organization settings broadly; update canonical permissions/default roles/catalog and handlers. Add server-side employee/department filtering or a safe bounded response; never rely on frontend filtering. Verify: unauthorized read/write tests, scope tests, default-role matrix, no sensitive fields.

- [ ] **4. Build `/employment-phases` page.** Create route guard/lazy route, page, URL filter state, table/card responsive view, loading/empty/error/retry states, employee link, phase assignment modal, status update flow, and mutation cache invalidation. Reuse `EmployeePhasesTab`, `PhaseJourneyTimeline`, assignment/status modals where practical; do not duplicate business logic. Verify: direct URL, reload/back-forward, employee filter chip/clear, create/update/error flows, mobile and keyboard behavior.

- [ ] **5. Build `/attendance-policy-assignments` page.** Extract/reuse `PolicyAssignmentsSection` and `usePolicyAssignments` into a standalone page with employee filter, group/user filter, bounded lookup, create/delete confirmation, and visible permission-gated actions. Keep organization default rule editing in Settings unless explicitly required. Verify: empty/loading/error states, filtered employee URL, mutation failures, no unauthorized fetch, no raw T2 data.

- [ ] **6. Add sidebar and route compatibility.** Add both links to `use-nav-menu-items.ts`, route constants, active-group detection, translations, and mobile navigation through existing sidebar. Add `beforeLoad` guards that fail closed. Extend `employees_.$employeeId.tsx` redirects for `phases` and `attendance-policy`; invalid/unknown tabs remain on profile. Verify: no redirect loop, no lost `employeeId`, old bookmarks never `404`, sidebar hides links without permission.

- [ ] **7. Remove replaced profile tabs after parity checkpoint.** Remove phase and attendance-policy tab IDs/config, drawer renderers/imports, keyboard order entries, obsolete profile hint/link code, and only dead files/tests confirmed unused. Keep reusable modals/hooks/components still used by centralized pages. Update employee page/drawer tests and i18n. Verify: remaining profile tabs render, removed tabs absent, keyboard wrap skips them, profile CRUD remains intact.

- [ ] **8. Verification and role audit.** Run focused tests, then `leedevkit test web`, `leedevkit test api`, and `leedevkit test all`. Read each phase result, including integration/E2E. Build route × role acceptance matrix before browser testing; use documented least-privilege dev accounts and five isolated role sessions. Record route result, expected `403`/redirect versus unexpected `404`/`5xx`/crash, console errors, and network evidence.

## Done when
- [ ] Both pages provide required read/manage behavior without N+1 collection fetching.
- [ ] Backend authorization and role catalog match sidebar/route visibility.
- [ ] Old profile bookmarks redirect safely; removed tabs do not render.
- [ ] Documents, Contracts, Salary, and Compensation remain unchanged.
- [ ] Web, API, full regression, and applicable role-route audit evidence are recorded.

## Rollback
Revert page/sidebar/profile entry-point changes while retaining additive API contracts. Do not delete stored phase or policy data and do not remove existing API fields.

## Review focus
- Phase collection API must not silently resolve every employee through frontend fan-out.
- Policy assignment permission must not inherit broad settings access accidentally.
- Historical employment records must preserve `employment_id`; do not force active employment.
- Employee/department names in lists must remain T3 and avoid T2 leakage.
