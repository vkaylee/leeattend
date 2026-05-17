DROP INDEX IF EXISTS idx_payroll_incentive_policies_temporal;

ALTER TABLE payroll_incentive_policies
DROP COLUMN effective_from,
DROP COLUMN effective_to;