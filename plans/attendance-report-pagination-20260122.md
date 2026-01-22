# Implementation Plan: Paginated & Sortable Attendance Report

Implement pagination and sorting for the Attendance Report (Summary) in both backend and frontend.

## Approach
- **Backend:** Update `get_attendance_report` handler and `ReportService` to support `page`, `page_size`, `sort_by`, and `sort_order`. 
- **Sorting Logic:** Sort the aggregated `AttendanceSummary` list in memory before paginating.
- **Frontend:**
  - Create a reusable `PaginationControls` component.
  - Implement "Auto-page-size" logic: when the report month changes, set `pageSize` to the number of days in that month.
  - Add sorting icons to "Total Work Hours" and "Total Leave Days" columns.
  - Place pagination controls above and below the data table.

## Steps

1. **Backend Infrastructure (10 min)**
   - Add `PaginatedResponse<T>` to `apiserver/src/models.rs`.
   - Define `AttendanceReportSort` enum in `models.rs`.

2. **Backend Service Update (20 min)**
   - Update `ReportService::get_workspace_attendance_summary` in `apiserver/src/services/report_service.rs` to accept `sort`, `page`, and `page_size`.
   - Implement sorting logic (WorkHours, LeaveDays, Name).
   - Implement slicing for pagination.

3. **Backend Handler Update (10 min)**
   - Update `get_attendance_report` in `apiserver/src/handlers/attendance.rs` to parse query parameters.

4. **Frontend API & Types (5 min)**
   - Update `DailyAttendanceReport` and `AttendanceSummary` return types in `webdashboard/src/lib/api/attendance.ts`.
   - Update `fetchAttendanceReport` to pass new params.

5. **Frontend UI Components (20 min)**
   - Create `webdashboard/src/components/common/PaginationControls.tsx`.
   - Features: Jump to page, Previous/Next, Page info.

6. **Frontend Integration (40 min)**
   - Modify `webdashboard/src/pages/ReportsPage.tsx`.
   - Add state for `page`, `pageSize`, `sortBy`, `sortOrder`.
   - Add `useEffect` to sync `pageSize` with the selected month's day count.
   - Update table headers to be clickable for sorting.
   - Insert `PaginationControls` at top and bottom of the table.

7. **Verification (15 min)**
   - Run existing tests.
   - Add E2E test or manual verification for pagination/sorting.

## Timeline
| Phase | Duration |
|-------|----------|
| Backend | 40 min |
| Frontend | 65 min |
| Testing | 15 min |
| **Total** | **2 hours** |

## Rollback Plan
- Use `git checkout` on modified files.
- The changes are backward compatible if default values are provided for new parameters.

## Security Checklist
- [x] Input validation for `page` and `page_size`.
- [x] Whitelist `sort_by` fields.
- [x] Ensure workspace ID isolation is maintained.
