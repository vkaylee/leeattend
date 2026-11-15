BEGIN;

DROP INDEX IF EXISTS public.payroll_calculation_runs_policy_snapshot_idx;

INSERT INTO public.schema_migrations(version)
VALUES ('20261115050000_append_only_payroll_policy_snapshots');

COMMIT;
