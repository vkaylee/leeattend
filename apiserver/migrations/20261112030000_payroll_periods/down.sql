BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.payroll_periods) THEN
        RAISE EXCEPTION 'cannot rollback payroll periods: period data is present; restore backup or roll forward';
    END IF;
END
$$;

DROP TRIGGER IF EXISTS trg_payroll_periods_updated_at ON public.payroll_periods;
DROP FUNCTION IF EXISTS public.fn_payroll_periods_updated_at();
DROP INDEX IF EXISTS public.payroll_periods_status_idx;
DROP TABLE IF EXISTS public.payroll_periods;

DELETE FROM public.schema_migrations
WHERE version = '20261112030000_payroll_periods';

COMMIT;
