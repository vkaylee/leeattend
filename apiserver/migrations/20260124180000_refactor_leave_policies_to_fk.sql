-- Up Migration: Refactor Leave Policies to use Foreign Key and Add Config

-- 1. Add policy_config to leave_types
ALTER TABLE leave_types 
ADD COLUMN IF NOT EXISTS policy_config JSONB NOT NULL DEFAULT '{}'::jsonb;

-- 2. Add leave_type_id to leave_policy_definitions
ALTER TABLE leave_policy_definitions 
ADD COLUMN IF NOT EXISTS leave_type_id UUID REFERENCES leave_types(id) ON DELETE RESTRICT;

-- 3. Backfill leave_type_id based on string match
UPDATE leave_policy_definitions lpd
SET leave_type_id = lt.id
FROM leave_types lt
WHERE lpd.workspace_id = lt.workspace_id 
  AND lpd.leave_type = lt.code;

-- 4. Enforce NOT NULL and drop old column
ALTER TABLE leave_policy_definitions 
ALTER COLUMN leave_type_id SET NOT NULL;

ALTER TABLE leave_policy_definitions 
DROP COLUMN leave_type;

-- 6. Add Unique Constraint
CREATE UNIQUE INDEX IF NOT EXISTS idx_leave_policy_definitions_ws_type 
ON leave_policy_definitions(workspace_id, leave_type_id);
