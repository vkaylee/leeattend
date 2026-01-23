# Implementation Plan: Fix Daily Report Employee Filtering

Enable server-side filtering by employee ID for the "Daily Detail" report tab. Currently, the backend ignores the `employee_ids` filter for daily reports, causing performance issues and incorrect UI behavior.

## Approach
- **Pattern:** Propagate `employee_ids` filter down the stack: `Handler` -> `Service` -> `Repository`.
- **Optimization:** Apply the filter at the earliest possible point (Repository) to reduce the number of employees processed in the loop.
- **Backward Compatibility:** `employee_ids` will be an `Option`, ensuring the default behavior (fetching all employees) remains intact if no filter is applied.

## Steps

1. **Repository Layer Update (10 min)**
   - **File:** `apiserver/src/repositories/roster_repository.rs`
   - **Function:** `get_workspace_schedules`
   - **Change:** 
     - Update signature to accept `employee_ids: Option<&Vec<Uuid>>`.
     - Update the initial employee fetch query to filter by IDs using `AND ($2::uuid[] IS NULL OR id = ANY($2))`.

2. **Service Layer Update (10 min)**
   - **File:** `apiserver/src/services/report_service.rs`
   - **Function:** `get_daily_attendance_report`
   - **Change:** 
     - Update signature to accept `employee_ids: Option<Vec<Uuid>>`.
     - Pass this parameter to `roster_repository::get_workspace_schedules`.

3. **Handler Layer Update (5 min)**
   - **File:** `apiserver/src/handlers/attendance.rs`
   - **Function:** `get_attendance_report`
   - **Change:** Pass the parsed `employee_ids` from query params into `ReportService::get_daily_attendance_report`.

4. **Verification (10 min)**
   - **Manual Test:** 
     - Call `/api/workspaces/:id/reports?type=daily&employee_ids=...` and verify response contains only selected employees.
     - Call without `employee_ids` and verify full list.
   - **Unit Test:** Create a simple test case in `report_service_tests.rs` (if exists) or `attendance_robustness_tests.rs` to verify filtering logic.

## Timeline
| Phase | Duration |
|-------|----------|
| Repository Update | 10 min |
| Service Update | 10 min |
| Handler Update | 5 min |
| Verification | 10 min |
| **Total** | **35 min** |

## Rollback Plan
- Revert changes to the 3 files using `git restore`.
- The change is strictly additive (new optional parameter), so risk is low.

## Security Checklist
- [x] **RBAC:** Existing RBAC in handler protects the endpoint.
- [x] **Input Validation:** UUID parsing ensures no SQL injection via `employee_ids`.
- [x] **Data Isolation:** `workspace_id` constraint is maintained in all queries.
