-- Up Migration: Update default pairing_algorithm to 'anchor_first'
ALTER TABLE workspaces ALTER COLUMN pairing_algorithm SET DEFAULT 'anchor_first';

-- Update existing workspaces that still use the old default if needed
-- The user seems to want 'anchor_first' as the new standard
UPDATE workspaces SET pairing_algorithm = 'anchor_first' WHERE pairing_algorithm = 'dp_pattern';
