-- Rollback: remove scope_type column and its index
DROP INDEX IF EXISTS idx_workspace_roles_scope_type;
ALTER TABLE workspace_roles DROP COLUMN IF EXISTS scope_type;
