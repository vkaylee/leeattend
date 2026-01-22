# Implementation Plan: Raw Attendance Logs Data Integrity & Robustness

## Approach
- **Why this solution:** Ensuring the uniqueness of attendance logs prevents duplicate calculations and maintains data integrity. The `ON CONFLICT DO NOTHING` approach allows for idempotent imports, where re-uploading the same data doesn't trigger errors or duplicate records. Skipping non-existent employees with detailed reporting prevents system crashes and provides actionable feedback to administrators.
- **Key Changes:**
  - Database migration to add a unique constraint on `(workspace_id, employee_id, punch_time)`.
  - Update repository to use `ON CONFLICT DO NOTHING`.
  - Refactor `AttendanceImportService` to provide granular feedback instead of failing the entire batch when an employee is missing.

## Steps

1. **Database Migration (10 min)**
   - Create a new migration file: `apiserver/migrations/20260122150000_add_unique_constraint_to_raw_logs.sql`.
   - Add a unique index on `(workspace_id, employee_id, punch_time)`.

2. **Repository Update (10 min)**
   - Modify `apiserver/src/repositories/raw_attendance_log_repository.rs`.
   - Update `create_raw_log` to include `ON CONFLICT (workspace_id, employee_id, punch_time) DO NOTHING`.
   - Return a `Result<bool, sqlx::Error>` where `true` means a row was inserted and `false` means it was skipped.

3. **Import Service Refactoring (30 min)**
   - Modify `apiserver/src/services/attendance_import_service.rs`.
   - Change the "Validation Phase" from "Hard Fail" to "Collection of errors".
   - Collect missing employee IDs and report them in the `error_log` of the `AttendanceImportJob`.
   - Successfully process rows for existing employees even if some are missing.

4. **Testing (20 min)**
   - Create an integration test to verify duplicate handling and missing employee reporting.

## Timeline
| Phase | Duration |
|-------|----------|
| Migration | 10 min |
| Repository | 10 min |
| Service Refactor | 30 min |
| Testing | 20 min |
| **Total** | **~1 hour 10 min** |

## Rollback Plan
- Revert migration using `sqlx migrate revert`.
- Revert code changes in `raw_attendance_log_repository.rs` and `attendance_import_service.rs`.
