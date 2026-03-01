-- Add holiday_work_premium_amount to payslips for transparent holiday compensation
ALTER TABLE payslips
  ADD COLUMN holiday_work_premium_amount NUMERIC(15,2) NOT NULL DEFAULT 0;
