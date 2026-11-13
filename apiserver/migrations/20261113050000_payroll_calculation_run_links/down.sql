BEGIN;

DELETE FROM public.schema_migrations
WHERE version = '20261113050000_payroll_calculation_run_links';

DROP INDEX IF EXISTS public.payroll_calculation_runs_payslip_idx;
ALTER TABLE public.payroll_calculation_runs DROP COLUMN IF EXISTS payslip_id;
ALTER TABLE public.payroll_formula_rule_versions DROP COLUMN IF EXISTS retired_at;

COMMIT;
