-- Add partial day support to leave_requests
-- duration_type: 'full_day' (default), 'first_half', 'second_half', 'custom_hours'
-- start_time/end_time: only used when duration_type = 'custom_hours'

ALTER TABLE leave_requests
  ADD COLUMN IF NOT EXISTS duration_type VARCHAR(20) NOT NULL DEFAULT 'full_day',
  ADD COLUMN IF NOT EXISTS start_time TIME DEFAULT NULL,
  ADD COLUMN IF NOT EXISTS end_time TIME DEFAULT NULL;
