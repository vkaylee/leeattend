# Implementation Plan: Special Attendance Policies (Refactor & Parental Support)

Automatically handle early leave for parents and other special conditions using a robust Policy system.

## Approach
- **Goal:** Support "Early Leave" and other special conditions (like breastfeeding breaks) using a flexible policy system rather than duplicating shifts.
- **Why this solution:** We already have a `working_conditions` table and `PolicyResolver`. We will now fully integrate them into the calculation engine.
- **Cleanup:** Since the app is not in production, we will refactor the `AttendanceCalculator` to accept a `PolicyRules` struct instead of a long list of parameters, improving maintainability.

## Steps

1. **Backend Models: Define Policy Structs** (10 min)
   - Modify `apiserver/src/models.rs`:
     - Add `PolicyRules` struct containing all calculation flags (`grace_period`, `early_leave_minutes`, etc.).
     - Update `WorkingCondition` type to include standard types.

2. **Backend: Refactor Attendance Calculator** (20 min)
   - Modify `apiserver/src/services/attendance_calculator.rs`:
     - Change signature of `calculate_buckets` and `calculate_daily_units` to take `&PolicyRules`.
     - Update logic to correctly apply `authorized_early_leave_minutes` from the rules.
     - Update all unit tests in the file.

3. **Backend: Implement Policy Resolver logic** (15 min)
   - Enhance `apiserver/src/domain/policy/resolver.rs`:
     - Create a method `resolve_attendance_rules(pool, employee_id, date)` that:
       1. Fetches workspace defaults.
       2. Fetches `working_conditions` for the employee.
       3. Merges them into a `PolicyRules` object.

4. **Backend: Service Integration** (20 min)
   - Modify `apiserver/src/services/report_service.rs` and `payroll_service.rs`:
     - Replace hardcoded parameters with a call to `PolicyResolver::resolve_attendance_rules`.

5. **Frontend: Working Conditions Management** (30 min)
   - Modify `webdashboard/src/pages/EmployeeDetailPage.tsx`:
     - Add a section to manage "Working Conditions" (CRUD for the `working_conditions` table).

6. **Testing** (20 min)
   - Create `apiserver/tests/special_policy_tests.rs` to verify that an employee with an "EARLY_LEAVE" condition of 60 minutes is calculated correctly.

## Timeline
| Phase | Duration |
|-------|----------|
| Models & Calculator Refactor | 30 min |
| Policy Resolver & Integration | 35 min |
| Frontend UI | 30 min |
| Testing | 20 min |
| **Total** | **~2 hours** |

## Rollback Plan
- Revert code changes.

## Security Checklist
- [x] Input validation for policy parameters.
- [x] Auth checks: Only admins/owners can assign policies.
