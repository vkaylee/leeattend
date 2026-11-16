# Roles and Job Titles Settings IA

Status: Approved

## Goal
Separate RBAC roles from employee job titles: roles stay under Settings > Access; job titles belong under Settings > Organization & People; remove duplicate role entry from the main sidebar.

## Constraints and Interfaces
- Preserve existing uncommitted user changes; touch only navigation, route metadata, translations, and the job-title contract required by approved scope.
- `roles` remain permission-bearing RBAC entities. `job_title` remains employment data unless catalog scope is explicitly approved.
- Keep authorization server-side and UI-filtered: roles use existing `roles:manage` / `members:roles:manage`; job-title management needs an explicit permission before any API mutation.
- Use i18n keys for all visible labels; keep settings routes centralized in `webdashboard/src/lib/routes.ts`.
- Rule IDs: `UI-UX-WCAG-AA`, `RBAC-DEFAULT-DENY`, `DEV-TEST-GATE`, `CLEAN-NO-DEAD-CODE`.

## Tasks
- [x] 1. Confirm scope: keep existing free-text `employment.job_title`; no managed job-title catalog or CRUD.
- [x] 2. Normalize RBAC naming in settings: change `Vai trò & Chức danh` to `Vai trò` in `use-settings-nav.tsx`, `route-metadata.ts`, and locale files; retain `/settings/roles` and existing permissions. Verify no remaining user-facing mixed label.
- [x] 3. Remove duplicate main-sidebar role shortcut from `use-nav-menu-items.ts`; keep `Settings` as the single entry. Update active-group behavior and navigation tests. Verify unauthorized users never see role links; authorized users reach role management through Settings.
- [x] 4. Skip catalog implementation: `job_title` remains employment-level free text; no API, migration, or new permission added.
- [x] 5. Update frontend tests for settings navigation, role labels, and main-sidebar regression. Existing settings loading, empty, filtering, and permission tests remain passing.
- [x] 6. Verification: focused pattern command found no files due Vitest filename matching; full unit suite and full web gate completed. Inspect result below.

## Test Impact Matrix
- Changed: visible settings grouping, sidebar links, route metadata, optional job-title management.
- Preserved: `/settings/roles` URL, role permissions, existing employment `job_title` values, all unrelated sidebar routes.
- Boundaries: empty settings groups, missing/duplicate titles, loading/error states, direct route access without permission.
- Security: role and job-title mutation denied without explicit permission; no client-only authorization.
- Compatibility: existing bookmarks to `/settings/roles`; old employment rows with free-text titles.

## Done When
- [x] Roles display only as RBAC under Settings > Access.
- [x] Main sidebar has no duplicate role-management shortcut.
- [x] Job-title placement matches approved scope and does not confuse it with RBAC.
- [x] i18n, accessibility, authorization, focused tests, and required LeeDevKit verification pass.

## Verification Results
- `./leedevkit test web --unit-only`: 320 test files passed, 2,989 tests passed.
- `./leedevkit test web`: E2E 43 passed, 9 skipped; i18n, typecheck, and lint passed.
- Existing lint warnings remain in repository baseline: 1,681 warnings, 0 errors.
- No API/schema changes; `./leedevkit test all` not required for this frontend-only IA change.

## Review Focus
- Whether job titles need a real organization-wide catalog or remain employment-level free text.
- Permission ownership for job-title CRUD.
- Backward compatibility for existing `job_title` values and direct URLs.
