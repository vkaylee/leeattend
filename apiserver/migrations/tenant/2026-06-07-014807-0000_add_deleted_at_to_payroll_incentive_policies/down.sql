DROP INDEX IF EXISTS idx_payroll_incentive_policies_deleted_at;

ALTER TABLE payroll_incentive_policies DROP COLUMN IF EXISTS deleted_at;
