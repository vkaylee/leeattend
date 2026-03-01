-- Add missing workspace_id indexes for tenant isolation and query performance

-- payroll_incentive_policies: NO index at all
CREATE INDEX IF NOT EXISTS idx_payroll_incentive_policies_workspace
  ON payroll_incentive_policies(workspace_id);

-- working_patterns: NO index at all
CREATE INDEX IF NOT EXISTS idx_working_patterns_workspace
  ON working_patterns(workspace_id);

-- leave_accrual_logs: has workspace_id but no index
CREATE INDEX IF NOT EXISTS idx_leave_accrual_logs_workspace
  ON leave_accrual_logs(workspace_id);

-- leave_policy_assignments: only has GIST exclusion, no B-tree index
CREATE INDEX IF NOT EXISTS idx_leave_policy_assignments_workspace
  ON leave_policy_assignments(workspace_id);

-- employee_working_patterns: has workspace_id but only (employee_id, start_date) index
CREATE INDEX IF NOT EXISTS idx_employee_working_patterns_workspace
  ON employee_working_patterns(workspace_id);

-- Composite indexes for common query patterns

-- attendance_daily_summaries: common query is (workspace_id, employee_id, date)
CREATE INDEX IF NOT EXISTS idx_daily_summaries_ws_emp_date
  ON attendance_daily_summaries(workspace_id, employee_id, date);

-- shift_swap_requests: queries by requester/target within workspace
CREATE INDEX IF NOT EXISTS idx_shift_swap_ws_requester
  ON shift_swap_requests(workspace_id, requester_id);
CREATE INDEX IF NOT EXISTS idx_shift_swap_ws_target
  ON shift_swap_requests(workspace_id, target_id);

-- time_entries: common query is workspace + employee
CREATE INDEX IF NOT EXISTS idx_time_entries_ws_employee
  ON time_entries(workspace_id, employee_id);
