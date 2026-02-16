-- Up Migration
DROP INDEX IF EXISTS idx_unique_log;
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_log ON zkt_raw_attendance(device_id, user_id, timestamp, punch, status);

-- Down Migration
-- DROP INDEX IF EXISTS idx_unique_log;
-- CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_log ON zkt_raw_attendance(device_id, user_id, timestamp);
