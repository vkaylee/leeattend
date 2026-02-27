-- Add holiday tracking to attendance_daily_summaries
ALTER TABLE attendance_daily_summaries
ADD COLUMN is_holiday BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN holiday_name VARCHAR(255);
