### # Implementation Plan: Dynamic Payroll System (Component-Based)

### ## Approach
- **Solution:** Use a **Component-Based Architecture** where every pay item (Base Salary, Meal Allowance, etc.) is a distinct entity.
- **Why this:** 
    - Decouples calculation logic from hard-coded fields.
    - Allows per-workspace (SaaS tenant) configuration of "What counts as Base for OT".
    - Supports flexible "Standard Working Day" policies (Fixed vs. Actual) which varies by company culture.
- **Alternatives considered:**
    - *Formula Engine:* Too complex for MVP, high risk of user error.
    - *Hard-coded Templates:* Not scalable for a SaaS targeting diverse industries.

---

### ## Steps

1. **Database Migrations** (15 min)
   - Create `payroll_settings` to store workspace-level rules (Std day mode, hours/day).
   - Create `salary_components` to define types of pay.
   - Create `employee_salary_configs` to link employees to components.
   ```sql
   -- Example schema snippet
   CREATE TABLE salary_components (
       id TEXT PRIMARY KEY,
       workspace_id TEXT,
       name TEXT,
       is_base_for_ot BOOLEAN,
       is_taxable BOOLEAN
   );
   ```

2. **Backend Logic - Payroll Calculator** (40 min)
   - Implement `PayrollCalculator` in `apiserver/src/services/payroll/`.
   - Logic: 
     1. Calculate `StandardDays` based on policy.
     2. Identify `BaseForOT` by summing components with `is_base_for_ot = true`.
     3. Compute `HourlyRate = BaseForOT / (StandardDays * HoursPerDay)`.
     4. Finalize `Total = ΣComponents + (HourlyRate * OTHours * Multiplier)`.

3. **API Endpoints Integration** (20 min)
   - `GET/PATCH /workspaces/{id}/payroll-settings`
   - `GET/POST /salary-components`
   - `POST /payroll/calculate` (Dry run or finalize).

4. **Frontend UI** (45 min)
   - **Settings Page:** Toggle between "Fixed days" and "Actual days".
   - **Component Manager:** CRUD for Allowances with a "Calculate OT based on this" checkbox.
   - **Payslip Preview:** Breakdown of how the OT rate was derived.

5. **Testing** (30 min)
   - **Unit Tests:** Mock scenarios for 26-day fixed vs. 2026-Jan actual (22 days).
   - **Integration Tests:** Verify component sums in the final payroll result.

---

### ## Timeline
| Phase | Duration |
|-------|----------|
| DB Migrations | 15 min |
| Backend Core Logic | 40 min |
| API Endpoints | 20 min |
| Frontend UI | 45 min |
| Testing & QA | 30 min |
| **Total** | **~2.5 hours** |

---

### ## Rollback Plan
1. Revert database migrations using `sqlx migrate revert`.
2. Git checkout previous stable version of `apiserver` and `webdashboard`.
3. Clear `payroll_results` cache if any calculation was stored.

---

### ## Security Checklist
- [ ] **RBAC:** Only `Admin` or `Owner` can modify `payroll_settings` or `salary_components`.
- [ ] **Data Integrity:** Prevent deletion of components used in historical payroll records (Soft delete).
- [ ] **Input Validation:** Ensure `fixed_std_days` is positive and reasonable (e.g., 15-31).
- [ ] **Audit Logs:** Log every change to an employee's salary configuration.
