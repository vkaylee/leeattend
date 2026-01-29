# Changelog

## [Unreleased]

### Added
- **Overtime Claim & Approval Workflow:**
    - New "System Detect -> Claim -> Approve" process for overtime.
    - `attendance_daily_summaries` table to track daily stats and claim status.
    - **ESS:** Employees can now see "System Detected OT" and submit claims with reasons via the "My Attendance" page.
    - **Manager:** New "OT Approvals" page to review, approve, or reject OT claims.
    - Automated recalculation of daily summaries on time entry changes or log ingestion.

### Changed
- **My Attendance Page:** Added "OT Status" column and "Claim" actions.
- **Backend:** Updated `AttendanceSummaryService` to handle state transitions (PendingEmployee -> PendingManager -> Approved).
- **API:** Added endpoints for claiming and approving OT.

### Fixed
- Fixed escaping issues in dynamic SQL queries for enum types.
