BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.schedule_day_revisions
        WHERE state <> 'legacy_unverified'
    ) OR EXISTS (
        SELECT 1
        FROM public.work_segments
        WHERE schedule_revision_id IS NOT NULL
    ) THEN
        RAISE EXCEPTION 'legacy schedule backfill rollback blocked: lifecycle evidence exists';
    END IF;
END;
$$;

DELETE FROM public.schedule_day_revisions
WHERE state = 'legacy_unverified'
  AND source = 'legacy_unverified'
  AND resolver_version = 'legacy-import-v1'
  AND source_snapshot->>'provenance' = 'legacy_unverified';

ALTER TABLE public.schedule_day_revisions
    ALTER COLUMN state TYPE varchar(16);

DELETE FROM public.schema_migrations
WHERE version = '20261114040000_schedule_legacy_backfill';

COMMIT;
