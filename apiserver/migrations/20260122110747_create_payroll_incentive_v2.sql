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
