BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.employments
        WHERE relationship_id IS NOT NULL
    ) OR EXISTS (
        SELECT 1
        FROM public.employee_contracts
        WHERE relationship_id IS NOT NULL
    ) OR EXISTS (
        SELECT 1
        FROM public.employment_relationships
    ) THEN
        RAISE EXCEPTION
            'cannot rollback employment relationship backfill: relationship data is present; restore backup or roll forward';
    END IF;
END
$$;

DELETE FROM public.schema_migrations
WHERE version = '20261112020000_employment_relationship_backfill';

COMMIT;
