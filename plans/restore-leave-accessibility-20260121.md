### # Implementation Plan: Restore Leave Management Accessibility & Functionality

### ## Approach
- **Why this solution**: Re-establishing the Sidebar link directly addresses the "lost" page complaint (since Policies are a tab within Leave Requests). Adding the manual accrual trigger in the Balances page unlocks the core logic that updates employee entitlements, which is currently "stuck" due to lack of a UI entry point.

### ## Steps

1. **Sidebar Navigation Update**
   - Modify `webdashboard/src/components/layout/Sidebar.tsx`.
   - Add `leave-requests` link to the `time` section.

2. **Accrual Trigger Integration**
   - Modify `webdashboard/src/pages/LeaveBalancesPage.tsx`.
   - Add "Run Accrual" button calling `triggerAccrualRun`.

3. **Verification**
   - Test Sidebar navigation.
   - Test Accrual trigger functionality.
