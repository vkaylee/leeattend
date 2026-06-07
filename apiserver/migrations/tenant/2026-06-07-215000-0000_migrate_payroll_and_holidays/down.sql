-- 1. Recreate legacy tables
CREATE TABLE payroll_settings (
    workspace_id UUID NOT NULL,
    std_working_day_mode TEXT NOT NULL,
    fixed_std_days NUMERIC(12,4) NOT NULL,
    working_days_per_week INT NOT NULL,
    hours_per_day NUMERIC(12,4) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    effective_date DATE NOT NULL,
    max_ot_hours_per_day NUMERIC(12,4),
    max_ot_hours_per_month NUMERIC(12,4),
    PRIMARY KEY (workspace_id, effective_date)
);

CREATE TABLE department_payroll_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    department_id UUID NOT NULL REFERENCES departments(id) ON DELETE CASCADE,
    std_working_days NUMERIC(12,4) NOT NULL,
    effective_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    hours_per_day NUMERIC(12,4) NOT NULL
);

CREATE TABLE employee_payroll_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    std_working_days NUMERIC(12,4) NOT NULL,
    effective_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    hours_per_day NUMERIC(12,4) NOT NULL
);

-- 2. Populate legacy tables
-- Restore payroll_settings
INSERT INTO payroll_settings (
    workspace_id,
    std_working_day_mode,
    fixed_std_days,
    working_days_per_week,
    hours_per_day,
    effective_date,
    max_ot_hours_per_day,
    max_ot_hours_per_month
)
SELECT 
    workspace_id,
    COALESCE(MAX(CASE WHEN config_key = 'payroll.std_working_day_mode' THEN value_text END), 'fixed') AS std_working_day_mode,
    COALESCE(MAX(CASE WHEN config_key = 'payroll.fixed_std_days' THEN value_decimal END), 26.0) AS fixed_std_days,
    COALESCE(MAX(CASE WHEN config_key = 'payroll.working_days_per_week' THEN value_int END), 5) AS working_days_per_week,
    COALESCE(MAX(CASE WHEN config_key = 'payroll.hours_per_day' THEN value_decimal END), 8.0) AS hours_per_day,
    effective_from AS effective_date,
    MAX(CASE WHEN config_key = 'payroll.max_ot_hours_per_day' THEN value_decimal END) AS max_ot_hours_per_day,
    MAX(CASE WHEN config_key = 'payroll.max_ot_hours_per_month' THEN value_decimal END) AS max_ot_hours_per_month
FROM workspace_config_entries
WHERE scope_type = 'workspace' AND config_key LIKE 'payroll.%'
GROUP BY workspace_id, effective_from;

-- Restore department_payroll_configs
INSERT INTO department_payroll_configs (
    department_id,
    std_working_days,
    hours_per_day,
    effective_date
)
SELECT 
    scope_id AS department_id,
    COALESCE(MAX(CASE WHEN config_key = 'payroll.fixed_std_days' THEN value_decimal END), 26.0) AS std_working_days,
    COALESCE(MAX(CASE WHEN config_key = 'payroll.hours_per_day' THEN value_decimal END), 8.0) AS hours_per_day,
    effective_from AS effective_date
FROM workspace_config_entries
WHERE scope_type = 'department' AND config_key LIKE 'payroll.%'
GROUP BY scope_id, effective_from;

-- Restore employee_payroll_configs
INSERT INTO employee_payroll_configs (
    employee_id,
    std_working_days,
    hours_per_day,
    effective_date
)
SELECT 
    scope_id AS employee_id,
    COALESCE(MAX(CASE WHEN config_key = 'payroll.fixed_std_days' THEN value_decimal END), 26.0) AS std_working_days,
    COALESCE(MAX(CASE WHEN config_key = 'payroll.hours_per_day' THEN value_decimal END), 8.0) AS hours_per_day,
    effective_from AS effective_date
FROM workspace_config_entries
WHERE scope_type = 'employee' AND config_key LIKE 'payroll.%'
GROUP BY scope_id, effective_from;

-- 3. Delete config entries
DELETE FROM workspace_config_entries WHERE config_key LIKE 'payroll.%';
DELETE FROM workspace_config_entries WHERE scope_type = 'employer';

-- 4. Revert holidays table changes
DROP INDEX IF EXISTS idx_holidays_workspace_location_date;
ALTER TABLE holidays DROP COLUMN IF EXISTS location_id;

-- 5. Revert scope check constraint
ALTER TABLE workspace_config_entries DROP CONSTRAINT IF EXISTS workspace_config_entries_scope_type_check;
ALTER TABLE workspace_config_entries ADD CONSTRAINT workspace_config_entries_scope_type_check 
    CHECK (scope_type IN ('workspace', 'department', 'employee'));
