BEGIN;

DELETE FROM public.schema_migrations
WHERE version = '20261113030000_payroll_input_registry';

DROP TABLE IF EXISTS public.payroll_inputs;
DROP TABLE IF EXISTS public.payroll_input_batches;
DROP TABLE IF EXISTS public.payroll_metric_definitions;

COMMIT;
