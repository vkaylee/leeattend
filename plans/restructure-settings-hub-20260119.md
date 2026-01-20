# Implementation Plan: Restructure Settings Hub

Modernize the workspace configuration by centralizing scattered settings into a unified, categorized "Settings Hub". This reduces sidebar clutter and improves administrative workflow.

## Approach
- **Centralized Hub:** Create a nested layout for Settings with its own sub-navigation.
- **Categorization:** Group settings by domain: General, People, Time & Leave, Payroll, and Connectivity.
- **Sidebar Cleanup:** Collapse high-frequency configuration items (Patterns, Rules, Departments) into the Settings module.

## Steps

### 1. Preparation & Scaffolding (10 min)
- Create directory structure for settings pages.
- Identify shared state/logic in `WorkspaceSettingsPage.tsx` to be moved to hooks or context if needed.

```bash
mkdir -p webdashboard/src/pages/settings
mkdir -p webdashboard/src/components/settings
```

### 2. Core Implementation: Settings Layout (20 min)
- Create `SettingsLayout.tsx` to provide sub-navigation (sidebar or tabs) within the main content area.
- This layout will wrap all `/settings/*` routes.

### 3. Decomposition of WorkspaceSettingsPage (40 min)
Split the overloaded `WorkspaceSettingsPage` into smaller, focused components/pages:
- `GeneralSettings.tsx`: Workspace Info, Language, Timezone, Theme, Profile.
- `TimeSettings.tsx`: Attendance Config (Rounding, Units), Working Patterns migration.
- `PayrollSettings.tsx`: Salary Components, Assignment Rules migration.
- `PeopleSettings.tsx`: Members (RBAC), Departments migration.
- `ConnectivitySettings.tsx`: Agents, Devices migration.

### 4. Routing & Sidebar Integration (20 min)
- Update `webdashboard/src/App.tsx`:
    - Define a new `/settings` route group with nested children.
    - Update the `Sidebar` to remove individual items (Working Patterns, Rules, Departments, Agents, Devices, Members).
    - Add a single "Settings" item with a Badge or Sub-menu if desired.

```typescript
// Example Route Structure
<Route path="settings" element={<SettingsLayout />}>
  <Route index element={<Navigate to="general" replace />} />
  <Route path="general" element={<GeneralSettings />} />
  <Route path="people" element={<PeopleSettings />} />
  <Route path="time" element={<TimeSettings />} />
  <Route path="payroll" element={<PayrollSettings />} />
  <Route path="connectivity" element={<ConnectivitySettings />} />
</Route>
```

### 5. Verification & Cleanup (15 min)
- Ensure all deep links are redirected or updated.
- Verify translation keys are consistent.
- Run tests to ensure navigation still works correctly.

## Timeline
| Phase | Duration |
|-------|----------|
| Scaffolding | 10 min |
| Settings Layout | 20 min |
| Decomposition | 40 min |
| Integration | 20 min |
| Verification | 15 min |
| **Total** | **~1 hour 45 min** |

## Rollback Plan
1. Revert changes to `webdashboard/src/App.tsx` (restores sidebar and old routes).
2. Keep the new `pages/settings` files (they won't be active if routes are reverted).
3. Ensure `WorkspaceSettingsPage.tsx` is not deleted until migration is 100% verified.

## Security Checklist
- [x] Auth checks: Ensure only admins can access the `/settings` routes (verified by existing `WorkspaceGuard`).
- [x] RBAC: Verify `RoleManagementModal` still works within the new `PeopleSettings` view.
- [x] Input validation: Maintain existing validation for workspace updates.
