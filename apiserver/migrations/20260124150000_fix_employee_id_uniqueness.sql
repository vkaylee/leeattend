-- Up Migration: Scope employee_id_number uniqueness to workspace

-- 1. Drop the explicit unique index
DROP INDEX IF EXISTS idx_employees_employee_id_number;

-- 2. Drop the implicit unique constraint
ALTER TABLE employees DROP CONSTRAINT IF EXISTS employees_employee_id_number_key;

-- 3. Create a new unique index scoped to the workspace
CREATE UNIQUE INDEX IF NOT EXISTS idx_employees_workspace_employee_id ON employees(workspace_id, employee_id_number);
