-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_shifts_location_id;
DROP INDEX IF EXISTS idx_shifts_employer_id;

ALTER TABLE shifts DROP COLUMN IF EXISTS location_id;
ALTER TABLE shifts DROP COLUMN IF EXISTS employer_id;
