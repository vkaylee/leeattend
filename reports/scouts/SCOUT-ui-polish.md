# Scout Report: UI Polish - Audit Logs & Scope Badge

## Exploration Scope
- Target: AuditLogPage filters, TeamLead scope badge
- Boundaries: `webdashboard/src/pages/settings`, `webdashboard/src/components/layout`, `webdashboard/src/lib/api`

## Patterns Discovered
### Pattern: Audit Log Fetching
- **Location**: `webdashboard/src/pages/settings/AuditLogPage.tsx`, `webdashboard/src/lib/api/audit.ts`
- **Usage**: Uses `@tanstack/react-query` to call `fetchAuditLogs(workspaceId, { page, limit })`. To add filters, `fetchAuditLogs` must be updated to accept `action`, `startDate`, and `endDate`, and pass them as URL query params.
- **Must Follow**: Yes

### Pattern: User Permissions & Roles
- **Location**: `webdashboard/src/hooks/use-permissions.ts`, `webdashboard/src/components/layout/Navbar.tsx`
- **Usage**: `useMyPermissions` hook provides `roles`, `hasRole`, and `managedDepartmentIds`. `Navbar.tsx` currently hardcodes role display. We should inject `useMyPermissions` to dynamically show the role and a "Department-scoped" badge for `TeamLead` when `managedDepartmentIds` is set (not null/undefined).
- **Must Follow**: Yes

## Integration Points
| Point | File | Function | New Code Location |
| ----- | ---- | -------- | ----------------- |
| Audit API | `src/lib/api/audit.ts` | `fetchAuditLogs` | Add filter params |
| Audit UI | `src/pages/settings/AuditLogPage.tsx` | `AuditLogPage` | Add filter inputs & state |
| Navbar Role | `src/components/layout/Navbar.tsx` | `Navbar` | Replace hardcoded role/badge |
