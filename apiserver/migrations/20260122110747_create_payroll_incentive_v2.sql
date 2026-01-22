-- Results of incentive evaluation for a payslip
CREATE TABLE payslip_incentives (
    id SERIAL PRIMARY KEY,
    payslip_id UUID NOT NULL,
    policy_name TEXT NOT NULL,
    amount DOUBLE PRECISION NOT NULL,
    tier_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (payslip_id) REFERENCES payslips(id) ON DELETE CASCADE
);

-- Fix types and constraints for policies
-- First, convert id and workspace_id to UUID if they were text
-- (We assume they were text from the very first attempt)
ALTER TABLE payroll_incentive_policies ALTER COLUMN id TYPE UUID USING id::uuid;
ALTER TABLE payroll_incentive_policies ALTER COLUMN workspace_id TYPE UUID USING workspace_id::uuid;
ALTER TABLE payroll_incentive_policies ALTER COLUMN base_amount TYPE DOUBLE PRECISION;
ALTER TABLE payroll_incentive_policies ALTER COLUMN is_active TYPE BOOLEAN USING (is_active::integer::boolean);
ALTER TABLE payroll_incentive_policies ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at::timestamptz;

-- Fix tiers
ALTER TABLE payroll_incentive_tiers ALTER COLUMN id TYPE UUID USING id::uuid;
ALTER TABLE payroll_incentive_tiers ALTER COLUMN policy_id TYPE UUID USING policy_id::uuid;
ALTER TABLE payroll_incentive_tiers ALTER COLUMN max_unpaid_leave_days TYPE DOUBLE PRECISION;
ALTER TABLE payroll_incentive_tiers ALTER COLUMN max_sick_leave_days TYPE DOUBLE PRECISION;
ALTER TABLE payroll_incentive_tiers ALTER COLUMN max_total_leave_days TYPE DOUBLE PRECISION;
ALTER TABLE payroll_incentive_tiers ALTER COLUMN payout_amount TYPE DOUBLE PRECISION;