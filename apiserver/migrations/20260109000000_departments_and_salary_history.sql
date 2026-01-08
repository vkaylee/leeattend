-- Up Migration: Departments and Salary History

-- 1. Create departments table
CREATE TABLE IF NOT EXISTS departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    manager_id UUID REFERENCES employees(id) ON DELETE SET NULL, -- Department head
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_departments_workspace_id ON departments(workspace_id);

-- 2. Create employee_department_history table
CREATE TABLE IF NOT EXISTS employee_department_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    department_id UUID NOT NULL REFERENCES departments(id) ON DELETE CASCADE,
    effective_date DATE NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_emp_dept_hist_employee_id ON employee_department_history(employee_id);

-- 3. Transition salary_settings to salary_history
-- First, create the new table
CREATE TABLE IF NOT EXISTS salary_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    base_salary DECIMAL(15, 2) NOT NULL,
    salary_type VARCHAR(20) NOT NULL DEFAULT 'monthly',
    currency VARCHAR(10) NOT NULL DEFAULT 'VND',
    effective_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Migrate existing data from salary_settings
INSERT INTO salary_history (employee_id, base_salary, salary_type, currency, effective_date)
SELECT employee_id, base_salary, salary_type, currency, created_at::DATE
FROM salary_settings;

-- Drop the old table
DROP TABLE salary_settings;

CREATE INDEX IF NOT EXISTS idx_salary_history_employee_id ON salary_history(employee_id);
CREATE INDEX IF NOT EXISTS idx_salary_history_effective_date ON salary_history(effective_date);
