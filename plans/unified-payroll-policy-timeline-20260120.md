# Implementation Plan: Unified Payroll Policy Timeline

### ## Approach
- **Problem**: Redundant UI between the standalone "Standard Days" form and the "History" table. Empty history causes confusion when the form shows defaults.
- **Solution**: 
    - Consolidate all payroll policy management into the `TemporalConfigTable`.
    - Remove the standalone `PayrollSettingsForm` from `PayrollSettingsSection`.
    - Ensure the table always shows at least one record (the system default if no history exists).
    - Highlight the currently active policy (Effective Date <= Now) with a distinct "Active" badge and priority styling.

### ## Steps

1. **Refactor Component Composition** (15 min)
    - File: `webdashboard/src/components/payroll/PayrollSettingsSection.tsx`
    - Remove `PayrollSettingsForm` definition and usage.
    - Transform `PayrollSettingsSection` into a focused container that fetches history and current settings, then passes them to an enhanced `TemporalConfigTable`.

2. **Enhance TemporalConfigTable UI/UX** (30 min)
    - File: `webdashboard/src/components/common/TemporalConfigTable.tsx`
    - **Active Highlighting**: Add a visual wrapper or background for the "Active" row.
    - **Empty State Guard**: If `records` is empty, inject a "Virtual Default Record" (26 days, 8 hours) with an "Initial Setup" tag.
    - **Header Integration**: Move the "Calculator" icon and main title into the table header for a unified look.

3. **Improve Action Flow** (15 min)
    - File: `webdashboard/src/components/common/TemporalConfigTable.tsx`
    - Streamline the "Add New" form to feel like a "Policy Update" rather than just a log entry.
    - Add clear tooltips explaining how the "Effective Date" affects payroll calculation.

4. **Verify & Localize** (15 min)
    - Update `vi.json` and `en.json` to reflect the unified terminology (e.g., "Manage Policy" instead of "Config History").
    - Run `npm run type-check` and verify the build.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Refactor Composition | 15 min |
| UI/UX Enhancement | 30 min |
| Logic & Flow Improvement | 15 min |
| Verification | 15 min |
| **Total** | **1 hour 15 min** |

### ## Rollback Plan
- Revert `PayrollSettingsSection.tsx` and `TemporalConfigTable.tsx` to their states before this plan.

### ## Security Checklist
- [x] Input validation (Ensure days/hours are positive numbers).
- [x] Date validation (Effective date must be valid).
