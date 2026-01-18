-- Up Migration: 3-Tier Temporal Payroll Settings

-- 1. Refactor payroll_settings to support temporal versions
-- Existing workspaces should have an effective_date starting from 2026-01-01 (or earlier)
ALTER TABLE payroll_settings DROP CONSTRAINT IF EXISTS payroll_settings_pkey;
ALTER TABLE payroll_settings ADD COLUMN IF NOT EXISTS effective_date DATE NOT NULL DEFAULT '2026-01-01';
-- Ensure we don't have duplicates before adding PK
-- For now, assume one row per workspace from previous migrations.
ALTER TABLE payroll_settings ADD PRIMARY KEY (workspace_id, effective_date);

-- 2. Create department_payroll_configs for versioned department overrides
CREATE TABLE IF NOT EXISTS department_payroll_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    department_id UUID NOT NULL REFERENCES departments(id) ON DELETE CASCADE,
    std_working_days DECIMAL(5, 2) NOT NULL,
    effective_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(department_id, effective_date)
);

-- 3. Create employee_payroll_configs for versioned individual special cases
CREATE TABLE IF NOT EXISTS employee_payroll_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    std_working_days DECIMAL(5, 2) NOT NULL,
    effective_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employee_id, effective_date)
);

-- 4. Migrate existing department overrides
-- Existing overrides are in departments.std_working_days
INSERT INTO department_payroll_configs (department_id, std_working_days, effective_date)
SELECT id, std_working_days, '2026-01-01'
FROM departments
WHERE std_working_days IS NOT NULL
ON CONFLICT DO NOTHING;

-- 5. Drop legacy column from departments (optional, but cleaner)
-- We will keep it for now but deprecate it in resolver logic.
-- ALTER TABLE departments DROP COLUMN std_working_days;
