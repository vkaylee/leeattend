-- Up Migration: Dynamic Payroll System

-- 1. Create payroll_settings table
CREATE TABLE IF NOT EXISTS payroll_settings (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    std_working_day_mode TEXT NOT NULL DEFAULT 'fixed', -- 'fixed', 'actual_month'
    fixed_std_days DECIMAL(5, 2) NOT NULL DEFAULT 26.0,
    working_days_per_week INTEGER NOT NULL DEFAULT 6, -- Used if mode is 'actual_month'
    hours_per_day DECIMAL(4, 2) NOT NULL DEFAULT 8.0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. Create salary_component_definitions table
CREATE TABLE IF NOT EXISTS salary_component_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    code VARCHAR(50) NOT NULL, -- e.g., 'BASE', 'MEAL', 'PHONE'
    is_base_for_ot BOOLEAN NOT NULL DEFAULT FALSE,
    is_taxable BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(workspace_id, code)
);

-- 3. Create employee_salary_components table
-- This stores the specific amounts assigned to employees for each component
CREATE TABLE IF NOT EXISTS employee_salary_components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    component_definition_id UUID NOT NULL REFERENCES salary_component_definitions(id) ON DELETE CASCADE,
    amount DECIMAL(15, 2) NOT NULL DEFAULT 0.0,
    effective_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_employee_salary_components_employee_id ON employee_salary_components(employee_id);
CREATE INDEX IF NOT EXISTS idx_employee_salary_components_effective_date ON employee_salary_components(effective_date);

-- 4. Initialize default payroll settings for existing workspaces
INSERT INTO payroll_settings (workspace_id)
SELECT id FROM workspaces
ON CONFLICT (workspace_id) DO NOTHING;

-- 5. Create a default 'BASE' component for each workspace and migrate data
-- This is a bit complex in pure SQL for multi-tenant, but we can do a basic migration
-- We'll create a 'BASE' component for each workspace that has employees with salary_history
INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable)
SELECT DISTINCT w.id, 'Lương cơ bản', 'BASE', TRUE, TRUE
FROM workspaces w
JOIN employees e ON e.workspace_id = w.id
JOIN salary_history sh ON sh.employee_id = e.id
ON CONFLICT (workspace_id, code) DO NOTHING;

-- Link existing salary_history to the new employee_salary_components
-- Note: This assumes only one 'BASE' definition per workspace created above
INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date, created_at)
SELECT 
    sh.employee_id, 
    scd.id, 
    sh.base_salary, 
    sh.effective_date,
    sh.created_at
FROM salary_history sh
JOIN employees e ON e.id = sh.employee_id
JOIN salary_component_definitions scd ON scd.workspace_id = e.workspace_id AND scd.code = 'BASE';
