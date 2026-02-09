# Changelog

## [Unreleased]

### Added
- **Production Environment Setup**:
    - **Dockerized Deployment**: Implemented a comprehensive `docker-compose.yml` for production, orchestrating API Server, Frontend (Nginx), PostgreSQL, and Redis.
    - **Multi-Stage Dockerfiles**: Created optimized, security-focused Dockerfiles for `apiserver` and `webdashboard`.
    - **Automated Reverse Proxy**: Configured Nginx within the frontend container to handle SPA routing and API/WebSocket proxying.
    - **Production Config Template**: Added `.env.example.prod` for standardized environment management.
- **Testing Infrastructure Enhancements**:
    - **Smart Worker Pool**: Implemented a real worker pool in `run_parallel_ordered` to strictly respect `NUM_WORKERS` limit.
    - **Parallel Execution Capping**: Added hard limits (1-2 workers) to prevent WSL Disk I/O overload and system freezes.
    - **Comprehensive Summary Report**: Added a colorized summary table at the end of test runs showing Task Name, Status (PASSED/FAILED/TIMEOUT), and Duration.
    - **Persistent Results Tracking**: Implemented file-based results tracking (`.test_logs/results.txt`) to preserve task metadata across subshells and phases.
    - **Real-time Progress Indicator**: Added active task tracking with `[WAIT] Running: ...` status updates for better visibility during parallel execution.
- **Webdashboard Testing Stabilization**:
    - **Mock API Architecture**: Implemented full backend mocking using Playwright Intercept (`page.route`). Frontend tests are now 100% decoupled from Rust backend and database.
    - **Strict 1-Worker Policy**: Playwright is now forced to run with a single worker in Docker to eliminate resource contention and flakiness.
    - **Multi-language Support**: Standardized Unit and E2E tests to support both English and Vietnamese locales using regex matchers.
    - **Stable Timezone Logic**: Standardized date formatting to `en-GB` with `h23` to prevent "24:00" discrepancies across environments.
    - **Extended Timeouts**: Increased global timeouts to 90s for E2E and 60s for Unit Tests to accommodate Docker overhead.
- **Unified Test Infrastructure v3**:
    - **Full Containerization**: All tests now run inside Docker containers, ensuring clean environments and isolation.
    - **Cargo Nextest Integration**: Switched to `nextest` for 2x faster Rust test execution and better process management.
    - **Advanced RAM Disk Optimization**: Automatically moves `.sqlx` metadata and `.test_logs` to RAM, solving WSL2 Disk I/O bottlenecks.
    - **Docker Build Optimization**: Integrated `cargo-chef` for multi-stage builds, caching Rust dependencies separately from source code to reduce build times from minutes to seconds.
    - **Internal Networking**: Removed host port mapping (`5433:5432`) for the test database; all services now communicate via internal Docker network.
    - **Postgres Tuning**: Optimized test database with `fsync=off` and `synchronous_commit=off` for maximum data throughput.
    - **SQLx Offline Pre-compilation**: Improved CI/local workflow by using prepared metadata for host-side builds.
- **Unit Test Coverage**: Successfully reached **447/447 passing tests** (100%) for webdashboard logic.

### Fixed
- Fixed "window is not defined" error in Toast components by implementing proper cleanup logic.
- Resolved "ReferenceError" during settings page load by stabilizing hook execution order.
- Fixed i18n inconsistencies in unit tests by utilizing real i18next configuration instead of simple mocks.
- Fixed "Multiple elements found" error in MembersPage tests by refining locator specificity.
- Fixed net::ERR_ABORTED errors in Playwright by disabling Vite proxy during test execution.
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
