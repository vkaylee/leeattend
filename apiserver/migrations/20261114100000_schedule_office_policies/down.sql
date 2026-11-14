BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.schedule_office_policies) THEN
        RAISE EXCEPTION 'schedule office policy rollback blocked: policy evidence exists';
    END IF;
END;
$$;

DROP INDEX public.schedule_office_policies_approval_idx;
DROP INDEX public.schedule_office_policies_employer_effective_idx;
ALTER TABLE public.schedule_office_policies
    DROP CONSTRAINT schedule_office_policies_effective_no_overlap;
DROP TABLE public.schedule_office_policies;

DELETE FROM public.schema_migrations
WHERE version = '20261114100000_schedule_office_policies';

COMMIT;
