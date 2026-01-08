# Implementation Plan: Payroll and Shift Management System

## Approach
- **Why this solution**: To move from simple hour tracking to a professional payroll system that handles rates, shifts, and automated payslip generation.
- **Alternatives considered**: Manual CSV export and external Excel calculation (discarded for poor user experience and lack of automation).

## Steps

1. **Database Migration** (10 min)
   - Create `apiserver/migrations/20260108230000_payroll_and_shifts.sql`.
   - Add tables: `salary_settings`, `shifts`, `employee_shifts`, `payslips`.

2. **Backend Models & Repositories** (30 min)
   - Update `apiserver/src/models.rs` with new structs.
   - Implement `SalaryRepository`, `ShiftRepository`, and `PayrollRepository`.

3. **Service Layer Implementation** (40 min)
   - Create `apiserver/src/services/payroll_service.rs` for net pay calculation.
   - Create `apiserver/src/services/shift_service.rs` for scheduling logic.

4. **API Handlers** (20 min)
   - Add endpoints in `apiserver/src/handlers.rs`.
   - Register routes in `apiserver/src/lib.rs`.

5. **Frontend UI** (40 min)
   - Update `webdashboard/src/lib/api.ts` with new endpoints.
   - Create pages for Shift Management and Payroll.

## Timeline
| Phase | Duration |
|-------|----------|
| Migration | 10 min |
| Backend Logic | 70 min |
| API Endpoints | 20 min |
| Frontend UI | 40 min |
| **Total** | **2 hours 20 min** |

## Rollback Plan
- Run `sqlx migrate revert`.
- Revert source changes via `git checkout`.

## Security Checklist
- [ ] RBAC: Only 'admin' or 'owner' roles can view/edit salary data.
- [ ] Input validation: Ensure salary rates and shift times are valid.
- [ ] Audit logs: Log changes to salary settings.
