# Implementation Plan: Restructure Settings Tabs & Relocate Rules

### ## Approach
- **Context**: Current settings pages (`PayrollSettings`, `TimeSettings`) are monolithic lists that mix unrelated logic (Time Assignment in Payroll) and feel cluttered.
- **Solution**: 
    1.  **Relocate**: Move "Auto-Assignment Rules" (which assign Time Patterns) from Payroll to Time Settings.
    2.  **Tabbed UI**: Implement a clean tabbed interface for both pages to reduce vertical scrolling and cognitive load.
    3.  **Cohesion**: Group "Base Config" and "History" in Payroll into a single "Policy" tab to show temporal evolution clearly.

### ## Steps

1. **Relocation & i18n Update** (5 min)
    - Update `en.json`, `vi.json` to ensure "Assignment Rules" labels are available under Time category if needed.

2. **Refactor Time Settings** (20 min)
    - File: `webdashboard/src/pages/settings/TimeSettings.tsx`
    - Implement `useState` for tabs.
    - **Tab 1 (General)**: Attendance Config (Work Unit, Rounding, Pairing).
    - **Tab 2 (Patterns)**: `WorkingPatternsPage`.
    - **Tab 3 (Rules)**: `AssignmentRulesPage` (Moved here).

3. **Refactor Payroll Settings** (20 min)
    - File: `webdashboard/src/pages/settings/PayrollSettings.tsx`
    - Implement `useState` for tabs.
    - **Tab 1 (Policy)**: `PayrollSettingsSection` (which contains both Form and History).
    - **Tab 2 (Components)**: `SalaryComponentManager`.
    - Remove `AssignmentRulesPage`.

4. **Visual Enhancement** (15 min)
    - Update `PayrollSettingsSection.tsx` to ensure the Form and History are visually connected (e.g., using a shared container or clear vertical flow).

5. **Verification** (15 min)
    - Run `npm run type-check` in `webdashboard`.
    - Manual verification of tab switching and data persistence.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Preparation | 5 min |
| Time Settings Refactor | 20 min |
| Payroll Settings Refactor| 20 min |
| Visual Polish | 15 min |
| Verification | 15 min |
| **Total** | **1 hour 15 min** |

### ## Rollback Plan
- Revert changes to `PayrollSettings.tsx` and `TimeSettings.tsx` using `git checkout`.

### ## Security Checklist
- [x] Auth checks (Verified: `getCurrentWorkspaceId` is used).
- [x] RBAC (Verified: Settings are only accessible by Admin/Owner via router guards).
