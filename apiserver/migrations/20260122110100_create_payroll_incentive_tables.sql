-- Payroll Incentive Policies
CREATE TABLE payroll_incentive_policies (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    name TEXT NOT NULL,
    base_amount REAL NOT NULL,
    payout_type TEXT NOT NULL DEFAULT 'fixed', -- 'fixed' or 'percentage'
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Incentive Tiers (Thresholds)
CREATE TABLE payroll_incentive_tiers (
    id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL,
    priority INTEGER NOT NULL, -- evaluation order (lower first)
    max_late_early_count INTEGER,
    max_unpaid_leave_days REAL,
    max_sick_leave_days REAL,
    max_total_leave_days REAL,
    payout_amount REAL NOT NULL, -- fixed value or percentage (0.0 - 1.0)
    FOREIGN KEY (policy_id) REFERENCES payroll_incentive_policies(id) ON DELETE CASCADE
);