# Implementation Plan: Deprecate Legacy Fixed Shifts & Migrate to Patterns (2026-01-17)

## Problem
The system currently maintains a legacy "Fixed Shift" assignment (`employee_shifts` table) and a modern "Working Pattern" system. Maintaining both adds complexity to the effective shift resolution logic and creates confusion for users. We need to unify everything into the Pattern system.

## Approach
- **Phased Deprecation**: 
    1. Migrate existing data from `employee_shifts` to `employee_working_patterns`.
    2. Simplify `get_effective_shift` logic by removing levels 4 and 5.
    3. Update the UI to hide "Fixed Shift" selectors.
    4. Refactor tests to stop using legacy tables.

## Steps

### 1. Database: Migration & Conversion (20 min)
- Create a migration that:
    - Automatically creates a 7-day "Fixed Pattern" for each unique shift currently assigned fixedly.
    - Inserts records into `employee_working_patterns` based on `employee_shifts`.
    - Marks `employee_shifts` and `department_default_shifts` tables as deprecated (rename or comment).

### 2. Backend: Logic Simplification (30 min)
- **Files to modify**: `apiserver/src/repositories/roster_repository.rs`
- **Action**: Remove SQL joins/subqueries related to `employee_shifts` and `department_default_shifts`. The new priority will be:
    1. `shift_rosters` (Manual Override)
    2. `employee_working_patterns` (Individual Pattern)
    3. `departments.default_working_pattern_id` (Department Pattern)

### 3. UI: User Experience Unification (20 min)
- **Files to modify**: 
    - `webdashboard/src/pages/EmployeesPage.tsx`
    - `webdashboard/src/pages/DepartmentsPage.tsx`
- **Action**: Remove the "Fixed Shift" dropdown and replace it with a link to the "Schedule/Patterns" tab.

### 4. Testing: Refactor Suite (30 min)
- **Files to modify**: Multiple files in `apiserver/tests/`
- **Action**: Replace raw SQL inserts into `employee_shifts` with helper calls to assign patterns.

## Timeline
| Phase | Duration |
|-------|----------|
| Migration | 20 min |
| Logic Refactor | 30 min |
| UI Cleanup | 20 min |
| Test Refactor | 30 min |
| **Total** | **1 hour 40 min** |

## Rollback Plan
1. Revert code changes.
2. The migration will be additive or re-enable the old joins.

## Security Checklist
- [ ] Ensure `end_date` is correctly handled during migration to prevent future schedule overlap.
