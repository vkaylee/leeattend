-- Add standard_units and overtime_units to payslips
ALTER TABLE payslips ADD COLUMN standard_units INT NOT NULL DEFAULT 0;
ALTER TABLE payslips ADD COLUMN overtime_units INT NOT NULL DEFAULT 0;
