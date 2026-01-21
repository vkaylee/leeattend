# Implementation Plan: Employee Bulk Import Service

### ## Approach
- **Why this solution:** Leverages existing `AttendanceImportService` and `ImportJob` infrastructure to handle bulk data in the background, providing status polling, large file support, and per-row error reporting.
- **Alternatives considered:** Synchronous import. Rejected due to timeout risks for large datasets and poor error feedback.

### ## Steps

1. **Backend: Extend Import Service** (30 min)
   - Modify `apiserver/src/services/attendance_import_service.rs`.
   - Add `process_employee_row` logic to handle employee creation.
   - Update `process_job` to dispatch to employee logic when `target_type == "employee"`.

2. **Backend: API Handler** (15 min)
   - Modify `apiserver/src/handlers/attendance.rs` (central import hub).
   - Add `upload_employee_excel` handler.
   - Register route `POST /workspaces/:ws_id/employees/upload` in `apiserver/src/lib.rs`.

3. **Frontend: API Client** (10 min)
   - Modify `webdashboard/src/lib/api/attendance.ts`.
   - Add `uploadEmployees` function.

4. **Frontend: UI Integration** (30 min)
   - Modify `webdashboard/src/pages/EmployeesPage.tsx`.
   - Add "Import" button and integrate `AttendanceImportDialog`.
   - Refactor `AttendanceImportDialog.tsx` to support generic target types and requirements descriptions.

5. **Testing** (30 min)
   - Create `apiserver/tests/employee_bulk_import_tests.rs`.
   - Verify: Valid import, Duplicate ID handling, Validation errors reporting.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Backend Service | 30 min |
| API Handlers | 15 min |
| Frontend UI | 40 min |
| Testing & Debug | 35 min |
| **Total** | **2 hours** |

### ## Rollback Plan
1. Revert code changes via `git checkout`.
2. Delete new test files.

### ## Security Checklist
- [ ] **RBAC:** Admin/Owner role check on upload endpoint.
- [ ] **Validation:** File type (xlsx/csv) and size (10MB) limits.
- [ ] **Integrity:** Check for duplicate `employee_id` within workspace.
