BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.schedule_day_revisions
        WHERE state IN ('published', 'explicit_off', 'superseded')
    ) OR EXISTS (
        SELECT 1
        FROM public.config_entries
        WHERE scope_type = 'employer'
          AND config_key = 'schedule.lifecycle.enabled'
    ) THEN
        RAISE EXCEPTION 'schedule activation migration rollback blocked: activation or lifecycle evidence exists';
    END IF;
END;
$$;

ALTER TABLE public.config_entries
    DROP CONSTRAINT IF EXISTS config_entries_schedule_lifecycle_effective_no_overlap;

DELETE FROM public.schema_migrations
WHERE version = '20261114030000_schedule_activation_ranges';

COMMIT;
