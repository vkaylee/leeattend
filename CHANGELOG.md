# Changelog

Tất cả các thay đổi lớn của dự án sẽ được ghi nhận tại đây.

## [Unreleased]

### Added
- **Graceful Shutdown Handling**:
    - **`agent-main` (Rust)**: Implemented `tokio_util::sync::CancellationToken` to orchestrate clean shutdowns of supervisor loops, WebSocket listeners, and child processes upon receiving `SIGINT` or `SIGTERM`.
    - **`agent-zk-worker` (Python)**: Implemented native signal handlers using `asyncio.Event` to ensure the JSON-RPC server closes connections and exits gracefully.
    - **Sidecar Lifecycle Management**: The Rust orchestrator now explicitly kills and waits for the Python sidecar process during its own shutdown sequence.
- **Unified Test Orchestrator (`run-tests.sh`)**:
    - **Selective Execution**: Added flags (`--all`, `--api`, `--web`) to target specific components, significantly reducing iteration time.
    - **Pattern Matching**: Implemented support for an optional pattern argument to filter test cases across different runners (Cargo, Pytest, Playwright, Vitest).
    - **Automated Environment**: The orchestrator now automatically manages the full test lifecycle, including Postgres Docker setup, isolated database creation, and migrations.
    - **Resilient Filtering**: Improved internal shell scripts to handle complex regex patterns and gracefully manage "no tests found" scenarios (e.g., Pytest exit code 5).
- **Unified Attendance Log Processing**:
    - **`AttendanceLogService`**: Created a central service to handle log ingestion from all sources (Agent WebSocket and Manual Import).
    - **Business Logic Parity**: Ensures employee mapping, timezone-aware parsing, and automatic pairing (FSM) are applied consistently.
    - **Timezone-Aware Agent Logs**: The WebSocket handler now fetches workspace configuration during handshake to correctly parse "Naive" device timestamps into UTC.
    - **Robust Testing**: Added `attendance_log_service_tests.rs` covering timezone conversion, duplicate prevention, and error handling.
- **Robust Agent Configuration System**:
    - **Smart Fallbacks**: Implemented a centralized `Config` struct in `agent-main` with intelligent fallbacks for `AGENT_ID`, `DATABASE_URL`, and `CENTRAL_SERVER_URL`.
    - **Dynamic Path Resolution**: Added automatic detection for Python virtual environments (`venv` vs `venv_linux`), resolving path issues across different Linux distributions.
    - **Environment Template**: Created `.env.template` for the agent to guide manual configuration and ensure standard defaults.
    - **Console Awareness**: Added a colorful startup summary and explicit warnings when fallback values are being used instead of environment variables.
    - **Configurable RPC Timeout**: Added `RPC_TIMEOUT_SECONDS` to the configuration system (default 30s) to prevent `TimedOut` errors during large attendance log fetches from ZKTeco devices.
- **Partial Attendance Import Success**:
    - **Robust Batch Processing**: Enhanced `AttendanceImportService` to support "Partial Success," allowing valid records to be imported even if the batch contains duplicate logs or non-existent employee IDs.
    - **Database Level Protection**: Added a unique constraint to `raw_attendance_logs` to prevent duplicate punches at the database level.
    - **Comprehensive Testing**: Added a dedicated integration test (`attendance_partial_import.rs`) to verify the "Partial Success" behavior and error logging.

### Fixed
- **Daily Report Filtering**: Resolved an issue where filtering by Department or Employee ID returned empty results due to incorrect JOIN logic in the reporting service.
- **Pagination State Management**: Fixed a bug in `webdashboard` where changing the page size on the Payroll Summary table did not correctly refresh the data set.

### Changed
- **Payroll Summary Polish**: Improved alignment and overflow handling in the Payroll Summary table for better responsiveness.

### Added
- **Variable-based Payroll Incentive Engine (Diligence Bonus)**:
    - **Incentive Context**: During payroll calculation, the engine aggregates a high-fidelity context including `total_late_early_count`, `unpaid_leave_days`, and `sick_leave_days`.
    - **Policy Matching**: The `PayrollEvaluator` matches this context against a prioritized list of `IncentiveTiers`.
    - **Generosity Principle**: Tiers are sorted by priority. The engine selects the first matching tier (e.g., Tier 1 for 0 violations, Tier 2 for < 2 violations), rewarding the most diligent behavior.
