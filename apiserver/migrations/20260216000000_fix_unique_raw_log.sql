-- Up Migration
ALTER TABLE raw_attendance_logs DROP CONSTRAINT IF EXISTS unique_raw_log;

-- Add punch and status columns with default -1
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_attribute WHERE attrelid = 'raw_attendance_logs'::regclass AND attname = 'punch') THEN
        ALTER TABLE raw_attendance_logs ADD COLUMN punch INTEGER NOT NULL DEFAULT -1;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_attribute WHERE attrelid = 'raw_attendance_logs'::regclass AND attname = 'status') THEN
        ALTER TABLE raw_attendance_logs ADD COLUMN status INTEGER NOT NULL DEFAULT -1;
    END IF;
END $$;

-- Populate columns from raw_data
UPDATE raw_attendance_logs SET 
    punch = COALESCE((raw_data->>'punch')::INTEGER, -1),
    status = COALESCE((raw_data->>'status')::INTEGER, -1);

-- Add unique constraint
ALTER TABLE raw_attendance_logs 
ADD CONSTRAINT unique_raw_log UNIQUE (workspace_id, employee_id, punch_time, punch, status);

-- Down Migration
-- ALTER TABLE raw_attendance_logs DROP CONSTRAINT unique_raw_log;
-- ALTER TABLE raw_attendance_logs ADD CONSTRAINT unique_raw_log UNIQUE (workspace_id, employee_id, punch_time);
