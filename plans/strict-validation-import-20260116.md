# Plan: Implement Strict Validation for Attendance Import

## Goal
Ensure that when importing attendance logs (Excel/CSV), all Employee IDs in the file exist in the system. If any are missing, reject the entire import and report the missing IDs.

## Approach
Modify the `AttendanceImportService` to perform a "Validate First" strategy within the background job.

## Steps

1.  **Scout Employee Repository:**
    - Check `apiserver/src/repositories/employee_repository.rs`.
    - Add `get_existing_employee_ids(pool, workspace_id, employee_ids)` method if it doesn't exist. This should take a `Vec<String>` of IDs and return a `HashSet<String>` or `Vec<String>` of *existing* IDs.

2.  **Refactor `AttendanceImportService::process_job`:**
    - Instead of streaming row-by-row and inserting, read all valid rows into a temporary structure: `struct PendingRow { emp_id: String, punch_time: DateTime<Utc> }`.
    - Collect all unique `emp_id`s from `PendingRows`.
    - Call the new repository method to check existence.
    - Calculate `missing_ids = all_ids - existing_ids`.
    - **Decision Point:**
        - If `missing_ids` is not empty:
            - Abort import.
            - Update `attendance_import_jobs` status to `failed`.
            - Write `error_log`: "Found X unknown employees: [list]".
        - If `missing_ids` is empty:
            - Proceed to loop through `PendingRows`.
            - Insert `RawAttendanceLog`.
            - Trigger `PairingService`.
            - Update progress.

3.  **Refactor `process_excel_range` and CSV logic:**
    - Change these from "process and insert" to "parse and return rows".

## Benefits
- **Data Integrity:** No "ghost" logs for non-existent employees.
- **Clear Feedback:** Admin gets a precise list of who needs to be added.
- **Atomic-ish:** We don't partially import half the file and then fail.
