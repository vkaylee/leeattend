# Implementation Plan: Debug Workspace Tools

## Approach
Implement a "Debug" tab in Workspace Settings to allow developers to quickly clear specific groups of data (Attendance, Schedules, Leave, Payroll, etc.) during development.

- **Backend:** Add a dev-only endpoint `/api/dev/workspaces/:ws_id/clear` wrapped in `#[cfg(debug_assertions)]`.
- **Frontend:** Add a "Debug" tab in the Workspace Settings page, visible only when `import.meta.env.DEV` is true.

## Steps

### 1. Backend Handler Implementation (30 min)
- Modify `apiserver/src/handlers.rs`:
    - Add `ClearWorkspaceDataQuery` struct.
    - Add `clear_workspace_data` handler.
    - Implement `DELETE` logic for groups:
        - `attendance`: `time_entries`, `raw_attendance_logs`, `attendance_adjustments`, `attendance_import_jobs`.
        - `schedules`: `shift_rosters`, `temporal_shift_assignments`.
        - `leave`: `leave_requests`, `leave_balances`.
        - `payroll`: `payslips`, `salary_history`.
        - `full_reset`: All above + `employees`, `shifts`, `departments`, `leave_policies`.
- Modify `apiserver/src/lib.rs`:
    - Register route `POST /api/dev/workspaces/:ws_id/clear` inside `#[cfg(debug_assertions)]` block.

### 2. Frontend API Layer (10 min)
- Modify `webdashboard/src/lib/api.ts`:
    - Add `clearWorkspaceData(wsId: string, group: string): Promise<void>`.

### 3. Frontend UI Component (30 min)
- Create `webdashboard/src/components/workspace/DebugWorkspaceTools.tsx`:
    - UI with buttons for each data group.
    - Loading states and success/error notifications.
    - Confirmation dialog before deletion.

### 4. Integration into Settings (15 min)
- Modify `webdashboard/src/pages/WorkspaceSettings.tsx` (or the relevant settings component):
    - Add "Debug" tab if `import.meta.env.DEV`.
    - Render `DebugWorkspaceTools`.

### 5. Testing & Verification (15 min)
- Verify buttons appear in dev mode.
- Test each button and verify data is deleted in the database.
- Ensure production build does not include these tools.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend Implementation | 30 min |
| Frontend API Layer | 10 min |
| Frontend UI Component | 30 min |
| Integration | 15 min |
| Testing & Verification | 15 min |
| **Total** | **1 hour 40 min** |

## Rollback Plan
- Revert changes in `apiserver/src/handlers.rs`, `apiserver/src/lib.rs`, `webdashboard/src/lib/api.ts`.
- Delete `DebugWorkspaceTools.tsx`.

## Security Checklist
- [x] **Compile-time check:** Backend route must use `#[cfg(debug_assertions)]`.
- [x] **Runtime check:** Frontend tab must use `import.meta.env.DEV`.
- [x] **Workspace Isolation:** Delete queries must always be scoped by `workspace_id`.
- [x] **UX:** Add destructive action warnings.
