# Systematic UI Route Optimization

- UI-AUDIT-01: Audit all documented roles and representative allowed/denied routes with MCP evidence before editing.
- UI-SHARED-02: Fix repeated behavior at shared shell/design-token/startup layers before page-specific components.
- UI-A11Y-03: Preserve keyboard access, focus visibility, dialog Escape handling, and focus restoration.
- UI-VERIFY-04: Re-run desktop/mobile route smoke, console/network evidence, then `leedevkit test web`.

## Test impact matrix

- Changed behavior: runtime feature-flag bootstrap no longer calls protected API before authentication; mobile nav drawer handles Escape, focus trap, and focus restoration.
- Preserved behavior: authenticated feature flags load after login/reload; desktop sidebar/navigation and route links remain unchanged.
- Boundaries: unauthenticated login, authenticated reload, mobile drawer open/close, empty focusable list, desktop viewport.
- Failures: feature-flag API failure remains fail-closed; drawer close remains usable by button/backdrop/Escape.
- Security: no credential or token changes; expected unauthenticated request noise removed.
- Compatibility: React 19, TanStack Router, existing Sidebar tests and mobile layout.

## Department cleanup test impact

- Changed behavior: departments route keeps list/table management only; removes department cards and standard-workday/payroll configuration modal.
- Preserved behavior: create, edit, delete, search, manager filter, detail drawer, and responsive list actions remain available.
- Boundaries: empty list, filtered empty list, mobile card rendering, desktop table rendering, and action menu without configuration item.
- Failures: department CRUD and detail drawer remain isolated from removed payroll configuration API.
- Security: remove no authorization boundary; existing department permission gates remain.
- Compatibility: generated routes unchanged; deleted config component has no remaining callers.
