-- Up Migration
-- Add unique constraint to prevent duplicate punch logs for the same employee at the same time
ALTER TABLE raw_attendance_logs 
ADD CONSTRAINT unique_raw_log UNIQUE (workspace_id, employee_id, punch_time);

-- Down Migration
-- ALTER TABLE raw_attendance_logs DROP CONSTRAINT unique_raw_log;
