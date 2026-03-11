-- Add workspace_id column to payslips for SaaS tenant isolation
-- payslips currently only references employee_id; adding workspace_id
-- enables direct workspace scoping without JOIN to employees

-- 1. Add nullable column first
ALTER TABLE payslips ADD COLUMN IF NOT EXISTS workspace_id UUID;

-- 2. Backfill from employees table
UPDATE payslips p
SET workspace_id = e.workspace_id
FROM employees e
WHERE p.employee_id = e.id
  AND p.workspace_id IS NULL;

-- 3. Set NOT NULL constraint
ALTER TABLE payslips ALTER COLUMN workspace_id SET NOT NULL;

-- 4. Add FK constraint
ALTER TABLE payslips ADD CONSTRAINT fk_payslips_workspace
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE;

-- 5. Add index for workspace_id queries
CREATE INDEX IF NOT EXISTS idx_payslips_workspace_id
  ON payslips(workspace_id);
