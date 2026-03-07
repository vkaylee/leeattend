-- Drop pairing_algorithm column from workspaces
-- This column is no longer used since the shift-aware auto-adaptive strategy was introduced.
ALTER TABLE workspaces DROP COLUMN IF EXISTS pairing_algorithm;
