BEGIN;

DELETE FROM public.schema_migrations
WHERE version = '20261113040000_payroll_formula_versions';

DROP TABLE IF EXISTS public.payroll_calculation_runs;
DROP TABLE IF EXISTS public.payroll_formula_rule_versions;
DROP TABLE IF EXISTS public.payroll_formula_rules;

COMMIT;
