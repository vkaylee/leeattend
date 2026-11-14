BEGIN;

DELETE FROM public.schema_migrations
WHERE version = '20261114090000_payroll_calculation_run_policy_snapshot';

DROP INDEX IF EXISTS public.payroll_calculation_runs_policy_snapshot_idx;

COMMIT;
