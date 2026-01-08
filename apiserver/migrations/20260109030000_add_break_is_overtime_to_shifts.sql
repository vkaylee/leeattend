-- Add break_is_overtime column to shifts table
ALTER TABLE shifts ADD COLUMN break_is_overtime BOOLEAN NOT NULL DEFAULT FALSE;
