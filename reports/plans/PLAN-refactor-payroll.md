# Implementation Plan: Refactor Payroll Service

## 📌 User Request (VERBATIM)
> Improve và implement test và dùng ai-test.sh để test

## 🎯 Acceptance Criteria (Derived from User Request)
| ID | Criterion | Verification Method |
|----|-----------|---------------------|
| AC1 | Improve `payroll_service` (SRP) | Function `calculate_monthly_payslip` is broken down |
| AC2 | Implement tests | Add basic unit tests for the extracted inner parts |
| AC3 | Use `ai-test.sh` to test | Execute `./ai-test.sh` and ensure it succeeds |

## 📋 Context Summary
**Architecture**: `calculate_monthly_payslip` is a large function that needs extraction.
**Patterns**: Extract method pattern, encapsulate loop state.
**Constraints**: Do not break the current behavior of payroll calculation. Follow existing timezone logic and ot_ceiling.

## Overview
Refactor `calculate_monthly_payslip` into smaller helper functions in `PayrollService` to improve SRP and readability. Validate changes with `./ai-test.sh --api`.

## Phase 1: Refactor `payroll_service.rs`
### Tasks
- [x] Task 1.1: Extract `resolve_segment_context` block.
  - Agent: `backend-engineer`
  - File(s): `apiserver/src/services/payroll_service.rs`
  - Acceptance: The function is smaller.
- [x] Task 1.2: Extract inner loop `calculate_daily_attendance`.
  - Agent: `backend-engineer`
  - File(s): `apiserver/src/services/payroll_service.rs`
  - Acceptance: The nested loops for shifts/entries are moved to a separate function.

### Exit Criteria
- [x] `calculate_monthly_payslip` reads like a high-level orchestration function.

## Phase 2: Implement Tests
### Tasks
- [ ] Task 2.1: Add / check tests for `calculate_daily_attendance` or the new helpers.
  - Agent: `tester`
  - File(s): `apiserver/src/services/payroll_service.rs` (tests module)
  - Acceptance: Unit tests cover the newly extracted logic or confirm it works.

## Risks
| Risk | Impact | Mitigation | Rollback |
|------|--------|------------|----------|
| Breaking OT Logic | High | Keep the exact same logic and just pass variables around | `git checkout -- apiserver/src/services/payroll_service.rs` |

## Implementation Notes
Ensure mutable accumulators (like `total_late_early_count`, `segment_std_units`, `segment_ot_units`) are properly handled either as return tuples or wrapped in a tracking struct.
