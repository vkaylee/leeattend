CREATE TABLE IF NOT EXISTS payslip_rule_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payslip_id UUID NOT NULL REFERENCES payslips(id) ON DELETE CASCADE,
    rule_code TEXT NOT NULL,
    amount DECIMAL(15, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_payslip_rule_evaluations_payslip_id
    ON payslip_rule_evaluations(payslip_id);
