-- Add workspace_id to time_entries
ALTER TABLE time_entries ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;

-- Backfill workspace_id from employees table
UPDATE time_entries
SET workspace_id = employees.workspace_id
FROM employees
WHERE time_entries.employee_id = employees.id;

-- Make workspace_id NOT NULL after backfill
ALTER TABLE time_entries ALTER COLUMN workspace_id SET NOT NULL;

-- Add index for performance
CREATE INDEX IF NOT EXISTS idx_time_entries_workspace_id ON time_entries(workspace_id);
