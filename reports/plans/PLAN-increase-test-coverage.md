# Implementation Plan: Increase Test Coverage

## 📌 User Request (VERBATIM)
> increase test coverage to 50%

## 🎯 Acceptance Criteria (Derived from User Request)
| ID | Criterion | Verification Method |
|----|-----------|---------------------|
| AC1 | Coverage script runs successfully | Execute `./run-tests.sh --api --coverage` without compilation errors |
| AC2 | Test coverage >= 50% | LCOV and HTML coverage reports show at least 50% line coverage for the workspace |

## 📋 Context Summary
**Architecture**: The project is split into `apiserver` and `agent-main`. Testing is containerized using `podman`/`docker` via `run-tests.sh`.
**Patterns**: SQLx is used for database interaction, which requires either a live migrated database at compile time or `.sqlx` offline data. Test scripts must follow PTY safety and parallel test isolation.
**Constraints**: Currently, `test-coverage.sh` fails because `apiserver` lacks a live DB connection or `.sqlx` data during the coverage compilation step. This MUST be fixed before measuring or increasing coverage.

## Overview
This plan outlines the steps to resolve the test coverage compilation failure, measure the baseline coverage, write necessary tests to reach 50% coverage, and verify the final result.

## Prerequisites
- [x] Scout report generated (`./reports/scouts/SCOUT-increase-test-coverage.md`)

## Phase 1: Fix Coverage Infrastructure
### Tasks
- [ ] Task 1.1: Fix `apiserver` test coverage compilation
  - Agent: `backend-engineer`
  - File(s): `scripts/test-coverage.sh` or `apiserver/.sqlx/`
  - Acceptance: `cargo llvm-cov` can compile the `apiserver` test suite without SQLx errors
  - Verification: Run `./run-tests.sh --api --coverage` and ensure it passes the building phase

### Exit Criteria
- [ ] Coverage reports are generated successfully

## Phase 2: Measure Baseline & Identify Gaps
### Tasks
- [ ] Task 2.1: Run coverage and identify uncovered files
  - Agent: `tester`
  - File(s): `run-tests.sh`
  - Acceptance: Baseline coverage % is known and target files for new tests are identified
  - Verification: Read the generated `coverage-apiserver-html/index.html` or `summary`

### Exit Criteria
- [ ] Target modules for new tests are selected

## Phase 3: Write Missing Tests
### Tasks
- [ ] Task 3.1: Implement unit and integration tests
  - Agent: `tester`
  - File(s): `apiserver/tests/*` and `agent-main/tests/*`
  - Acceptance: Meaningful tests are written for the target modules
  - Verification: Tests pass locally

### Exit Criteria
- [ ] New tests compile and pass

## Phase 4: Final Coverage Verification
### Tasks
- [ ] Task 4.1: Verify 50% coverage target
  - Agent: `tester`
  - File(s): `run-tests.sh`
  - Acceptance: Overall test coverage is >= 50%
  - Verification: Output of `./run-tests.sh --api --coverage` confirms the target is met

### Exit Criteria
- [ ] AC2 is satisfied

## Risks
| Risk | Impact | Mitigation | Rollback |
|------|--------|------------|----------|
| Tests are flaky | M | Isolate database state per test run | Revert test additions |
| SQLx offline data gets stale | L | Add `cargo sqlx prepare` to CI or scripts | Clear `.sqlx` dir and use live DB |

## Rollback Strategy
1. Revert changes to `test-coverage.sh`.
2. Delete any added test files or `.sqlx` directories.

## Implementation Notes
Implementer MUST start by fixing the SQLx compilation issue during the coverage run. The easiest path might be to run `cargo sqlx prepare --workspace` against the dev DB before running the coverage tests, or inject a database connection URL pointing to the migrated test DB in the coverage script.
