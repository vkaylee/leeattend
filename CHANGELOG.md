# Changelog

Tất cả các thay đổi lớn của dự án sẽ được ghi nhận tại đây.

## [Unreleased]

### Added
- **Variable-based Payroll Incentive Engine (Diligence Bonus)**:
    - **Tiered Evaluation System**: Implemented a flexible, priority-based evaluation engine in `PayrollEvaluator` to match attendance summaries against tiered thresholds.
    - **Incentive Context**: Enhanced monthly payroll calculation to track detailed metrics including `total_late_early_count`, `unpaid_leave_days`, and `sick_leave_days`.
    - **Policy Management UI**: Added `BonusPolicyManager` to Settings for configuring multi-threshold rules (e.g., "Full bonus if < 2 lates and 0 unpaid leave days").
    - **Payslip Integration**: Incentives are now automatically computed, stored in `payslip_incentives`, and visualized in both the Admin Payroll table and the ESS portal.
- **Employee ID Normalization**:
    - **Automatic Cleanup**: Implemented `normalize_employee_id` to automatically trim whitespace and remove leading zeros from purely numeric IDs (e.g., "007" -> "7").
    - **Data Consistency**: Enforced normalization across employee creation, bulk imports (XLSX), and updates to prevent duplicate records and linking errors.
- **Special Attendance Policies (Parental Support & privileges)**:
    - **Dynamic Privileges**: Implemented a "Working Condition Overlay" system to handle individual employee attendance privileges (e.g., Early Leave for Childcare, Breastfeeding Breaks, Medical Recovery).
    - **Policy Resolution**: Enhanced `PolicyResolver` to dynamically fetch and merge employee-specific privileges based on date validity.
    - **Refactored Calculation Core**: Standardized `AttendanceCalculator` to use a `PolicyRules` context, improving code quality and enabling robust support for `authorized_early_leave_minutes` and `is_early_leave_paid`.
    - **Management UI**: Added a dedicated "Policies" tab to the Employee Detail view for assigning and tracking special attendance regimes.
    - **API Integration**: New endpoints for CRUD operations on employee working conditions (`GET/POST/DELETE .../employees/:id/conditions`).
    - **Full Localization**: Added comprehensive translations for policies and privileges in English, Vietnamese, and Chinese.
- **Leave Balance Pagination**:
    - **Performance**: Implemented server-side pagination for `GET /workspaces/:ws_id/leave/balances` to handle large employee datasets.
    - **Backend**: Updated repository and service layers to support `limit`, `offset`, and `q` (search) parameters.
    - **Frontend**: Refactored `LeaveBalancesPage` to use server-side state management with `PaginationControls` and debounced search.
    - **API**: Introduced `PaginatedResponse<T>` standard for leave endpoints.
- **Early Arrival Policy**:
    - **Feature**: Implemented configurable `early_arrival_strategy` (Block/Allow/Clamp) to control check-in behavior before shift start.
    - **UI**: Added Grace Period configuration to Shift settings and Policy selector to Workspace Time settings.
    - **Logic**: Updated `AttendanceCalculator` to support clamping overtime to the grace window.
- **Debug Toolbar Improvements**:
    - **Backend**: Enhanced `clear_debug_db` handler with new capabilities:
        - `employees`: Wipes all employee data (logs, payroll, history) while preserving workspace configuration (Departments, Shifts, Policies).
        - `seed`: Automatically generates a demo environment with 2 Departments, 1 Shift, and 5 Employees.
    - **Backend**: Hardened `full_reset` (Factory Reset) to ensure comprehensive cleanup of all workspace artifacts, including assignment rules and working patterns.
    - **Frontend**: Redesigned `DebugToolbar` with a structured layout:
        - **Data Operations**: Granular wipe for Attendance, Payroll, and Rosters.
        - **Destructive Zone**: Explicit "Wipe Staff" and "Factory Reset" actions with safety confirmations.
        - **Seeding**: One-click "Quick Seed Demo Team" for rapid testing setup.
- **Smart Pagination & Sorting for Attendance Reports**:
    - **Dual Pagination**: Implemented top and bottom pagination controls for the Attendance Report table.
    - **Smart Page Size**: The default number of rows per page now automatically adjusts to the number of days in the selected month (e.g., 28, 30, or 31).
    - **Dynamic Sorting**: Added server-side sorting by Employee Name, Total Work Hours, and Total Leave Days (Ascending/Descending).
    - **Server-side Filtering**: Integrated employee ID filtering into the backend reporting engine for better performance on large datasets.
    - **Interactive UI**: Created a reusable `PaginationControls` component with quick-jump to page and adjustable page size selector.
    - **Backend Architecture**: Added a generic `PaginatedResponse<T>` and `AttendanceReportSort` models to support consistent paginated APIs.
    - **SQLX Resilience**: Migrated report queries to `sqlx::query` to avoid offline cache synchronization issues during rapid iteration.
- **Employee Bulk Import Improvements**:
    - **"Silent Skip" Logic**: Enhanced import backend to automatically skip duplicate employee IDs (using `ON CONFLICT DO NOTHING`) instead of failing the entire batch.
    - **Progress Feedback**: Import job status now correctly distinguishes between inserted rows and skipped rows.
    - **UI Enhancements**: Replaced generic "Add New" button with a dedicated "Import from File" button (Upload icon) on the Employees page.
    - **Localization**: Added specific translation keys for import actions across EN, VI, ZH.
- **Frontend Routing Architecture**:
    - Decoupled routing logic into a standalone `AppRoutes.tsx` component to support isolated integration testing.
    - Added comprehensive **Page Rendering Tests** (`PageRendering.test.tsx`) to verify core pages (Dashboard, Employees, Leave Balances) render without crashing.
- **Attendance Pairing Strategy Enhancement**:
    - Backend: Extended `AnchorFirstStrategy` limit from 12 to 16 hours to support longer employee shifts.
    - Backend: Updated pairing strategy unit tests to verify the 16-hour window and edge cases.
- **ESS Phase 1 Completion**:
    - Frontend: Implemented **PunchCard** component for clock-in/out with real-time status and error handling.
    - Frontend: Added **TimeEntryRow** and **AdjustmentRequestModal** for employee self-service time tracking.
    - Frontend: Added direct access to ESS Clock page in the main Sidebar.
    - Frontend: Modernized `MyAttendancePage` and `MyProfilePage` with premium UI aesthetics.
- **Global UI Modernization (Premium Pass)**:
    - Frontend: Modernized all administrative pages (Departments, Shifts, Working Patterns, Reports, Agents, Devices) with premium card layouts and glassmorphism elements.
    - Frontend: Refined common UI components (`Modal`, `Sheet`, `Combobox`, `Navbar`) for consistent "Metro-style" aesthetic.
    - Frontend: Modernized Payroll interface, including `PayrollTable`, `AdjustmentDrawer`, and `BatchCalculationModal`.
    - Frontend: Improved I18n labels for workspace settings and theme configuration across all supported languages (EN, VI, ZH).
- **Test Suite Enhancements**:
    - Frontend: Added `MembersPage.test.tsx` for comprehensive member management testing.
    - Frontend: Implemented `use-debounce` mock to improve Vitest execution stability.
    - Frontend: Updated existing integration tests to match the new modernized UI structures.

### Changed
- **Legacy Code Removal**:
    - Removed `TimeTrackingPage.tsx` and legacy `/time-tracking` route, fully replacing it with the new `AttendanceLogsPage` and ESS portal.
    - Updated Dashboard navigation to point to `/attendance-logs` instead of the deprecated time tracking page.
    - Cleaned up redundant navigation links in `Sidebar.tsx` and `Navbar.tsx`.
- **Codebase Maintenance**:
    - Documentation: Added comprehensive test audit reports and Phase 1 implementation summaries.
    - Configuration: Updated `.gitignore` to exclude large generated artifacts.
    - Cleaned up accidental temporary directories from the workspace.

### Fixed
- **E2E Test Stabilization**:
    - **Language-Agnostic Locators**: Implemented `data-testid` across all critical pages (Employees, Shifts, Attendance Logs, Settings) to ensure tests pass regardless of UI language (VI/EN/ZH).
    - **Navigation Resilience**: Refactored `route-regression.spec.ts` to use direct navigation (`page.goto`), bypassing flakiness from sidebar animations and layout shifts.
    - **Integration Test Reliability**: Refactored `EmployeeSearchIntegration.test.ts` to use stable test IDs, ensuring consistent verification of search results.
    - **CI Performance**: Optimized `worker-orchestrator.ts` and `playwright.config.ts` to support isolated storage states and parallel execution with reduced system load.
- **Postgres Migration Syntax**: Resolved syntax errors in integration tests by standardizing on Postgres-compatible SQL (e.g., using `TIMESTAMPTZ` and `TRUE`/`FALSE`).
- **Employee Name Refactor**: Updated API tests to use `full_name` instead of deprecated `first_name` and `last_name` fields, aligning test expectations with the current data model.
- **Bulk Import Reliability**: Fixed a formatting bug in the multipart body construction for integration tests, ensuring consistent boundary handling for file uploads.
- **Report Pagination Tests**: Updated integration tests to correctly handle the new `PaginatedResponse` structure returned by the attendance report API.
- **Leave Balance Re-calculation**: Resolved an issue where manual "Update Balance" triggers would not reflect policy changes for past dates. The system now forces a re-calculation and updates balances if the accrual amount differs.
- **TypeScript & Linting**: Fixed multiple type errors in `PayrollTable`, `PunchCard`, and `Sidebar` components to ensure a clean build.
- **Test Stability**: Fixed flaky E2E tests by adjusting timeouts and ensuring proper environment teardown.

## [2026.01.21] - Previous Release Notes... (omitted)
    - Added comprehensive integration tests for the batch calculation engine.
- **Payroll Standard Days Traceability**:
    - Backend: Enhanced `PolicyResolver` and `PayrollService` to track and persist the source of standard working days (Individual > Department > Workspace).
    - Backend: Added `std_days_source` column to `payslip_segments` for granular auditability.
    - Frontend: Implemented `PayrollPriorityLegend` to visualize the 3-tier payroll hierarchy in the `PayrollTable` segment details.
    - Added comprehensive integration tests for payroll traceability (`payroll_traceability_tests.rs`).
- **Schedule Traceability System**:
    - Backend: Enhanced `RuleEvaluator` to track rule outcomes and decision logic for every shift assignment.
    - Frontend: Implemented `PriorityLegend` and `ScheduleTracePopover` to visualize why a specific shift was assigned (e.g., Roster Override vs. Pattern).
    - Added smart intervention hints for scheduling conflicts and rule violations.
- **Employee Self-Service (ESS) Phase 3**:
    - **My Attendance Page**: New dedicated view for employees to track their own attendance history and pairing results.
    - **Traceability Visualization**: Integrated traceability metadata into the ESS portal, allowing employees to see how their work units were calculated.
- **Settings Hub Consolidation**:
    - Centralized navigation for all workspace configurations under `/workspace/settings`.
    - Merged Departments, Members, and Role management into a unified "People" settings module.
    - Standardized sub-pages for General, Time, Payroll, and Connectivity settings.
- **FSM Attendance Pairing Strategy**:
    - Re-implemented and optimized the **Finite State Machine (FSM)** pairing algorithm to handle complex punch sequences with explicit status intent.
    - Added BSON-based `generation_metadata` to time entries for deep auditability of pairing decisions.
- **Robust i18n Testing Suite**:
    - Enhanced `translation-usage.test.ts` with advanced detection for missing keys, untranslated keys (value equals key path), and hardcoded JSX strings.
    - Implemented stricter validation for locale file synchronization to ensure 100% key parity across `en.json`, `vi.json`, and `zh.json`.
- **Payroll UI Localization**:
    - Added comprehensive translations for payroll segments (Period, Basis, Std Days, Actual, Amount) in all supported languages.
    - Localized table headers and status labels in the `PayrollTable` component.

### Changed
- **UI/UX Refinement**:
    - Created a generic `HierarchyLegend` component to provide a unified "Vertical Metro Map" visual language for all hierarchical decision systems (Shifts, Payroll).
    - Refactored the Shift `PriorityLegend` to consume the new generic component, maintaining consistency while reducing code duplication.
    - Updated `PriorityLegend` to use a **Vertical Metro Map** style, providing a clearer visualization of the priority hierarchy and fallback logic.
    - Standardized `PriorityLegend` colors and labels across English and Vietnamese locales.
- **Refactored Settings Hub i18n**:
    - Standardized translation keys for Settings categories (General, People, Time, Payroll, Connectivity) to resolve "Missing Key" warnings.
    - Updated `App.tsx` and `PeopleSettings.tsx` to use leaf-node translation keys instead of object paths for better reliability.
- **Improved E2E Test Stability**:
    - Increased Playwright timeouts to 120s in `playwright.config.ts` and individual specs to handle slow CI/CD environments.
    - Updated E2E locators in `members-ui`, `sidebar-navigation`, and `stability-regression` to match recent UI refactors (Settings Hub).
    - Fixed shared-state interference in `change-password.spec.ts` by adding a password reset teardown.

### Fixed
- **UI Navigation**: Resolved a blank page issue at `/workspace/leave/balances` by adding the missing route definition in `AppRoutes.tsx`.
- **Locale Desynchronization**: Resolved numerous missing translation keys in `vi.json` and `zh.json` identified by the new testing suite.
- **Settings Page Rendering**: Refactored `GeneralSettings.tsx` and `TimeSettings.tsx` into container/presentational components to fix ESLint cascading render warnings.
- **App Layout**: Fixed broken imports for `AssignmentRulesPage` and `WorkspaceSettingsPage`.
- **JSON Quality**: Fixed invalid JSON syntax (trailing commas) and synchronized key structures across all locale files.
- **Attendance Pairing Engine**:
    - Resolved "Log Stealing" bug where 36-hour search windows caused overlaps between adjacent days.
    - Implemented a **16-Hour Gap Limit** (960 minutes) across pairing strategies to prevent aggressive pairing of punches across multi-day gaps.
    - Added "Missing Punch Out (Gap)" status when the duration limit is exceeded to improve audit clarity.

## [0.2.1] - 2026-01-18

### Added
- **Attendance Pairing Traceability**:
    - Added `generation_metadata` (JSONB) to `time_entries` table to track the pairing algorithm and source logs.
    - Implemented a "Debug Mode" in Attendance Reports to visualize pairing metadata and BSON source data directly in the UI.
- **Manual Attendance Reprocessing**:
    - New API endpoint `POST /api/workspaces/:ws_id/attendance/reprocess/:employee_id` to trigger manual pairing recalculation.
- **Hierarchical 3-tier Temporal Payroll**:
    - New schema for versioned payroll settings at Individual, Department, and Workspace levels.
    - `PolicyResolver` domain service for point-in-time rule resolution.
    - Multi-segment payroll calculation with proration support for mid-month changes.

### Changed
- Increased E2E test stability by raising DB pool `acquire_timeout` to 30s and Playwright navigation timeouts to 60s.
- Optimized sidebar navigation in E2E tests to handle pre-expanded sections.
- Updated `PayrollService` to use the new hierarchical resolver instead of legacy fixed fields.

## [0.2.0] - 2026-01-15

### Added
- **Dynamic Payroll & Attendance System V2**: Implemented a robust "Multi-Segment Calculation" engine to handle complex payroll scenarios.
    - **Prorated Salary Calculation**: The engine now slices the payroll period into segments based on salary or department changes (Change Points), calculating each segment with its own configuration (e.g., standard days).
    - **Working Conditions**: Introduced a new `working_conditions` table to handle authorized adjustments like "Early Leave for Childcare".
    - **Flexible Attendance Policy**: `AttendanceCalculator` now supports `authorized_early_leave_minutes` and `is_early_leave_paid` parameters, allowing employees to leave early without penalty and optionally get paid.
    - **Granular Snapshots**: Added `payslip_segments` table to store detailed calculation parameters for every segment, ensuring an immutable audit trail.
    - **Policy Resolver**: Created `PolicyResolver` domain service to handle hierarchical configuration resolution (Department > Workspace).
- **Full-Stack Multi-Language Parity (vi, en, zh)**: Achieved 100% key synchronization across all supported languages.
    - **Structural Synchronization**: Refactored `vi.json` and `zh.json` to strictly match the `en.json` baseline, moving feature modules (auth, ess, shifts, etc.) to the top level.
    - **Automated i18n Audit Suite**: Added a new test case to `translation-usage.test.ts` that enforces 100% key parity between all locale files, preventing desync in future development.
    - **Functional Key Coverage**: Verified that every translation key used in the codebase is present in all three language files.
- **Global UI Localization Hardening**:
    - **Reports & Logs**: Localized "Punch In" and "Punch Out" headers and status labels.
    - **Payroll Configuration**: Localized "Loading settings..." and other dynamic UI feedback.
    - **Assignment Rules**: Internationalized all form placeholders and condition hints.
    - **Working Patterns**: Localized pattern name and description placeholders.
    - **Leave Management**: Standardized "Carry-over" and "Carry-over limit" terminology across languages.
- **Strict Rust "Zero-Warning" Policy**: Enforced a project-wide policy where all Rust code must pass `cargo clippy --all-targets --all-features -- -D warnings`.
...
    - **Lint Cleanliness**: Refactored over 50 files to resolve stylistic, performance, and structural warnings.
    - **Complexity Management**: Introduced type aliases for deeply nested types in test infrastructure.
- **E2E Locator Standardization (Hybrid Pattern)**: Migrated the entire Playwright test suite to a resilient locator strategy.
...
### Changed
- **Payroll Architecture**: Transitioned from a monolithic calculation to a segmented approach. `PayrollService` now orchestrates the segmentation and aggregation logic.
- **Legacy Payroll Cleanup**: Removed the deprecated `salary_history` table and associated API endpoints/legacy logic. The system now exclusively uses the Dynamic Payroll (Salary Components) architecture for all calculations.
- **SQLx Upgrade (v0.8.6)**: Upgraded `sqlx` from 0.7 to 0.8.6 in `apiserver` to resolve future-incompatibility warnings (Rust 2024 never-type fallback) and synchronize with other crates.
- **SQLx Metadata Synchronization**: Updated offline query metadata in `.sqlx/` across the project.
- **Optimized E2E Timeouts**: Decreased global test timeout from **30s to 15s** and added a **5s expectation timeout** to enable faster failure detection during development and CI.
...
### Fixed
- **Rust Clippy Findings**: Manually resolved structural warnings including item ordering (moving `mod tests` to the end of files) and unnecessary borrows.
- **E2E Initialization Race Conditions**: Increased global test timeouts to **30 seconds** across Playwright, Vitest, and Pytest to ensure isolated environments (Rust + Vite) have sufficient time to start under parallel load.

## [Unreleased] - 2026-01-17
...