-- Add break_start, break_end and is_break_auto_deduct to shifts table
ALTER TABLE shifts ADD COLUMN break_start TIME;
ALTER TABLE shifts ADD COLUMN break_end TIME;
ALTER TABLE shifts ADD COLUMN is_break_auto_deduct BOOLEAN NOT NULL DEFAULT TRUE;
