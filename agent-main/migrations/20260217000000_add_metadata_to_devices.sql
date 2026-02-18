-- Add metadata columns to devices table
ALTER TABLE devices ADD COLUMN mac TEXT;
ALTER TABLE devices ADD COLUMN serial TEXT;
ALTER TABLE devices ADD COLUMN firmware TEXT;
ALTER TABLE devices ADD COLUMN platform TEXT;
ALTER TABLE devices ADD COLUMN hw_name TEXT;
ALTER TABLE devices ADD COLUMN users_count INTEGER DEFAULT 0;
ALTER TABLE devices ADD COLUMN logs_count INTEGER DEFAULT 0;
ALTER TABLE devices ADD COLUMN last_info_update DATETIME;
