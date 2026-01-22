-- Payroll Incentive Policies
CREATE TABLE payroll_incentive_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    name TEXT NOT NULL,
    base_amount DOUBLE PRECISION NOT NULL,
    payout_type TEXT NOT NULL DEFAULT 'fixed', -- 'fixed' or 'percentage'
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Incentive Tiers (Thresholds)
CREATE TABLE payroll_incentive_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL,
    priority INTEGER NOT NULL, -- evaluation order (lower first)
    max_late_early_count INTEGER,
    max_unpaid_leave_days DOUBLE PRECISION,
    max_sick_leave_days DOUBLE PRECISION,
    max_total_leave_days DOUBLE PRECISION,
    payout_amount DOUBLE PRECISION NOT NULL, -- fixed value or percentage (0.0 - 1.0)
    FOREIGN KEY (policy_id) REFERENCES payroll_incentive_policies(id) ON DELETE CASCADE
);
