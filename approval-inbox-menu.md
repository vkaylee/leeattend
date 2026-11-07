# Approval Inbox Menu Integration

Status: Implemented; verification blocked by isolated test environment startup

## Goal
Expose the existing approval inbox at `/approvals` through the primary navigation and remove the confirmed route, metadata, permission, and test drift found during audit.

## Scope and contracts
- Frontend-only navigation and contract-alignment change; no API or database changes.
- Preserve the existing dedicated `/approvals/ot` page.
- Use permissions already defined by the backend. Inbox visibility must use the existing navigation filter's any-permission semantics; do not introduce `attendance:adjustments:read`, which is not defined by the backend.
- Keep the inbox's four supported types unchanged: leave, overtime, attendance adjustment, and shift swap.

## Tasks
- [x] Add `ROUTES.APPROVALS` to the `time` group in `webdashboard/src/hooks/use-nav-menu-items.ts`, with existing backend permissions: `leave:requests:read`, `overtime:approve`, `attendance:adjustments:approve`, and `shift_swaps:manage`; add `/approvals` to `TIME_PATHS` so the active group remains correct.
- [x] Align the App Launcher approval entry in `webdashboard/src/lib/app-registry.ts` with the same existing permission names, keeping `approvals.inbox.title`, `approvals.inbox.subtitle`, `ROUTES.APPROVALS`, and `app-card-approvals` unchanged.
- [x] Correct `/approvals` title and breadcrumb metadata in `webdashboard/src/lib/route-metadata.ts` to use `approvals.inbox.title` with the existing Vietnamese fallback `Hộp thư Phê duyệt`.
- [x] Add `/approvals` to the route coverage list in `webdashboard/src/lib/__tests__/routes.test.ts` and add navigation assertions for sidebar visibility and `/approvals` active-group detection in the existing navigation tests.
- [x] Review the resulting diff for stale approval references only; do not remove `/approvals/ot`, workflow settings, or existing approval components because they remain distinct supported surfaces.

## Test impact matrix
- Changed behavior: `/approvals` appears in sidebar and App Launcher for users with any valid approval permission; active navigation highlights the time group; page metadata names the inbox.
- Existing behavior: `/approvals/ot` remains visible and routes unchanged; unrelated menu filtering remains permission-based.
- Boundaries: users with no listed approval permissions do not see the new item; users with exactly one valid permission do see it; undefined permission `attendance:adjustments:read` is no longer required.
- Security: navigation visibility is not authorization; backend endpoint checks remain authoritative. No API authorization bypass is introduced.
- Compatibility: route constant and generated route remain unchanged; no schema or dependency changes.
- Failure paths: no new network or persistence path; existing menu rendering and route tests cover the relevant failure-free contract.

## Verification
- Focused frontend tests for navigation, route coverage, and App Launcher permission filtering.
- Required project target: `./leedevkit test webdashboard` if supported by the repository target discovery; otherwise use the documented frontend target from `./leedevkit --help`.
- Smoke check: open the authenticated app with a permitted development account, confirm `Hộp thư Phê duyệt` appears in the menu, navigate to `/approvals`, and confirm `/approvals/ot` remains available.

- [x] `/approvals` is discoverable from primary navigation and App Launcher.
- [x] Navigation and App Launcher use only backend-defined permissions.
- [x] `/approvals` metadata displays inbox wording, not OT wording.
- [ ] Route, permission-filtering, and active-group tests pass. Blocked: `./leedevkit test web --unit-only` and `./leedevkit test web --lint-only --timeout 600 --quiet` stalled during isolated environment startup; unit retry exited after signal 15 cleanup.
- [ ] Full applicable LeeDevKit verification and authenticated UI smoke check pass. UI smoke check not run because the test/dev environment did not start successfully.
