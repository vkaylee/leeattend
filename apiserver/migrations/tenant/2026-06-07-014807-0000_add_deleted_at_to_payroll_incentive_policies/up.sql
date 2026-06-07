ALTER TABLE payroll_incentive_policies ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

CREATE INDEX idx_payroll_incentive_policies_deleted_at ON payroll_incentive_policies (workspace_id, deleted_at) WHERE deleted_at IS NULL;
