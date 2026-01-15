-- Add allow_early_ot to shifts table
ALTER TABLE shifts ADD COLUMN allow_early_ot BOOLEAN NOT NULL DEFAULT FALSE;
