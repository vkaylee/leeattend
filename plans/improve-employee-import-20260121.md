### # Implementation Plan: Improve Employee Bulk Import (Skip Duplicates & UI Icon)

### ## Approach
- **Backend:** Update the employee import service to skip existing employee IDs instead of failing. Use `ON CONFLICT (employee_id_number) DO NOTHING` at the repository level.
- **Backend Feedback:** Update the import job status to report `processed_rows` (only counting actual inserts). 
- **Frontend UI:** Replace the generic `Plus` icon with an `Upload` icon to clearly signal "File Upload". Update the success message to indicate best-effort completion.

### ## Steps

#### 1. Backend: Repository Enhancement
- **File:** `apiserver/src/repositories/employee_repository.rs`
- **Action:** Add `create_employee_tx_skip_duplicates` function that uses `ON CONFLICT (employee_id_number) DO NOTHING` for both `employees` and `employment_periods` tables.
- **Verification:** Since `employee_id_number` is globally UNIQUE in the current schema (checked migrations), this is safe.

#### 2. Backend: Service Logic Update
- **File:** `apiserver/src/services/attendance_import_service.rs`
- **Action:** 
    - In `process_employee_job`, remove the hard validation block that checks for existing IDs and fails the whole job.
    - Update `process_employee_import_row` to use the new skip-duplicates repository function.
    - Ensure `processed_rows` only increments if the DB insert actually happened (check rows affected).

#### 3. Frontend: UI Modernization
- **File:** `webdashboard/src/pages/EmployeesPage.tsx`
- **Action:** Change the icon for the "Import" button from `Plus` to `Upload`.
- **Action:** Change text from `common.add_new` to a more specific `common.import` or `employees.bulk_import_btn`.

#### 4. Frontend: Localization
- **File:** `webdashboard/src/i18n/locales/vi.json` (and en/zh)
- **Action:** Add `"bulk_import_btn": "Nhập từ File"` under `employees`.
- **Action:** Add `"import": "Nhập dữ liệu"` under `common`.

#### 5. Test Fix
- **File:** `apiserver/tests/employee_bulk_import_tests.rs`
- **Action:** Update `test_upload_employee_duplicate_id_fails` -> `test_upload_employee_duplicate_id_skips`.
- **Action:** Assert that the job completes with `Status::Completed` even when duplicates are present.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Backend Repo | 15 min |
| Backend Service | 25 min |
| Frontend UI & Locales | 15 min |
| Testing & Verification| 20 min |
| **Total** | **~1.25 hours** |

### ## Rollback Plan
- Revert repository query to standard `INSERT`.
- Restore hard validation in `attendance_import_service.rs`.

### ## Security Checklist
- [ ] No new fields added to models, so no risk of data exposure.
- [ ] Unique constraint on `employee_id_number` is already enforced by DB.