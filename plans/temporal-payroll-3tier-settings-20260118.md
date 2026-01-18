# Implementation Plan: 3-Tier Temporal Payroll Settings

## Approach
We will implement a hierarchical, versioned configuration system for "Standard Working Days" (Số ngày công chuẩn). This solves the problem of contract changes over time and special individual exceptions.

**Hierarchy of Resolution (Highest to Lowest Priority):**
1.  **Individual Override:** Versioned config specific to an employee.
2.  **Department Override:** Versioned config specific to a department.
3.  **Workspace Setting:** Versioned global fallback for the entire workspace.
4.  **System Default:** Hardcoded fallback (26 days).

**Key Technical Pattern:**
Each level will use an `effective_date`. The system will always fetch the record where `effective_date <= target_date` with the `MAX(effective_date)`.

## Steps

### 1. Database Schema (20 min)
Create migration: `apiserver/migrations/20260118030000_temporal_payroll_3tier.sql`
- **Global:** Refactor `payroll_settings` to include `effective_date` and remove single-row restriction.
- **Department:** Create `department_payroll_configs` (dept_id, std_working_days, effective_date).
- **Individual:** Create `employee_payroll_configs` (employee_id, std_working_days, effective_date).
- **Indices:** Add composite unique indices on `(id, effective_date)` for all three to prevent duplicate versions on the same day.

### 2. Backend Implementation (45 min)

#### 2.1. Repository Updates
- `payroll_repository.rs`: Update `get_payroll_settings` to support point-in-time lookup.
- `department_repository.rs`: Add methods to manage `department_payroll_configs`.
- `employee_repository.rs`: Add methods to manage `employee_payroll_configs`.

#### 2.2. Policy Resolver (The Brain)
- `apiserver/src/domain/policy/resolver.rs`:
    - Refactor `resolve_standard_days` to implement the 3-tier lookup logic.
    - It must accept `employee_id`, `department_id`, `workspace_id`, and the `date` of the payroll period.

#### 2.3. API Handlers
- Add/Update endpoints to allow admins to "Set Standard Days" at the Employee, Department, or Workspace level with an `effective_date`.

### 3. Frontend Implementation (45 min)
- **Workspace Level:** Update `PayrollSettingsSection.tsx` to handle versioned history.
- **Department Level:** Add a "Payroll Rules" tab in the Department edit modal.
- **Employee Level:** Add a "Payroll Rules" section in the Employee profile or salary config tab.
- **Components:** Create a reusable `TemporalConfigTable` to show history and add new versions.

### 4. Verification (20 min)
- **TDD:** Add a specific integration test `test_3tier_resolution` in `apiserver/tests/payroll_v2_tests.rs`.
    - Case: Emp A in Dept B.
    - Global = 26.
    - Dept B = 24 (Effective Jan 1).
    - Emp A = 22 (Effective Feb 1).
    - Resolution for Jan -> 24.
    - Resolution for Feb -> 22.

## Timeline
| Phase | Duration |
|-------|----------|
| DB & Migrations | 20 min |
| Backend Core Logic | 40 min |
| API & Handlers | 20 min |
| Frontend UI | 40 min |
| Verification | 20 min |
| **Total** | **~2.5 Hours** |

## Security Checklist
- [ ] **RBAC:** Only `owner` or `admin` can set these configurations.
- [ ] **Validation:** `std_working_days` must be > 0 and <= 31.
- [ ] **Date Integrity:** `effective_date` should ideally not be changed for already "Finalized" payslips (warn the user).
