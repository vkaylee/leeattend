# Task 3.4 Shifts, Shift Swaps, Rosters, and Schedules Evidence

Status: Complete

## 1. 14-Route Scope & Registration Parity

All 14 Task 3.4 routes are implemented and registered in `apiserver/main.go` on `net/http.ServeMux`:

| # | Method | Path | Handler | Permission / Guard | Parity Status |
|---|---|---|---|---|---|
| 1 | `GET` | `/api/v1/workspaces/{ws_id}/shifts` | `ShiftHandler.List` | Workspace Member | Verified |
| 2 | `POST` | `/api/v1/workspaces/{ws_id}/shifts` | `ShiftHandler.Create` | `shifts:create` | Verified |
| 3 | `PUT` | `/api/v1/workspaces/{ws_id}/shifts/{shift_id}` | `ShiftHandler.Update` | `shifts:update` | Verified |
| 4 | `GET` | `/api/v1/workspaces/{ws_id}/shifts/{shift_id}/versions` | `ShiftHandler.Versions` | `shifts:read` | Verified |
| 5 | `POST` | `/api/v1/workspaces/{ws_id}/shifts/assign` | `ShiftHandler.Assign` | `shifts:assign` | Verified |
| 6 | `GET` | `/api/v1/workspaces/{ws_id}/shift-swaps` | `ShiftSwapHandler.List` | `shift_swaps:manage` | Verified |
| 7 | `POST` | `/api/v1/workspaces/{ws_id}/shift-swaps` | `ShiftSwapHandler.Create` | `shift_swaps:manage` | Verified |
| 8 | `GET` | `/api/v1/workspaces/{ws_id}/shift-swaps/stats` | `ShiftSwapHandler.Stats` | `shift_swaps:manage` | Verified |
| 9 | `PUT` | `/api/v1/workspaces/{ws_id}/shift-swaps/{id}/approve` | `ShiftSwapHandler.Approve` | `shift_swaps:approve` | Verified |
| 10 | `GET` | `/api/v1/workspaces/{ws_id}/schedules` | `ScheduleHandler.List` | Workspace Member | Verified |
| 11 | `GET` | `/api/v1/workspaces/{ws_id}/employees/{id}/rosters` | `RosterHandler.ListEmployee` | `rosters:read` | Verified |
| 12 | `POST` | `/api/v1/workspaces/{ws_id}/employees/{id}/rosters` | `RosterHandler.Upsert` | `rosters:manage` | Verified |
| 13 | `POST` | `/api/v1/workspaces/{ws_id}/employees/{id}/rosters/bulk` | `RosterHandler.BulkUpsert` | `rosters:manage` | Verified |
| 14 | `DELETE` | `/api/v1/workspaces/{ws_id}/employees/{id}/rosters/{date}` | `RosterHandler.Delete` | `rosters:manage` | Verified |

## 2. Workspace Schedules Implementation (`GET /api/v1/workspaces/{ws_id}/schedules`)

- **DTO & Query Validation (`internal/api/schedules.go`)**:
  - `WorkspaceSchedulesQuery`: `start_date` (required YYYY-MM-DD), `end_date` (required YYYY-MM-DD), optional comma-separated `employment_ids`, `page` (default 1), `limit` (default 50, clamped to 1..3000).
  - Validation returns 400 Validation on missing/malformed date, inverted range (`end_date < start_date`), or malformed employment UUIDs.
  - Member authorization guard checks workspace membership before query execution.
  - Returns `PaginatedEmployeeSchedules` matching Rust's `PaginatedResponse<EmployeeSchedule>`.
- **Effective Resolution Algorithm (`internal/repository/schedules.go`)**:
  - Implements the 5-tier priority hierarchy:
    1. **P1 Roster override (100)**: Direct manual roster date entry (`shift_rosters`).
    2. **P2 Individual working pattern (80)**: Weekly and rotation cycle days (`employee_working_patterns` + `working_pattern_details`).
    3. **P3 Dynamic assignment rules (60)**: Department, employee ID number, tags intersection, location, and employer matching via rule conditions. Highest priority rule wins (`applied`); subsequent matches emit `overridden` trace status.
    4. **P4 Fixed shift assignment (20)**: Individual employee shift assignment (`employee_shifts`).
    5. **P5 None (0)**: Day off.
  - Batch data loading for paged employees eliminates N+1 query patterns: single batched queries for contracts, secondments/assignments, holidays, rosters, patterns, rules, fixed shifts, department history, and shift template intervals.
  - Preserves holiday metadata (`is_holiday`, `is_holiday_paid`, `holiday_name`).

## 3. Atomic Shift Swap Approval & Conflict Detection

- **Conflict Detection**:
  - Rejects self-swaps (`requester_id == target_id`).
  - Validates active employment records for requester and target.
  - Rejects duplicate active swap requests on the same date.
  - Rejects approved swap conflicts for either employee on that date.
  - Rejects swaps when both employees have the same effective shift.
- **Atomic Approval**:
  - Enforces state transition guard: only requests with status `pending_admin` can transition to `approved` or `rejected`.
  - Transactional atomicity: status update, requester roster write, target roster write, and cache invalidation outbox write execute in a single database transaction (`service.WithTx`).
  - Failure in any step rolls back the entire operation.

## 4. Durable Roster Cache Invalidation Outbox

- Typed helper `EnqueueRosterCacheInvalidation` (`internal/repository/outbox.go`) writes `RosterCacheInvalidation` job into `outbox_jobs` in the same transaction as data modifications.
- Integrated into all roster and shift mutation points:
  - `RosterRepository.UpsertRoster`
  - `RosterRepository.BulkUpsertRoster`
  - `RosterRepository.DeleteRoster`
  - `ShiftSwapRepository.ApproveSwap`
  - `ShiftRepository.CreateShift`
  - `ShiftRepository.UpdateShift`
  - `ShiftRepository.AssignShift`
- Guarantees durability across Redis/cache restarts without losing invalidation events.

## 5. Verification & Tests

- `internal/api/schedules_test.go`:
  - `TestScheduleHandler_Unauthenticated`: 401 when user context missing.
  - `TestScheduleHandler_ForbiddenNonMember`: 403 when user is not workspace member.
  - `TestScheduleHandler_ValidationRequiredDates`: 400 validation error for missing/invalid dates and inverted range.
  - `TestScheduleHandler_Success`: 200 OK with paginated schedules.
- `internal/repository/schedules_test.go`:
  - `TestResolveEffectiveShift_RosterOverridesPatternAndFixed`: P1 priority override verification.
  - `TestResolveEffectiveShift_PatternPrecedesRuleAndFixed`: P2 individual pattern priority verification.
- `internal/api/shift_swaps_test.go`:
  - Conflict detection mapping (`ErrShiftSwapConflict` -> 409).
  - Validation error mapping (`ErrShiftSwapValidation` -> 400).
  - Invalid state transition rejection (`pending_target` -> 400).
  - State machine transition guard and approval execution.
- `internal/api/rosters_test.go`:
  - Duplicate date rejection for bulk upserts.
  - 202 Accepted response for valid bulk upserts.
  - Confirmed roster response for single upserts.
