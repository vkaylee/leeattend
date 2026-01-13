# Implementation Plan: Raw Attendance Logs & Pairing Logic

## Goal
Separate raw attendance events from processed sessions to improve auditability and data integrity.

## Subtasks
1. [ ] Create Database Migration for `raw_attendance_logs`.
2. [ ] Add `RawAttendanceLog` model and repository.
3. [ ] Implement `PairingService` with deduplication and session matching logic.
4. [ ] Update `AttendanceImportService` to use the new flow.
5. [ ] Add API handlers for Raw Logs.
6. [ ] Update Frontend `AttendanceLogsPage` and API client.

## Files to Modify/Create
- `apiserver/migrations/20260114000000_create_raw_attendance_logs.sql` (New)
- `apiserver/src/models.rs`
- `apiserver/src/repositories/raw_attendance_log_repository.rs` (New)
- `apiserver/src/services/pairing_service.rs` (New)
- `apiserver/src/services/mod.rs` (If it exists, or register in `lib.rs`)
- `apiserver/src/services/attendance_import_service.rs`
- `apiserver/src/handlers.rs`
- `webdashboard/src/pages/AttendanceLogsPage.tsx`
- `webdashboard/src/lib/api.ts`

## Success Criteria
- [ ] Importing a file creates entries in `raw_attendance_logs`.
- [ ] `PairingService` creates/updates entries in `time_entries` based on raw logs.
- [ ] UI displays flat events list from `raw_attendance_logs`.
- [ ] All tests pass.
