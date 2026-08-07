ALTER TABLE raw_attendance_logs
    ADD COLUMN source_timestamp text,
    ADD COLUMN source_timezone text,
    ADD COLUMN timestamp_kind text,
    ADD COLUMN conversion_status text,
    ADD COLUMN conversion_error text,
    ADD COLUMN received_at timestamptz;
