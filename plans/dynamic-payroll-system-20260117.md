# Implementation Plan: Dynamic Payroll & Attendance System V2

## Approach: Multi-Segment Calculation with Granular Snapshots
This plan implements a robust payroll engine capable of handling mid-period changes (e.g., department transfers, salary updates) by slicing the pay period into "segments". It also introduces "Working Conditions" to handle authorized adjustments (e.g., early leave for childcare) at the attendance level.

## 1. Database Schema Refinement

### 1.1 Departments Update
Allow departments to override the workspace-level standard working days.
```sql
ALTER TABLE departments ADD COLUMN std_working_days DECIMAL(5, 2); -- Nullable
```

### 1.2 Working Conditions (New Table)
Store special policies for employees (e.g., Childcare, Pregnancy).
```sql
CREATE TABLE working_conditions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL, -- 'EARLY_LEAVE', 'LATE_IN'
    minute_allowance INTEGER NOT NULL, -- e.g., 60 minutes
    is_paid BOOLEAN NOT NULL DEFAULT FALSE, -- If true, the allowance time counts as paid work hours
    start_date DATE NOT NULL,
    end_date DATE, -- Nullable (Indefinite)
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_working_conditions_employee_date ON working_conditions(employee_id, start_date, end_date);
```

### 1.3 Payslip Segments (New Table)
Store the granular calculation details for each slice of the payroll period. This acts as the **Snapshot** of history.
```sql
CREATE TABLE payslip_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payslip_id UUID NOT NULL REFERENCES payslips(id) ON DELETE CASCADE,
    
    -- Segment Context
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    department_id UUID REFERENCES departments(id), -- Snapshot of dept at this time
    
    -- Snapshot Parameters
    applied_base_salary DECIMAL(15, 2) NOT NULL,
    applied_std_days DECIMAL(5, 2) NOT NULL, -- The denominator used (22, 26, etc.)
    
    -- Calculated Values
    actual_work_days DECIMAL(5, 2) NOT NULL DEFAULT 0,
    paid_work_days DECIMAL(5, 2) NOT NULL DEFAULT 0, -- Actual + Paid Leaves + Adjustment Allowances
    amount DECIMAL(15, 2) NOT NULL,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_payslip_segments_payslip_id ON payslip_segments(payslip_id);
```

## 2. Core Logic Implementation

### 2.1 Policy Resolver (`apiserver/src/domain/policy/resolver.rs`)
Implement logic to resolve the configuration active at a specific date.
- `resolve_standard_days(employee_id, date)`: 
    - Check Department setting at `date`.
    - If None, fall back to `WorkspaceSettings`.
- `resolve_working_conditions(employee_id, date)`:
    - Fetch active records from `working_conditions`.

### 2.2 Attendance Processor (`apiserver/src/services/attendance/processor.rs`)
Update `process_daily_log` to respect Working Conditions.
- **Before:** If `checkout < shift_end`, mark `EARLY_LEAVE`.
- **After:** 
    - Calculate `early_minutes`.
    - Check if `EARLY_LEAVE` condition exists.
    - If `early_minutes <= condition.minute_allowance`:
        - Status = `PRESENT` (or `AUTHORIZED_EARLY`).
        - If `condition.is_paid`: Add `early_minutes` to `paid_hours`.

### 2.3 Payroll Segmentation Engine (`apiserver/src/services/payroll/calculator.rs`)
**The "Slicing" Algorithm:**
1.  **Identify Change Points:**
    - Scan `SalaryHistory`, `JobHistory` (Department transfers) within the payroll period.
    - Collect all effective dates.
2.  **Create Segments:**
    - Split the payroll period (e.g., 2026-01-01 to 2026-01-31) into segments based on change points.
    - E.g., Change on Jan 15 -> Segments: [Jan 1 - Jan 14], [Jan 15 - Jan 31].
3.  **Calculate Segment:**
    - For each segment:
        - `std_days` = `resolve_standard_days(employee, segment_start)`.
        - `daily_rate` = `base_salary / std_days`.
        - `paid_days` = Sum of paid days from Attendance & Leaves in this range.
        - `amount` = `daily_rate * paid_days`.
4.  **Persist:**
    - Save `Payslip` (Total Amount).
    - Save `PayslipSegments` (Details).

## 3. Testing Strategy

### 3.1 Unit Tests
- `test_resolve_std_days_hierarchy`: Verify Dept overrides Workspace.
- `test_attendance_adjustment_paid`: Verify early leave is counted as paid hours if configured.
- `test_attendance_adjustment_unpaid`: Verify early leave prevents 'Early' status but hours are actual.

### 3.2 Integration Tests (`tests/payroll_flow.rs`)
- **Scenario 1: Mid-month Department Transfer**
    - Emp starts in Factory (26 days).
    - Moves to Office (22 days) on 15th.
    - Verify Payslip has 2 segments with correct `applied_std_days`.
    - Verify Total Amount matches the prorated formula.

## 4. Rollback Plan
- Run `sqlx migrate revert` to drop new tables/columns.
- Revert code changes to `PayrollCalculator` to use the legacy "Whole Month" logic.