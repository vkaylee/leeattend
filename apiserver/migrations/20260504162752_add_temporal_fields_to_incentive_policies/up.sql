ALTER TABLE payroll_incentive_policies
ADD COLUMN effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
ADD COLUMN effective_to DATE;

CREATE INDEX idx_payroll_incentive_policies_temporal 
ON payroll_incentive_policies(workspace_id, effective_from, effective_to);
