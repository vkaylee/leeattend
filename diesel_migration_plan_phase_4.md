# Diesel Migration Plan - Phase 4: "The Long Tail" Migration & Integration

## 1. Overview
This phase focuses on wiring up already implemented Diesel repositories, migrating the remaining repositories using the Hybrid pattern, and performing a final audit and refactoring for consistency and performance.

## 2. Integration Tasks (Wiring existing Diesel Repositories)
Connect existing Diesel implementations in `app_builder.rs` and `apiserver/tests/integration/common/app.rs`.

- [x] `RawAttendanceLogRepository` -> `DieselRawAttendanceLogRepository` (Already done in both files)
- [x] `StagingLogRepository` -> `DieselStagingLogRepository` (Already done in both files)
- [x] `ShiftSwapRepository` -> `DieselShiftSwapRepository` (Already done in both files)
- [x] `DepartmentRepository` -> `DieselDepartmentRepository` (Already done in both files)
- [x] `ImportJobRepository` -> `DieselImportJobRepository` (Already done in both files)
- [x] `SalaryBaseRepository` -> `DieselSalaryBaseRepository` (Already done in both files)
- [x] `EmploymentPhaseRepository` -> `DieselEmploymentPhaseRepository` (Already done in both files)

## 3. Migration Tasks (New Diesel Implementations)
Implement Hybrid Diesel repositories for the following:

- [x] `WorkspaceStatsRepository` (Complex CTE queries - keep as SQLx only, no Diesel conversion needed)
- [x] `FeatureFlagRepository` -> Create trait + `DieselFeatureFlagRepository` + wire into AppState + update handlers
- [x] `AssignmentRuleRepository` -> Create `DieselAssignmentRuleRepository` (trait exists) + wire into AppState + update handlers

## 4. Refactor & Audit Tasks
- [x] **Strict Indentation Audit**: New repo initializations follow existing patterns in `app_builder.rs`.
- [x] **Todo/Panic Audit**: 26 intentional `panic!()` in `from_pool()` — all designed sentinels. Zero `todo!()` found.
- [x] **Agent Crate Audit**: Agent crate uses Redb (embedded K/V), NO SQLx usage. No action needed.
- [x] **Handlers Update**: `handlers/feature_flags.rs` and `handlers/assignment_rules.rs` now use trait-based repos through AppState.

## 5. Verification
- [x] Phase 4 code verified: 0 compile errors from new files (`diesel_feature_flag_repository.rs`, `diesel_assignment_rule_repository.rs`, models, handlers).
- [ ] `./test.sh api --skip-lint`: Blocked by 34 pre-existing Phase 3 errors (employee_service, employment_phase_repository, payroll_service, etc.).
- [ ] Full test run: Blocked pending Phase 3 completion.

---

## Progress Log

### [2026-05-11] Initial Planning
- Created Phase 4 migration plan.
- Analyzed existing repositories and their complexity.

### [2026-05-11] Analysis Complete
- Part A: All 7 repos already wired in both app_builder.rs and app.rs. Nothing to do.
- Part B: 3 repos already have Diesel (ImportJob, SalaryBase, EmploymentPhase). 
  - WorkspaceStats: 807 lines of complex CTE SQL — keep as SQLx only.
  - FeatureFlag + AssignmentRule: Need new Diesel implementations.
- Part C: Panic audit done — 26 intentional sentinels, 0 todo!().
- Part D: Agent crate has NO SQLx — uses embedded Redb K/V store.

### [2026-05-11] Implementation Complete
- Created Diesel entity models: `models/diesel/feature_flag.rs`, `models/diesel/assignment_rule.rs`
- Created `FeatureFlagRepositoryTrait` (new trait in `feature_flag_repository.rs`) with 8 methods
- Created `DieselFeatureFlagRepository` implementing `FeatureFlagRepositoryTrait`
- Created `DieselAssignmentRuleRepository` implementing existing `AssignmentRuleRepositoryTrait`
- Updated `state.rs`: Added `feature_flag_repo` to `PayrollState`, `assignment_rule_repo` to `ShiftState`
- Updated `app_builder.rs`: Wired both repos via `DieselX::new(pool, diesel_pools)` pattern
- Updated `handlers/feature_flags.rs`: Uses `state.payroll.feature_flag_repo.*` trait methods
- Updated `handlers/assignment_rules.rs`: Uses `state.shifts.assignment_rule_repo.*` trait methods
- Registered new modules in `repositories/mod.rs` and `models/diesel/mod.rs`
- Note: Test container has pre-existing build issues (34 errors on clean HEAD). Test wiring (`app.rs`) deferred until container build stabilizes.
- Note: `WorkspaceStatsRepository` intentionally kept as SQLx-only (complex CTE queries with custom DB functions).
