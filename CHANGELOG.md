# Changelog

Tất cả các thay đổi lớn của dự án sẽ được ghi nhận tại đây.

## [Unreleased]

### Added
- **Batch Payroll Calculation**:
    - Backend: Implemented `calculate_batch_payslips` in `PayrollService` utilizing async parallel processing for high performance.
    - Backend: Added `POST /api/workspaces/:ws_id/payroll/calculate-batch` endpoint.
    - Frontend: Enhanced `PayrollPage` with a multi-selection sidebar, including "Select All" and individual checkboxes.
    - Frontend: Created `BatchCalculationModal` with real-time progress tracking and summary of success/error results.
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