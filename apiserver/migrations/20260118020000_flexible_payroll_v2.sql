-- Up Migration: Flexible Payroll System V2

-- 1. Add std_working_days to departments
ALTER TABLE departments ADD COLUMN IF NOT EXISTS std_working_days DECIMAL(5, 2);

-- 2. Create working_conditions table
CREATE TABLE IF NOT EXISTS working_conditions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL, -- 'EARLY_LEAVE', 'LATE_IN'
    minute_allowance INTEGER NOT NULL, -- e.g., 60 minutes
    is_paid BOOLEAN NOT NULL DEFAULT FALSE, -- If true, the allowance time counts as paid work hours
    start_date DATE NOT NULL,
    end_date DATE, -- Nullable (Indefinite)
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_working_conditions_employee_date ON working_conditions(employee_id, start_date);

-- 3. Create payslip_segments table
CREATE TABLE IF NOT EXISTS payslip_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payslip_id UUID NOT NULL REFERENCES payslips(id) ON DELETE CASCADE,
    
    -- Segment Context
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    department_id UUID REFERENCES departments(id),
    
    -- Snapshot Parameters
    applied_base_salary DECIMAL(15, 2) NOT NULL,
    applied_std_days DECIMAL(5, 2) NOT NULL,
    
    -- Calculated Values
    actual_work_days DECIMAL(5, 2) NOT NULL DEFAULT 0,
    paid_work_days DECIMAL(5, 2) NOT NULL DEFAULT 0,
    amount DECIMAL(15, 2) NOT NULL,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_payslip_segments_payslip_id ON payslip_segments(payslip_id);
