-- Add maximum overtime hours configuration to payroll_settings
-- Follows Vietnam Labor Code 2019 (Article 107): max 4h/day, 40h/month
-- NULL = no limit (for companies that don't apply caps)

ALTER TABLE payroll_settings
  ADD COLUMN IF NOT EXISTS max_ot_hours_per_day NUMERIC(4,1) DEFAULT 4.0,
  ADD COLUMN IF NOT EXISTS max_ot_hours_per_month NUMERIC(5,1) DEFAULT 40.0;
