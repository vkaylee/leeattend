-- Up Migration: Create employment_periods table and migrate existing data

CREATE TABLE IF NOT EXISTS employment_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    hire_date DATE NOT NULL,
    termination_date DATE, -- NULL if currently active
    is_rehire BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_employment_periods_employee_id ON employment_periods(employee_id);

-- 1. Migrate existing data from employees table
-- For each employee, create their first employment period
INSERT INTO employment_periods (employee_id, hire_date, termination_date, is_rehire, created_at, updated_at)
SELECT id, hire_date, termination_date, FALSE, created_at, updated_at 
FROM employees;

-- 2. Remove columns from employees table to avoid duplication
-- We need to handle dependencies first if any (indexes or views)
ALTER TABLE employees DROP COLUMN hire_date;
ALTER TABLE employees DROP COLUMN termination_date;
