-- Up Migration: Create leave_types reference table

-- 1. Create leave_types table (workspace-scoped reference table)
CREATE TABLE IF NOT EXISTS leave_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL,
    name TEXT NOT NULL,
    is_paid BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Ensure unique constraint exists (handles case where table exists but constraint doesn't)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'leave_types_workspace_id_code_key'
    ) THEN
        ALTER TABLE leave_types ADD CONSTRAINT leave_types_workspace_id_code_key UNIQUE (workspace_id, code);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_leave_types_workspace_id ON leave_types(workspace_id);

-- 2. Seed default leave types for all existing workspaces
INSERT INTO leave_types (workspace_id, code, name, is_paid)
SELECT w.id, lt.code, lt.name, lt.is_paid
FROM workspaces w
CROSS JOIN (VALUES
    ('vacation', 'Vacation', TRUE),
    ('sick', 'Sick Leave', TRUE),
    ('personal', 'Personal Leave', TRUE),
    ('unpaid', 'Unpaid Leave', FALSE),
    ('maternity', 'Maternity Leave', TRUE),
    ('paternity', 'Paternity Leave', TRUE)
) AS lt(code, name, is_paid)
ON CONFLICT (workspace_id, code) DO NOTHING;

-- 3. Add leave_type_id columns to existing tables (nullable initially for migration)
ALTER TABLE leave_requests ADD COLUMN IF NOT EXISTS leave_type_id UUID REFERENCES leave_types(id) ON DELETE RESTRICT;
ALTER TABLE leave_accrual_logs ADD COLUMN IF NOT EXISTS leave_type_id UUID REFERENCES leave_types(id) ON DELETE RESTRICT;

-- 4. Migrate existing VARCHAR values to FK references
UPDATE leave_requests
SET leave_type_id = lt.id
FROM leave_types lt, employees e
WHERE e.id = leave_requests.employee_id
  AND lt.workspace_id = e.workspace_id
  AND lt.code = leave_requests.leave_type
  AND leave_requests.leave_type_id IS NULL;

UPDATE leave_accrual_logs
SET leave_type_id = lt.id
FROM leave_types lt
WHERE lt.workspace_id = leave_accrual_logs.workspace_id
  AND lt.code = leave_accrual_logs.leave_type
  AND leave_accrual_logs.leave_type_id IS NULL;

-- 5. Create indexes for new FK columns
CREATE INDEX IF NOT EXISTS idx_leave_requests_leave_type_id ON leave_requests(leave_type_id);
CREATE INDEX IF NOT EXISTS idx_leave_accrual_logs_leave_type_id ON leave_accrual_logs(leave_type_id);

-- Note: Legacy VARCHAR columns kept for backward compatibility
-- Future migration can drop them after full application migration

-- Down Migration (commented):
-- DROP INDEX IF EXISTS idx_leave_accrual_logs_leave_type_id;
-- DROP INDEX IF EXISTS idx_leave_requests_leave_type_id;
-- ALTER TABLE leave_accrual_logs DROP COLUMN leave_type_id;
-- ALTER TABLE leave_requests DROP COLUMN leave_type_id;
-- DROP TABLE leave_types;
