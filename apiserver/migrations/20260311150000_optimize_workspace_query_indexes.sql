-- Optimize indexes for workspace_id query hardening
-- These support the new WHERE workspace_id = $x filters added to service-layer queries

-- ============================================================
-- SaaS tenant isolation: tables queried by workspace_id but missing index
-- ============================================================

-- shifts: listed by workspace (WHERE workspace_id = $1)
CREATE INDEX IF NOT EXISTS idx_shifts_workspace_id
  ON shifts(workspace_id);

-- holidays: holiday calendar per workspace
CREATE INDEX IF NOT EXISTS idx_holidays_workspace_id
  ON holidays(workspace_id);

-- workspace_members: has user_id index but not workspace_id leading
CREATE INDEX IF NOT EXISTS idx_workspace_members_workspace_id
  ON workspace_members(workspace_id);

-- ============================================================
-- Composite indexes for query patterns with workspace_id
-- ============================================================

-- time_entries: DELETE/SELECT by (employee_id, workspace_id, punch_in_time range)
-- Existing idx_time_entries_ws_employee(workspace_id, employee_id) helps but
-- adding punch_in_time avoids heap filter on time range scans
CREATE INDEX IF NOT EXISTS idx_time_entries_ws_emp_punch_time
  ON time_entries(workspace_id, employee_id, punch_in_time);

-- leave_balances: composite for existence check (JOIN employees for ws scope)
CREATE INDEX IF NOT EXISTS idx_leave_balances_emp_leave_type
  ON leave_balances(employee_id, leave_type_id);

-- department_payroll_configs: FK index for JOIN
CREATE INDEX IF NOT EXISTS idx_dept_payroll_configs_department_id
  ON department_payroll_configs(department_id);

-- employee_payroll_configs: FK index for JOIN
CREATE INDEX IF NOT EXISTS idx_emp_payroll_configs_employee_id
  ON employee_payroll_configs(employee_id);
