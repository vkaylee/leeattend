ALTER TABLE raw_attendance_logs
    DROP COLUMN IF EXISTS source_timestamp,
    DROP COLUMN IF EXISTS source_timezone,
    DROP COLUMN IF EXISTS timestamp_kind,
    DROP COLUMN IF EXISTS conversion_status,
    DROP COLUMN IF EXISTS conversion_error,
    DROP COLUMN IF EXISTS received_at;
