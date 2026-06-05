DROP INDEX IF EXISTS idx_employees_workspace_employee_id;
CREATE UNIQUE INDEX idx_employees_workspace_employee_id ON employees(workspace_id, employee_id_number);

ALTER TABLE employees DROP COLUMN IF EXISTS deleted_at;
