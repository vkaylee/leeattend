-- down.sql
DROP INDEX IF EXISTS idx_employers_parent_id;
ALTER TABLE employers DROP COLUMN IF EXISTS parent_id;
ALTER TABLE employers DROP COLUMN IF EXISTS relationship_type;
