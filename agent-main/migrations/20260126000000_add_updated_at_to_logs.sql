-- Add updated_at column to track syncing state changes
ALTER TABLE zkt_raw_attendance ADD COLUMN updated_at DATETIME DEFAULT CURRENT_TIMESTAMP;

-- Initialize updated_at for existing rows
UPDATE zkt_raw_attendance SET updated_at = created_at;
