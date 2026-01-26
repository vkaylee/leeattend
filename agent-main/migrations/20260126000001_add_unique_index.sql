CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_log ON zkt_raw_attendance(device_id, user_id, timestamp);
