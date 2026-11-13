BEGIN;

DELETE FROM public.schema_migrations
WHERE version = '20261114010000_payroll_calculation_run_idempotency';

ALTER TABLE public.payroll_formula_rule_versions
    DROP CONSTRAINT IF EXISTS payroll_formula_published_effective_no_overlap;

DROP INDEX IF EXISTS public.payroll_calculation_runs_formula_payslip_unique;

COMMIT;
