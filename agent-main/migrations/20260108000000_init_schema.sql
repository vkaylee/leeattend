-- Create zkt_raw_attendance table
CREATE TABLE IF NOT EXISTS zkt_raw_attendance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    status INTEGER,
    punch INTEGER,
    is_synced INTEGER DEFAULT 0 NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_device_id ON zkt_raw_attendance(device_id);
CREATE INDEX IF NOT EXISTS idx_user_id ON zkt_raw_attendance(user_id);
CREATE INDEX IF NOT EXISTS idx_timestamp ON zkt_raw_attendance(timestamp);
CREATE INDEX IF NOT EXISTS idx_is_synced ON zkt_raw_attendance(is_synced);

-- Create devices table for local configuration cache
CREATE TABLE IF NOT EXISTS devices (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT,
    ip TEXT NOT NULL,
    port INTEGER DEFAULT 4370 NOT NULL,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    last_seen DATETIME,
    is_active BOOLEAN DEFAULT 1
);