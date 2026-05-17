-- Drop the assignments table
DROP TABLE IF EXISTS public.payroll_incentive_assignments CASCADE;

-- Revert the tiers table changes
ALTER TABLE public.payroll_incentive_tiers DROP COLUMN rule_config;

ALTER TABLE public.payroll_incentive_tiers ADD COLUMN max_late_early_count integer;

ALTER TABLE public.payroll_incentive_tiers ADD COLUMN max_unpaid_leave_days double precision;

ALTER TABLE public.payroll_incentive_tiers ADD COLUMN max_sick_leave_days double precision;

ALTER TABLE public.payroll_incentive_tiers ADD COLUMN max_total_leave_days double precision;