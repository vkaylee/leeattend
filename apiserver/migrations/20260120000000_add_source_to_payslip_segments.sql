-- Add source traceability to payslip segments
ALTER TABLE payslip_segments ADD COLUMN IF NOT EXISTS std_days_source VARCHAR(50);
-- Backfill existing data with 'unknown' or 'workspace' if preferred, but nullable or default is fine for now. 
-- Since it's a new feature, we can leave it null for old records.
