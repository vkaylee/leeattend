# Long-Term Roles Management UI

Status: Approved

## Goal
Replace `/settings/roles` card grid with an accessible responsive role table that scales toward server-side filtering, pagination, and a role detail page without changing current RBAC behavior.

## Constraints and Interfaces
- Frontend-only Phase 1: preserve `useOrganizationRoles`, current API payload, `/settings/roles`, `roles:manage`, `members:roles:manage`, create/edit modal, delete confirmation, and permission matrix route.
- Use existing `SmartTable` only if its semantics and action-cell behavior fit; otherwise build the smallest responsive list/table component. Do not add a table abstraction for one screen.
- Do not load full permission/member data per row. Current role summary remains the only row data.
- System roles remain non-deletable; custom-role actions remain permission-gated. No hover-only actions.
- All visible labels use i18n. Preserve loading, empty, error, dark mode, keyboard focus, and mobile behavior.
- Rule IDs: `UI-UX-WCAG-AA`, `RBAC-DEFAULT-DENY`, `DEV-TEST-GATE`, `CLEAN-NO-DEAD-CODE`.

## Tasks
- [x] 1. Inspected existing `SmartTable`; chose a local role-list component because this screen has no sorting, editing cells, or pagination yet.
- [x] 2. Defined Phase 1 role row contract from current `OrganizationRole`: name, system/custom badge, description, and actions. Removed role ID and placeholder avatar presentation.
- [x] 3. Implemented responsive role list with desktop columns, mobile stacked descriptions, explicit action button labels, visible focus states, and no hover-only actions.
- [x] 4. Preserved permission and lifecycle behavior: create remains gated by `roles:manage`; edit/delete remain custom-role only; matrix route and delete confirmation remain intact.
- [x] 5. Added tests for list rendering, empty state, loading skeleton, unauthorized actions, and existing management actions.
- [x] 6. Kept Phase 2 seam at component callback/data boundaries; no API or route changes.
- [x] 7. Verification passed: focused role tests 5/5; full web gate passed with E2E 43 passed, 9 skipped, i18n/typecheck/lint passed. Existing lint warnings: 1,684 warnings, 0 errors.

## Test Impact Matrix
- Changed: role presentation from cards to table/list; action discoverability; responsive layout.
- Preserved: role data, API calls, RBAC permissions, route URLs, create/edit/delete/matrix behavior.
- Boundaries: zero roles, one role, many roles, long names/descriptions, missing descriptions, system roles, custom roles, mobile viewport, dark mode.
- Security: users without `roles:manage` cannot create/edit/delete; UI must not weaken server authorization; system roles cannot expose delete.
- Failure paths: loading, roles-query error state from existing hook, delete failure behavior, modal close/reopen.
- Accessibility: semantic table/list structure, action labels, keyboard menu access, visible focus, no hover-only controls.

## Done When
- [x] Card grid replaced by responsive role table/list.
- [x] No role ID truncation or fake member avatar remains in primary row UI.
- [x] Role actions remain permission-safe and keyboard-accessible.
- [x] Existing route, API, modal, confirmation, and matrix behavior preserved.
- [x] Focused tests and full web verification pass.

## Review Focus
- Whether `SmartTable` is appropriate for a mostly read-only role screen or adds unnecessary complexity.
- Whether error state is already exposed by `useOrganizationRoles` and needs a narrow UI addition.
- Whether row click should be deferred until role detail route exists; Phase 1 should avoid fake navigation.

## Future Phases
- Phase 2: server-side search, type/status filters, sorting, pagination, member count, permission count, updated timestamp.
- Phase 3: `/settings/roles/:roleId` with Overview, Permissions, Members, and Audit History.
- Phase 4: archive, clone, dependency checks before delete, and bulk assignment if required.
