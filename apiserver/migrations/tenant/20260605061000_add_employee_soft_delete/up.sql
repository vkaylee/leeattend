ALTER TABLE employees ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

DROP INDEX IF EXISTS idx_employees_workspace_employee_id;
CREATE UNIQUE INDEX idx_employees_workspace_employee_id ON employees(workspace_id, employee_id_number) WHERE deleted_at IS NULL;
