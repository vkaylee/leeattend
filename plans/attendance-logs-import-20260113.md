# Implementation Plan: Attendance Logs Import

### ## Approach
Implement a dedicated import flow for raw attendance logs (TimeEntries) by refactoring the existing import logic into a centralized service. This allows reusing the background job infrastructure (tracking, error logging, progress) while specializing the actual data insertion.

- **Backend**: Create `AttendanceImportService` to handle parsing and dispatching to either `adjustment` or `log` targets.
- **Backend**: Update `attendance_import_jobs` table to include `target_type` ("adjustment" | "log").
- **Frontend**: Extract common `ImportDialog` and integrate it into `AttendanceLogsPage`.

### ## Steps

1. **Database Migration** (5 min)
   - Add `target_type` (VARCHAR, default 'adjustment') to `attendance_import_jobs` table.
   - Run migration.

2. **Core Service Refactoring** (30 min)
   - Create `apiserver/src/services/attendance_import_service.rs`.
   - Move parsing logic (CSV/Excel) from `AttendanceAdjustmentService` to `AttendanceImportService`.
   - Implement `process_log_row` to create `TimeEntry` directly.

3. **Backend API Update** (15 min)
   - Add `POST /workspaces/:ws_id/attendance/logs/upload` handler in `handlers.rs`.
   - Update `AttendanceAdjustmentService::batch_import_from_file` to support the new target.

4. **Frontend Component Refactoring** (20 min)
   - Create `webdashboard/src/components/AttendanceImportDialog.tsx` by extracting logic from `AttendanceAdjustmentsPage.tsx`.
   - Ensure the dialog accepts an `endpoint` prop to support both log and adjustment uploads.

5. **Integration** (10 min)
   - Add "Import" button to `AttendanceLogsPage.tsx`.
   - Update `AttendanceAdjustmentsPage.tsx` to use the new shared dialog.

6. **Testing** (15 min)
   - Verify log import via a new integration test or manual verification.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Database | 5 min |
| Service Refactor | 30 min |
| API Layer | 15 min |
| Frontend Components| 20 min |
| Integration | 10 min |
| Testing | 15 min |
| **Total** | **~2 hours** |

### ## Rollback Plan
- Revert database migration.
- Restore `AttendanceAdjustmentService` from git history.
- Revert UI changes in dashboard pages.

### ## Security Checklist
- [x] Input validation (file extension check).
- [x] Auth checks (MemberContext verifies workspace access).
- [x] RBAC: Only `admin` or `owner` can upload.
- [x] Error handling (Job tracks failures in `error_log`).
