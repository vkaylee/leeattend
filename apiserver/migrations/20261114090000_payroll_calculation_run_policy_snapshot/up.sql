BEGIN;

CREATE UNIQUE INDEX IF NOT EXISTS payroll_calculation_runs_policy_snapshot_idx
    ON public.payroll_calculation_runs (payslip_id)
    WHERE payslip_id IS NOT NULL AND formula_version_id IS NULL;

INSERT INTO public.schema_migrations(version)
VALUES ('20261114090000_payroll_calculation_run_policy_snapshot');

COMMIT;
