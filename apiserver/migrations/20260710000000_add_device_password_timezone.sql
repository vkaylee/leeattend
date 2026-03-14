-- Add per-device password and timezone to devices table
ALTER TABLE devices ADD COLUMN IF NOT EXISTS password VARCHAR(50);
ALTER TABLE devices ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) DEFAULT 'UTC';
