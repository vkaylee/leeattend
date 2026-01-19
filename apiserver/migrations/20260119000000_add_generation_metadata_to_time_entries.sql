-- Up Migration: Add generation_metadata to time_entries
ALTER TABLE time_entries ADD COLUMN generation_metadata JSONB DEFAULT '{}'::jsonb;

-- Down Migration
-- ALTER TABLE time_entries DROP COLUMN generation_metadata;
