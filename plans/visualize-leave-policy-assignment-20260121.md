### # Implementation Plan: Visualize Leave Policy Assignment in Balances Page

### ## Approach
- **Why this solution**: Provide immediate visual confirmation of the active policy and its source (Individual, Department, or Workspace-wide) in the Leave Balances table.

### ## Steps

1. **Backend Model & Data Update**
   - Update `WorkspaceLeaveBalance` in `apiserver/src/models.rs`.
   - Update `get_workspace_balances` in `apiserver/src/services/leave_service.rs` to include policy info.

2. **Frontend Type Alignment**
   - Update `WorkspaceLeaveBalance` in `webdashboard/src/lib/api/types.ts`.

3. **Frontend UI Enhancement**
   - Update `webdashboard/src/pages/LeaveBalancesPage.tsx` with a "Policy" column.
