BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.schedule_shadow_mismatches) OR EXISTS (
        SELECT 1 FROM public.schedule_day_revisions
        WHERE state IN ('published', 'explicit_off', 'superseded')
    ) THEN
        RAISE EXCEPTION 'schedule shadow reconciliation rollback blocked: mismatch or lifecycle evidence exists';
    END IF;
END;
$$;

DROP INDEX public.schedule_shadow_mismatches_employment_date_idx;
DROP INDEX public.schedule_shadow_mismatches_employer_date_idx;
DROP INDEX public.schedule_shadow_mismatches_open_key;
DROP TABLE public.schedule_shadow_mismatches;

DELETE FROM public.schema_migrations
WHERE version = '20261114050000_schedule_shadow_reconciliation';

COMMIT;
