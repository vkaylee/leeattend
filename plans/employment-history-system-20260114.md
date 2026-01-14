# Implementation Plan: Employment History System (Re-hire Support)

### ## Approach
Normalize employment data by extracting work periods into a dedicated `employment_periods` table. This allows an employee to have multiple "hire" and "termination" events over time, enabling accurate seniority calculation and auditability without duplicating core personal data.

### ## Steps

1. **Database Migration** (10 min)
   - Create `employment_periods` table.
   - Migrate existing `hire_date` and `termination_date` from `employees` table.
   - Remove `hire_date` and `termination_date` from `employees`.
   ```sql
   CREATE TABLE employment_periods (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
       hire_date DATE NOT NULL,
       termination_date DATE, -- NULL if active
       is_rehire BOOLEAN DEFAULT FALSE,
       created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
   );
   -- Migrate data
   INSERT INTO employment_periods (employee_id, hire_date, termination_date)
   SELECT id, hire_date, termination_date FROM employees;
   -- Cleanup employees
   ALTER TABLE employees DROP COLUMN hire_date;
   ALTER TABLE employees DROP COLUMN termination_date;
   ```

2. **Core Backend Refactor** (20 min)
   - **Models**: Add `EmploymentPeriod` struct to `models.rs`.
   - **Repositories**: Update `EmployeeRepository` to fetch the latest `hire_date` via a JOIN or subquery.
   - **Services**:
     - `EmployeeService`: Update `create_employee` to insert into both tables.
     - Add `rehire_employee(employee_id, new_hire_date)` method.
     - `LeaveAccrualService`: Update seniority logic to sum all finished periods + current active period.

3. **API Integration** (15 min)
   - New endpoint: `GET /workspaces/:ws_id/employees/:id/history` to view all periods.
   - New endpoint: `POST /workspaces/:ws_id/employees/:id/rehire` to start a new period.

4. **UI Updates** (15 min)
   - Update `EmployeesPage`: Show latest `hire_date`.
   - Update `EmployeeDetails`: Add "Employment History" tab.
   - Add "Re-hire" button for inactive employees.

### ## Timeline
| Phase | Duration |
|-------|----------|
| DB Migration | 10 min |
| Backend Refactor | 25 min |
| API Handlers | 15 min |
| UI Components | 20 min |
| **Total** | **1 hour 10 min** |

### ## Rollback Plan
- Re-add `hire_date` and `termination_date` to `employees`.
- Populate them from the latest `employment_periods` record.
- Drop `employment_periods` table.

### ## Security Checklist
- [x] RBAC: Ensure only `owner` and `admin` can trigger a re-hire.
- [x] Validation: Ensure `new_hire_date` is after the last `termination_date`.
- [x] Audit: Keep `created_at` for all periods.

### ## Success Criteria
- [ ] Employee can be "Terminated" and then "Re-hired" multiple times.
- [ ] Seniority calculation in `LeaveAccrualService` correctly handles gaps between periods.
- [ ] 100% of existing tests pass after refactor.
