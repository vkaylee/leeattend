# Changelog

Tất cả các thay đổi lớn của dự án sẽ được ghi nhận tại đây.
## [Unreleased]
### Added
- **Official Payslip Document UI**:
    - Created a professional, print-ready "Official Payslip" document with a classic A4-style layout.
    - Integrated a Radix-UI based `PayslipModal` for previewing and printing payslips directly from the browser.
    - Added print-specific CSS optimizations (Tailwind `print:` modifiers) to ensure high-quality PDF/physical output.
- **Unified Payslip Line Items (Backend & Frontend)**:
    - **Backend**: Implemented `SalaryComponentWithDefinition` to include full metadata (name, code, tax status) in the payslip API response.
    - **Frontend**: Updated `PayslipDocument` to provide a granular breakdown of earnings based on actual salary components assigned to the employee.
    - **Prorated Calculation**: Implemented logic to calculate and display prorated amounts for salary components based on standard working days and actual paid days in the period.
- **Strict Boolean Payroll Calculation**: Removed "magic string" checks (`BASE`, `Lương cơ bản`) in `PayrollService`. The engine now strictly relies on the `is_base_for_ot` flag from the database schema for identifying base salary components.
- **Payroll Data Consistency Migration**: Added migration `20260123000000_ensure_base_salary_flags.sql` to backfill the `is_base_for_ot` flag for legacy salary components, ensuring backward compatibility with the new strict logic.
- **Smart Payroll UI Pagination**: Fixed an issue where pagination state would reset incorrectly during navigation. Improved table stability and user feedback during large dataset browsing.
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
- **Payslip Rendering Stability**:
    - Implemented defensive coding in `PayslipDocument` to prevent crashes (blank screen) when encountering malformed or partial salary component data.
    - Fixed a `TypeError` where numeric calculations on string-serialized decimals (from Rust) caused "toFixed is not a function" errors.
    - Added unit and integration tests to verify robust error handling for payslip rendering.
- **Payroll Transparency**: Added `standard_amount` to `IncentiveEvaluationResult` to track the reference value (e.g., Policy Base) alongside the actual payout, enabling "Standard vs Actual" comparison in Payslips.
- **SQL Migration Syntax**: Fixed a syntax error in migration `20260123000000` where incorrectly escaped characters caused PostgreSQL failures.
- **Chinese Locale Synchronization**: Added missing `payroll.selection` keys to `zh.json` to ensure consistency across all supported languages.
- **Outdated Integration Tests**: Updated `schedule_traceability_tests.rs` and `work_unit_integration_tests.rs` to match the latest `ReportService` and `RosterRepository` function signatures.
- **Daily Report Filtering**: Resolved an issue where filtering by Department or Employee ID returned empty results due to incorrect JOIN logic in the reporting service.
- **Pagination State Management**: Fixed a bug in `webdashboard` where changing the page size on the Payroll Summary table did not correctly refresh the data set.
### Changed
- **Payroll Summary Polish**: Improved alignment and overflow handling in the Payroll Summary table for better responsiveness.
- **Variable-based Payroll Incentive Engine (Diligence Bonus)**:
    - **Incentive Context**: During payroll calculation, the engine aggregates a high-fidelity context including `total_late_early_count`, `unpaid_leave_days`, and `sick_leave_days`.
    - **Policy Matching**: The `PayrollEvaluator` matches this context against a prioritized list of `IncentiveTiers`.
    - **Generosity Principle**: Tiers are sorted by priority. The engine selects the first matching tier (e.g., Tier 1 for 0 violations, Tier 2 for < 2 violations), rewarding the most diligent behavior.
