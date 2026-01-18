-- Up Migration: Refine Temporal Overrides
ALTER TABLE department_payroll_configs ADD COLUMN IF NOT EXISTS hours_per_day DECIMAL(5, 2);
ALTER TABLE employee_payroll_configs ADD COLUMN IF NOT EXISTS hours_per_day DECIMAL(5, 2);

-- Update existing records to default to 8 hours
UPDATE department_payroll_configs SET hours_per_day = 8.0 WHERE hours_per_day IS NULL;
UPDATE employee_payroll_configs SET hours_per_day = 8.0 WHERE hours_per_day IS NULL;

-- Make mandatory after migration
ALTER TABLE department_payroll_configs ALTER COLUMN hours_per_day SET NOT NULL;
ALTER TABLE employee_payroll_configs ALTER COLUMN hours_per_day SET NOT NULL;

-- Add recording of hours per day to payslip segments for auditability
ALTER TABLE payslip_segments ADD COLUMN IF NOT EXISTS applied_hours_per_day DECIMAL(5, 2);
UPDATE payslip_segments SET applied_hours_per_day = 8.0 WHERE applied_hours_per_day IS NULL;
ALTER TABLE payslip_segments ALTER COLUMN applied_hours_per_day SET NOT NULL;
