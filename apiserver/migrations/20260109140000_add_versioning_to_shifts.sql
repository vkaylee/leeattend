-- Up Migration: Add versioning support to shifts
ALTER TABLE shifts ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE shifts ADD COLUMN IF NOT EXISTS parent_shift_id UUID REFERENCES shifts(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_shifts_parent_id ON shifts(parent_shift_id);
CREATE INDEX IF NOT EXISTS idx_shifts_active ON shifts(is_active);
