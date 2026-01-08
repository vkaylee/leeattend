-- Up Migration: Create payroll and shifts related tables

-- 1. Create shifts table
CREATE TABLE IF NOT EXISTS shifts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    break_duration_minutes INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_shifts_workspace_id ON shifts(workspace_id);

-- 2. Create employee_shifts table (Assigning employees to shifts)
CREATE TABLE IF NOT EXISTS employee_shifts (
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    shift_id UUID NOT NULL REFERENCES shifts(id) ON DELETE CASCADE,
    effective_date DATE NOT NULL, -- Date from which this shift assignment is valid
    PRIMARY KEY (employee_id, shift_id, effective_date)
);

-- 3. Create salary_settings table
CREATE TABLE IF NOT EXISTS salary_settings (
    employee_id UUID PRIMARY KEY REFERENCES employees(id) ON DELETE CASCADE,
    base_salary DECIMAL(15, 2) NOT NULL,
    salary_type VARCHAR(20) NOT NULL DEFAULT 'monthly', -- 'hourly', 'monthly'
    currency VARCHAR(10) NOT NULL DEFAULT 'VND',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 4. Create payslips table
CREATE TABLE IF NOT EXISTS payslips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    base_pay_amount DECIMAL(15, 2) NOT NULL,
    overtime_pay_amount DECIMAL(15, 2) NOT NULL DEFAULT 0,
    deductions_amount DECIMAL(15, 2) NOT NULL DEFAULT 0,
    net_pay_amount DECIMAL(15, 2) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- 'draft', 'published', 'paid'
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_payslips_employee_id ON payslips(employee_id);
CREATE INDEX IF NOT EXISTS idx_payslips_period ON payslips(period_start, period_end);
