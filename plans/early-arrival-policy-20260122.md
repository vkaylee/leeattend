# Implementation Plan: Early Arrival Policy Support

This plan implements configurable early arrival strategies for workspaces. This allows businesses to either block early check-ins, allow them freely, or allow them but "clamp" the counted overtime to a specific window.

## Approach
- **Database**: Add `early_arrival_strategy` (enum: `allow`, `block`, `clamp`) to `workspaces`.
- **Backend (Models)**: Update `Workspace` and `UpdateWorkspaceRequest` to include the new field.
- **Backend (Calculator)**: Update `AttendanceCalculator` to support `clamp` logic (calculating OT only within the grace window).
- **Backend (Service)**: Update `TimeTrackingService` to only block if strategy is `block`.
- **Frontend**: Add configuration in `WorkspaceSettingsPage`.

## Steps

1. **Database Migration** (5 min)
   - Create migration `apiserver/migrations/20260122100000_add_early_arrival_strategy_to_workspaces.sql`.
   ```sql
   CREATE TYPE early_arrival_strategy_type AS ENUM ('allow', 'block', 'clamp');
   ALTER TABLE workspaces ADD COLUMN early_arrival_strategy early_arrival_strategy_type NOT NULL DEFAULT 'block';
   ```

2. **Backend: Update Models** (10 min)
   - Modify `apiserver/src/models.rs`:
     - Add `EarlyArrivalStrategy` enum.
     - Update `Workspace` and `UpdateWorkspaceRequest`.
   ```rust
   #[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, sqlx::Type, strum_macros::Display)]
   #[sqlx(type_name = "early_arrival_strategy_type", rename_all = "snake_case")]
   #[serde(rename_all = "snake_case")]
   pub enum EarlyArrivalStrategy {
       Allow,
       Block,
       Clamp,
   }
   ```

3. **Backend: Update Repositories** (5 min)
   - Update `apiserver/src/repositories/workspace_repository.rs` to handle the new field in SQL queries (fetch and update).

4. **Backend: Update Attendance Calculator** (15 min)
   - Modify `apiserver/src/services/attendance_calculator.rs`:
     - Update `calculate_buckets` to accept `EarlyArrivalStrategy`.
     - Implement `clamp` logic for `effective_ot_start`.
     - Update tests.

5. **Backend: Update Time Tracking Service** (10 min)
   - Modify `apiserver/src/services/time_tracking_service.rs`:
     - Read `early_arrival_strategy` from workspace.
     - Only return `BAD_REQUEST` if strategy is `Block`.

6. **Frontend: Update Settings UI** (15 min)
   - Modify `webdashboard/src/pages/WorkspaceSettingsPage.tsx`:
     - Add select field for "Quy tắc đến sớm" (Early Arrival Policy).
     - Options: "Chặn chấm công" (Block), "Cho phép" (Allow), "Cắt gọt OT" (Clamp).

7. **Integration & Testing** (15 min)
   - Create a new integration test `apiserver/tests/early_arrival_policy_tests.rs` to verify the different strategies.

## Timeline
| Phase | Duration |
|-------|----------|
| Database & Models | 15 min |
| Calculator Logic | 15 min |
| Service Logic | 10 min |
| Frontend UI | 15 min |
| Testing | 15 min |
| **Total** | **1 hour 10 min** |

## Rollback Plan
- Revert code changes.
- Run `sqlx migrate revert`.

## Security Checklist
- [x] Input validation (Enum validation)
- [x] Auth checks (Workspace admin only - handled by existing middleware)
- [x] Error handling
