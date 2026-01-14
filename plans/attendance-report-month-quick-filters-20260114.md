# Implementation Plan: Attendance Report Month Quick Filters

Adding quick action buttons to select "This Month", "Last Month", and "Custom" date ranges in the Attendance Report page.

## Approach
- **Selection Logic**: Use a state variable `filterMode` to track "this_month", "last_month", or "custom".
- **Helper Functions**: Create functions to calculate the date ranges for the current and previous months.
- **UI Integration**: Add a row of pill-style buttons above the date range picker. The existing date range inputs will only be visible when "Custom" is selected.
- **Default State**: Initial `filterMode` will be "this_month".

## Steps

1. **Create Helper Utilities** (10 min)
   - Function to get `[firstDay, lastDay]` of a specific month relative to now.

2. **Core Implementation** (20 min)
   - File to modify: `webdashboard/src/pages/ReportsPage.tsx`
   - Add `filterMode` state.
   - Update `dateRange` state automatically when `filterMode` changes (using `useEffect` or direct handler).
   - Add the Quick Filter UI component.

3. **Integration** (10 min)
   - Ensure `useQuery` properly reacts to the updated `dateRange`.
   - Wrap existing date inputs in a conditional render based on `filterMode === 'custom'`.

4. **Testing** (20 min)
   - Verify "This Month" selects from day 1 to last day of current month.
   - Verify "Last Month" selects from day 1 to last day of previous month.
   - Verify "Custom" allows manual selection and it triggers data reload.

## UI/UX Design
- **Active State**: The active pill button will have a distinct background (indigo-600) and white text.
- **Iconography**: Use `CalendarDays` or `Clock` for the quick buttons.

## Timeline
| Phase | Duration |
|-------|----------|
| Core Implementation | 20 min |
| UI Refinement | 10 min |
| Testing | 20 min |
| **Total** | **50 min** |

## Rollback Plan
- Revert changes to `ReportsPage.tsx`.

## Security Checklist
- [x] Input validation (date formats)
- [x] Auth checks (inherited from page)
- [x] Error handling (handling empty data for specific months)
