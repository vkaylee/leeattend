-- Add early_punch_in_grace_period_minutes column to shifts table
ALTER TABLE shifts ADD COLUMN early_punch_in_grace_period_minutes INTEGER NOT NULL DEFAULT 0;
