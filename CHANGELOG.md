# Changelog

Tất cả các thay đổi lớn của dự án sẽ được ghi nhận tại đây.

## [Unreleased]

### Added
- **Hierarchical 3-tier Temporal Payroll**:
    - New schema for versioned payroll settings at Individual, Department, and Workspace levels.
    - `PolicyResolver` domain service for point-in-time rule resolution.
    - Multi-segment payroll calculation with proration support for mid-month changes.
    - Reusable `TemporalConfigTable` React component for historical setting management.
    - Persistence of `applied_hours_per_day` in payslip segments for auditability.
- **Advanced Debug Suite**:
    - Floating Developer Toolbar for quick access to debug features.
    - **User Impersonation**: Switch roles (Owner/Admin/Member) or specific users in one click via `X-Debug-Impersonate-Id`.
    - **Time Travel**: Simulate any system date via `X-Debug-Mock-Date` to test future/past payroll logic.
    - **Debug API Directory**: Endpoint `/api/debug/users` to list available test identities.

### Changed
- Increased E2E test stability by raising DB pool `acquire_timeout` to 30s and Playwright navigation timeouts to 60s.
- Optimized sidebar navigation in E2E tests to handle pre-expanded sections and prevent accidental collapses.
- Switched E2E test URL matching to more flexible partial matching to reduce environment-specific flakiness.
- Synchronized i18n keys for working patterns (`delete_success`, `delete_error`, `confirm_delete`) across en, vi, and zh locales.
- Updated `PayrollService` to use the new hierarchical resolver instead of legacy fixed fields.

### Fixed
- **Department Transfer**: Fixed a bug where employee department transfers were not completing due to API return type mismatch (JSON vs Status).
- **Security Audit**: Added missing RBAC checks to the `transfer_employee` handler to ensure only Admins/Owners can move personnel.
- **Frontend State Leak**: Resolved an issue in `EmployeesPage` where the department selection state persisted between modal sessions.
- **TypeScript & Linting**: Fixed invalid i18next `t()` usage in `WorkingPatternsPage.tsx` and resolved `react-refresh` component export warnings.

### Added
- **3-Tier Temporal Payroll Settings**: Implemented a hierarchical versioning system for "Standard Working Days" and "Hours per Day" (Individual > Department > Workspace).
- **Point-in-Time Policy Resolver**: New logic to resolve payroll settings based on the payroll period's effective date.
- **Temporal Configuration UI**: Added `TemporalConfigTable` for managing historical and future configuration changes across Workspace, Departments, and Employee profiles.
- **Payroll Segmentation**: The payroll engine now automatically splits calculations into segments when settings (salary, department, or std-days) change mid-month.
- **Global Settings History**: Admins can now view and schedule changes to workspace-wide payroll configurations.

### Changed
- Refactored `PayrollService` to support multi-segment calculations and hierarchical policy resolution.
- Updated `PayslipSegment` to record `applied_hours_per_day` for enhanced auditability.
- Synchronized i18n keys for temporal settings across English, Vietnamese, and Chinese locales.

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