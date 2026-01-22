# Implementation Plan: Employee Single Name Field Refactor

### ## Approach
- **Refactor Goal**: Replace `first_name` and `last_name` with a single `full_name` field in the `employees` table and throughout the application.
- **Why this solution**: Simplifies the system, improves cultural inclusivity for varied name structures, and reduces redundant UI fields.
- **Migration Strategy**: 
    1.  Add `full_name` column.
    2.  Update `full_name` with `first_name || ' ' || last_name`.
    3.  Drop `first_name` and `last_name` columns.
- **Alternatives considered**: Keeping 2 fields but making one optional. Rejected as it maintains unnecessary complexity in models and forms.

### ## Steps

1.  **Database Migration** (10 min)
    -   Create a new migration in `apiserver/migrations/`.
    -   Combine existing name data into the new field.
    ```sql
    ALTER TABLE employees ADD COLUMN full_name VARCHAR(255);
    UPDATE employees SET full_name = TRIM(first_name || ' ' || last_name);
    -- Handle NOT NULL after data migration
    ALTER TABLE employees ALTER COLUMN full_name SET NOT NULL;
    -- Drop old columns
    ALTER TABLE employees DROP COLUMN first_name;
    ALTER TABLE employees DROP COLUMN last_name;
    ```

2.  **Backend Model & Repository Update** (30 min)
    -   Modify `apiserver/src/models.rs`: Update `Employee`, `CreateEmployeeRequest`, and `UpdateEmployeeRequest` structs.
    -   Modify `apiserver/src/repositories/employee_repository.rs`: Update SQL queries for CRUD operations.
    -   Modify `apiserver/src/repositories/leave_balance_repository.rs`, `roster_repository.rs`, etc. (everywhere `first_name` was used).

3.  **API Handlers & Service Update** (20 min)
    -   Modify `apiserver/src/services/employee_service.rs` and `attendance_import_service.rs`.
    -   Modify `apiserver/src/handlers/attendance.rs`.

4.  **Frontend Type & API Update** (15 min)
    -   Modify `webdashboard/src/lib/api/types.ts`: Update `Employee` interface.
    -   Modify `webdashboard/src/lib/api/attendance.ts`.

5.  **UI Components & Forms Update** (45 min)
    -   Modify `webdashboard/src/pages/EmployeesPage.tsx`: Update form inputs and table display.
    -   Modify `webdashboard/src/pages/ReportsPage.tsx`, `PayrollPage.tsx`, `ShiftsPage.tsx`.
    -   Update common components like `EmployeeSearch.tsx` and `AssignPolicyModal.tsx`.
    -   Update translation files (`en.json`, `vi.json`, `zh.json`).

6.  **Tests & Verification** (30 min)
    -   Update backend integration tests in `apiserver/tests/`.
    -   Update frontend Vitest tests in `webdashboard/src/pages/__tests__/`.
    -   Run `./run-tests.sh` to ensure full system parity.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Migration | 10 min |
| Backend Refactor | 50 min |
| Frontend Refactor | 60 min |
| Testing & Fixes | 60 min |
| **Total** | **3 hours** |

### ## Rollback Plan
1.  Revert code changes via `git checkout`.
2.  Run a reverse migration to split `full_name` back into `first_name` and `last_name` (Note: Splitting is imprecise).

### ## Security Checklist
- [ ] Ensure `full_name` has length validation (min 1, max 255) on backend.
- [ ] Verify that Roster and Payroll reports still correctly identify employees by ID.
- [ ] No changes to Auth/RBAC logic required.
