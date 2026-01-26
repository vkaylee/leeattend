# Changelog

Tất cả các thay đổi lớn của dự án sẽ được ghi nhận tại đây.

## [Unreleased]
### Added
- **Batch Data Ingestion**: Added `save_raw_attendance_batch` to Edge Agent, increasing write performance by ~100x.
- **SQLite Tuning**: Optimized PRAGMA settings (WAL, NORMAL sync, 64MB cache) for high-frequency logs.
- **Batch Worker Integration**: Implemented independent DB Writer worker with time-based and size-based flush logic.
- **Agent Infrastructure & Sync Engine Rewrite**:
    - **Inverted Communication Model**: Migrated from "Python Server / Rust Client" to a more robust "Rust Server / Python Client" architecture.
    - **Dynamic Port Assignment**: The Orchestrator now binds to a random available port (0) and passes it to the Sidecar, eliminating port conflicts.
    - **Centralized Stateful Ingestion**: Refactored `AttendanceLogService` into a stateful hub that manages its own `mpsc` buffer and background batch-flush worker.
    - **End-to-End Log Syncing**: Implemented a reliable sync loop in `CentralClient` that polls the local SQLite database for unsynced logs and pushes them to the Cloud API with full ACK support.
    - **Robust Supervisor**: The Orchestrator now proactively restarts the Sidecar and waits for a reverse-connection if the process dies.
    - **Integration Verified**: Added comprehensive integration tests verifying the full path: Hardware Logic -> SQLite -> WebSocket -> Cloud API.
### Added
- **Smart Leave Policy Assignment**:
    - **Automatic Conflict Resolution**: Implemented "Delete Future" logic when assigning mid-term policies, ensuring a clean and consistent timeline without manual cleanup.
    - **Assignment History**: Added a detailed history table in Leave Settings to track all policy assignments (Workspace, Department, and Employee scope).
    - **Multi-language Support**: Full translation for Leave Settings in English, Vietnamese, and Chinese.
- **Leave Balance Visibility Improvements**:
    - **Unified View**: Transitioned to `CROSS JOIN` architecture in the Balances page, ensuring all employees are immediately visible for all leave types once a policy is assigned (even with 0 balance).
    - **Robust Fallbacks**: Graceful handling of missing default leave types using CTE-based priority resolution.

### Changed
- **Payroll UX Refactor**:
    - **Search Persistence**: Improved search input behavior in PayrollSummaryTable to prevent focus loss during server-side loading states.
    - **Optimized Layout**: Cleaned up the header and pagination controls for better accessibility and focus.

- **Leave Management System Refactor**:
    - **Architecture**: Migrated from string-based `leave_type` to a normalized `leave_types` reference table with strict Foreign Key constraints.
    - **Frontend**: Added 'Leave Types' management in Settings and unified 'Assign Policy' modals.
    - **Data Integrity**: Backfilled existing data and handled orphans with a robust migration strategy (Jan 24-25, 2026).
- **Overtime Policy System**:
    - Implemented `OvertimeEngine` for flexible, prioritized segment-based calculation.
    - Added comprehensive backend support for `OvertimePolicy`, `OvertimeRule`, and `OvertimePolicyAssignment`.
    - Integrated `AssignOvertimePolicyModal` and `OvertimePolicyManager` in the WebDashboard.
    - Verified with integration tests: `apiserver/tests/overtime_policy_tests.rs`.
- **Official Payslip Document UI**:
    - Created a professional, print-ready "Official Payslip" document with a classic A4-style layout.
    - Integrated a Radix-UI based `PayslipModal` for previewing and printing payslips directly from the browser.
    - Added print-specific CSS optimizations (Tailwind `print:` modifiers).

### Security
- **Strict Multi-Tenancy Enforcement**:
    - Added mandatory `workspace_id` column to `time_entries` table.
    - Implemented `apiserver/tests/schema_isolation_tests.rs` to automatically verify that all database tables (except whitelisted global ones) include `workspace_id` for data isolation.
- **RBAC Audit \& Hardening**:
    - Implemented strict administrative checks (`ctx.is_admin_or_owner()`) across all Overtime Policy handlers.
    - Migrated handlers from `UserId` to `MemberContext` to ensure workspace-level isolation and role verification.

### Fixed
- **Leave Accrual Logic**:
    - Updated seniority bonus logic to "Highest Milestone Reached" instead of additive stacking, preventing double-counting.
    - Fixed `LeaveAccrualService` to correctly handle milestone-based accrual rules.
- **ESS Leave Balances**:
    - Updated `get_my_leave_balances` API to correctly join with the new `leave_types` table, resolving 500 errors.
- **Test Suite Stability**:
    - Fixed `leave_api_tests.rs` to use valid `leave_type_id` UUIDs instead of random ones or legacy strings.
    - Updated `payroll_tests.rs` to use correct local timezone (Asia/Ho_Chi_Minh) and break duration settings, resolving rate calculation discrepancies.
    - Fixed `leave_accrual_performance_tests.rs` and `reprocess_isolation_tests.rs` to include the mandatory `workspace_id` in manual time entry insertions.
- **Employee Search Performance**:
    - Enforced a 50-record limit at the repository level to prevent database strain and client-side lag.
    - Refactored `EmployeesPage` to use debounced server-side searching.
- **Test Stability**:
    - Fixed a critical compilation error in `employee_search_limit_tests.rs` regarding SQLx executor borrowing.
- **Code Quality**:
    - Eliminated redundant `filteredEmployees` variable in `EmployeesPage.tsx`.
    - Added `OvertimePolicy` and `OvertimeRule` TypeScript interfaces to replace `any` types in the payroll module.

### Changed
- **Payroll Transparency**: Added `standard_amount` to `IncentiveEvaluationResult` for better auditability.

### Documentation
- Updated `codebase-summary.md` with details on Leave Management V2, Overtime Engine, and Performance Optimizations.
- Updated `api-reference.md` to reflect the new `workspace_id` field in TimeEntry responses.
