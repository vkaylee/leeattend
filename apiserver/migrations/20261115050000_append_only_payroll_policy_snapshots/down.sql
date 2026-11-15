BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.payroll_calculation_runs
        WHERE formula_version_id IS NULL
          AND payslip_id IS NOT NULL
        GROUP BY payslip_id
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION 'cannot rollback append-only policy snapshots: duplicate payslip history exists';
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS payroll_calculation_runs_policy_snapshot_idx
    ON public.payroll_calculation_runs (payslip_id)
    WHERE payslip_id IS NOT NULL AND formula_version_id IS NULL;

DELETE FROM public.schema_migrations
WHERE version = '20261115050000_append_only_payroll_policy_snapshots';

COMMIT;
