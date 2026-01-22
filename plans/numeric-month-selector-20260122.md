# Implementation Plan: Numeric Month Selector

Update all month selection interfaces in the WebDashboard to display numeric values (1-12) instead of text labels ("Tháng 1", "January", etc.), aligning with the minimal Metro-style UI.

## ## Approach
- **Component-level change**: Modify `MonthYearPicker.tsx` to render raw numbers in its grid.
- **Page-level labels**: Update labels in `PayrollPage.tsx` and `ReportsPage.tsx` that prefix months with "Tháng" or "T" to just show the number.
- **Why**: Cleaner, high-density UI and faster visual scanning for numeric-oriented users.

## ## Steps

1. **Modify MonthYearPicker Component** (10 min)
   - File: `webdashboard/src/components/common/MonthYearPicker.tsx`
   - Change the button content from `t("common.month_short", ...)` to just `{m}`.

2. **Update Payroll Page Labels** (10 min)
   - File: `webdashboard/src/pages/PayrollPage.tsx`
   - Update the period display label to remove the `t("common.month")` prefix.

3. **Update Reports Page Labels** (10 min)
   - File: `webdashboard/src/pages/ReportsPage.tsx`
   - Update the "Specific Month" filter label to remove the "T" prefix.

4. **Testing** (10 min)
   - Verify visually in Payroll and Reports pages.
   - Check mobile responsiveness.

## ## Timeline
| Phase | Duration |
|-------|----------|
| Implementation | 30 min |
| Testing | 10 min |
| **Total** | **40 min** |

## ## Rollback Plan
- Revert the 3 modified files using git.

## ## Security Checklist
- [x] No security impact.
- [x] Accessibility: Screen readers might read "1" instead of "Month 1". Adding `aria-label` is recommended if high accessibility is required.
