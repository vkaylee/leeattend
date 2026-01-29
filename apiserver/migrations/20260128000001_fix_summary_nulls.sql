-- Fix null constraints
ALTER TABLE attendance_daily_summaries ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE attendance_daily_summaries ALTER COLUMN updated_at SET NOT NULL;
