BEGIN;

DROP INDEX IF EXISTS attendance_daily_summaries_policy_provenance_idx;

ALTER TABLE public.attendance_daily_summaries
    DROP CONSTRAINT IF EXISTS attendance_daily_summaries_minutes_nonnegative,
    DROP COLUMN shift_version_id,
    DROP COLUMN work_policy_version_id;

DELETE FROM public.schema_migrations
WHERE version = '20261119010000_attendance_policy_provenance';

COMMIT;
