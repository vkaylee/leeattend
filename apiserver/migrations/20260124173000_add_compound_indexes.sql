-- Up Migration: Add compound performance indexes

-- 1. Compound index for common employee filtering pattern (workspace + active status)
-- Optimizes queries like: SELECT * FROM employees WHERE workspace_id = ? AND is_active = true
CREATE INDEX IF NOT EXISTS idx_employees_workspace_active ON employees(workspace_id, is_active);

-- Down Migration (commented):
-- DROP INDEX IF EXISTS idx_employees_workspace_active;
