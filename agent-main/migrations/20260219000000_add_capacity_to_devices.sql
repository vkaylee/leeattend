-- Add capacity columns to devices table
ALTER TABLE devices ADD COLUMN users_capacity INTEGER DEFAULT 0;
ALTER TABLE devices ADD COLUMN logs_capacity INTEGER DEFAULT 0;
