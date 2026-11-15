BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.employment_relationships
        WHERE tenure_id IS NOT NULL
    ) OR EXISTS (
        SELECT 1
        FROM public.employee_tenures
    ) THEN
        RAISE EXCEPTION
            'cannot rollback employee tenure migration: tenure data is present; restore backup or roll forward';
    END IF;
END
$$;

ALTER TABLE public.employment_relationships
    DROP CONSTRAINT employment_relationships_tenure_identity_fkey;

DROP INDEX IF EXISTS public.idx_employment_relationships_tenure_id;
ALTER TABLE public.employment_relationships
    DROP COLUMN tenure_id;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employee_tenure_reconciliation;
DROP TABLE public.employee_tenure_reconciliation;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employee_tenures;
DROP TABLE public.employee_tenures;

DELETE FROM public.schema_migrations
WHERE version = '20261115030000_employee_tenures';

COMMIT;
