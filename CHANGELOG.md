# Changelog

## [Unreleased]

### Added
- **Unified Test Infrastructure v3**:
    - **Full Containerization**: All tests now run inside Docker containers, ensuring clean environments and isolation.
    - **Cargo Nextest Integration**: Switched to `nextest` for 2x faster Rust test execution and better process management.
    - **Advanced RAM Disk Optimization**: Automatically moves `.sqlx` metadata and `.test_logs` to RAM, solving WSL2 Disk I/O bottlenecks.
    - **Docker Build Optimization**: Integrated `cargo-chef` for multi-stage builds, caching Rust dependencies separately from source code to reduce build times from minutes to seconds.
    - **Internal Networking**: Removed host port mapping (`5433:5432`) for the test database; all services now communicate via internal Docker network.
    - **Postgres Tuning**: Optimized test database with `fsync=off` and `synchronous_commit=off` for maximum data throughput.
    - **SQLx Offline Pre-compilation**: Improved CI/local workflow by using prepared metadata for host-side builds.

### Fixed
- Fixed host-to-container database connectivity issues on WSL2.
- Resolved race conditions in parallel test execution by utilizing `nextest` process isolation.
- Fixed interactive password prompt hangs in `psql` during setup by implementing `PGPASSWORD` and `trust` authentication.

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
