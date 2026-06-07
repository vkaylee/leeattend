-- 1. Alter scope check constraint to include 'employer'
ALTER TABLE workspace_config_entries DROP CONSTRAINT IF EXISTS workspace_config_entries_scope_type_check;
ALTER TABLE workspace_config_entries ADD CONSTRAINT workspace_config_entries_scope_type_check 
    CHECK (scope_type IN ('workspace', 'employer', 'department', 'employee'));

-- 2. Add location_id to holidays
ALTER TABLE holidays ADD COLUMN location_id UUID REFERENCES work_locations(id) ON DELETE SET NULL;
CREATE INDEX idx_holidays_workspace_location_date ON holidays (workspace_id, location_id, date);

-- 3. Migrate payroll_settings -> workspace_config_entries
-- std_working_day_mode
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_text, effective_from)
SELECT workspace_id, 'workspace', NULL, 'payroll.std_working_day_mode', std_working_day_mode, effective_date
FROM payroll_settings;

-- fixed_std_days
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
SELECT workspace_id, 'workspace', NULL, 'payroll.fixed_std_days', fixed_std_days, effective_date
FROM payroll_settings;

-- working_days_per_week
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_int, effective_from)
SELECT workspace_id, 'workspace', NULL, 'payroll.working_days_per_week', working_days_per_week, effective_date
FROM payroll_settings;

-- hours_per_day
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
SELECT workspace_id, 'workspace', NULL, 'payroll.hours_per_day', hours_per_day, effective_date
FROM payroll_settings;

-- max_ot_hours_per_day
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
SELECT workspace_id, 'workspace', NULL, 'payroll.max_ot_hours_per_day', max_ot_hours_per_day, effective_date
FROM payroll_settings
WHERE max_ot_hours_per_day IS NOT NULL;

-- max_ot_hours_per_month
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
SELECT workspace_id, 'workspace', NULL, 'payroll.max_ot_hours_per_month', max_ot_hours_per_month, effective_date
FROM payroll_settings
WHERE max_ot_hours_per_month IS NOT NULL;


-- 4. Migrate department_payroll_configs -> workspace_config_entries
-- std_working_days -> fixed_std_days
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
SELECT d.workspace_id, 'department', c.department_id, 'payroll.fixed_std_days', c.std_working_days, c.effective_date
FROM department_payroll_configs c
JOIN departments d ON c.department_id = d.id;

-- hours_per_day
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
SELECT d.workspace_id, 'department', c.department_id, 'payroll.hours_per_day', c.hours_per_day, c.effective_date
FROM department_payroll_configs c
JOIN departments d ON c.department_id = d.id;


-- 5. Migrate employee_payroll_configs -> workspace_config_entries
-- std_working_days -> fixed_std_days
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
SELECT e.workspace_id, 'employee', c.employee_id, 'payroll.fixed_std_days', c.std_working_days, c.effective_date
FROM employee_payroll_configs c
JOIN employees e ON c.employee_id = e.id;

-- hours_per_day
INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
SELECT e.workspace_id, 'employee', c.employee_id, 'payroll.hours_per_day', c.hours_per_day, c.effective_date
FROM employee_payroll_configs c
JOIN employees e ON c.employee_id = e.id;


-- 6. Drop legacy tables
DROP TABLE IF EXISTS employee_payroll_configs;
DROP TABLE IF EXISTS department_payroll_configs;
DROP TABLE IF EXISTS payroll_settings;
