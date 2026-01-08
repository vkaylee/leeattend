-- Add is_break_overtime to track dynamic break-to-overtime cases
ALTER TABLE time_entries ADD COLUMN is_break_overtime BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE attendance_adjustment_requests ADD COLUMN is_break_overtime BOOLEAN NOT NULL DEFAULT FALSE;
