-- Up Migration: Drop overly restrictive unique constraint
DROP INDEX IF EXISTS idx_leave_policy_definitions_ws_type;

-- We can instead ensure unique policy names per workspace if desired, but for now just ID is enough.
CREATE UNIQUE INDEX IF NOT EXISTS idx_leave_policy_definitions_ws_name
ON leave_policy_definitions(workspace_id, name);
