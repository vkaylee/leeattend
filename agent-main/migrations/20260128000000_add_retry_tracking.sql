-- Add retry tracking fields to zkt_raw_attendance
ALTER TABLE zkt_raw_attendance ADD COLUMN retry_count INTEGER DEFAULT 0 NOT NULL;

-- Create failed logs table (dead letter queue)
CREATE TABLE IF NOT EXISTS zkt_failed_logs (
    id INTEGER PRIMARY KEY,
    device_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    status INTEGER,
    punch INTEGER,
    failed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    failure_reason TEXT NOT NULL,
    retry_attempts INTEGER NOT NULL,
    first_attempt_at DATETIME,
    last_attempt_at DATETIME
);

CREATE INDEX IF NOT EXISTS idx_failed_logs_device ON zkt_failed_logs(device_id);
CREATE INDEX IF NOT EXISTS idx_failed_logs_failed_at ON zkt_failed_logs(failed_at);
