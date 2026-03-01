-- Comprehensive Index Optimization Migration
-- Based on full schema audit: adds missing FK indexes, tenant isolation indexes,
-- and composite indexes for common query patterns.

-- ============================================================
-- 🔴 HIGH PRIORITY: Tables with no indexes or missing FK indexes
-- ============================================================

-- payroll_incentive_tiers: NO indexes at all
CREATE INDEX IF NOT EXISTS idx_incentive_tiers_policy_id
  ON payroll_incentive_tiers(policy_id);
CREATE INDEX IF NOT EXISTS idx_incentive_tiers_policy_priority
  ON payroll_incentive_tiers(policy_id, priority);

-- overtime_rules: missing FK index
CREATE INDEX IF NOT EXISTS idx_overtime_rules_policy_id
  ON overtime_rules(policy_id);

-- payslip_incentives: missing FK index
CREATE INDEX IF NOT EXISTS idx_payslip_incentives_payslip_id
  ON payslip_incentives(payslip_id);

-- ============================================================
-- 🟠 MEDIUM PRIORITY: Tenant isolation & temporal queries
-- ============================================================

-- overtime_policy_definitions: missing workspace_id index
CREATE INDEX IF NOT EXISTS idx_overtime_policy_definitions_workspace
  ON overtime_policy_definitions(workspace_id);

-- leave_policy_definitions: missing workspace_id B-tree index
CREATE INDEX IF NOT EXISTS idx_leave_policy_definitions_workspace
  ON leave_policy_definitions(workspace_id);

-- salary_component_definitions: workspace_id filter
CREATE INDEX IF NOT EXISTS idx_salary_comp_defs_workspace
  ON salary_component_definitions(workspace_id);

-- employee_department_history: missing department FK index
CREATE INDEX IF NOT EXISTS idx_emp_dept_hist_department_id
  ON employee_department_history(department_id);

-- employee_department_history: temporal lookup (current department)
CREATE INDEX IF NOT EXISTS idx_emp_dept_hist_emp_date
  ON employee_department_history(employee_id, effective_date DESC);

-- employee_salary_components: missing FK index for component_definition_id
CREATE INDEX IF NOT EXISTS idx_employee_salary_comp_def_id
  ON employee_salary_components(component_definition_id);

-- employee_shifts: shift_id FK index already exists (20260124160000_performance_and_search.sql)

-- ============================================================
-- 🟡 LOW PRIORITY: Query performance improvements
-- ============================================================

-- leave_requests: date range lookup
CREATE INDEX IF NOT EXISTS idx_leave_requests_date_range
  ON leave_requests(start_date, end_date);

-- leave_balances: single-column employee_id lookup
CREATE INDEX IF NOT EXISTS idx_leave_balances_employee_id
  ON leave_balances(employee_id);

-- payslips: status filter
CREATE INDEX IF NOT EXISTS idx_payslips_status
  ON payslips(status);

-- shift_rosters: FK shift_id for cascade SET NULL
CREATE INDEX IF NOT EXISTS idx_shift_rosters_shift_id
  ON shift_rosters(shift_id);

-- raw_attendance_logs: composite index for common query pattern
-- (covers workspace + employee + time range scans)
CREATE INDEX IF NOT EXISTS idx_raw_logs_ws_emp_time
  ON raw_attendance_logs(workspace_id, employee_id, punch_time);

-- ============================================================
-- 🔵 ADDITIONAL: Remaining FK indexes from second pass
-- ============================================================

-- leave_policy_assignments: FK policy_id (used in JOINs and RESTRICT deletes)
CREATE INDEX IF NOT EXISTS idx_leave_policy_assignments_policy_id
  ON leave_policy_assignments(policy_id);

-- overtime_policy_assignments: FK policy_id
CREATE INDEX IF NOT EXISTS idx_overtime_policy_assignments_policy_id
  ON overtime_policy_assignments(policy_id);

-- assignment_rules: FK pattern_id (JOIN with working_patterns)
CREATE INDEX IF NOT EXISTS idx_assignment_rules_pattern_id
  ON assignment_rules(pattern_id);

-- attendance_adjustment_requests: FK time_entry_id (JOIN with time_entries)
CREATE INDEX IF NOT EXISTS idx_attendance_adj_time_entry_id
  ON attendance_adjustment_requests(time_entry_id);

-- employee_working_patterns: FK pattern_id (JOIN with working_patterns)
CREATE INDEX IF NOT EXISTS idx_employee_working_patterns_pattern_id
  ON employee_working_patterns(pattern_id);

-- payslip_segments: FK payslip_id already indexed, add department_id FK
CREATE INDEX IF NOT EXISTS idx_payslip_segments_department_id
  ON payslip_segments(department_id);

-- departments: FK manager_id (SET NULL on delete — needs scan)
CREATE INDEX IF NOT EXISTS idx_departments_manager_id
  ON departments(manager_id);

-- leave_accrual_logs: FK policy_assignment_id
CREATE INDEX IF NOT EXISTS idx_leave_accrual_logs_policy_assignment_id
  ON leave_accrual_logs(policy_assignment_id);
