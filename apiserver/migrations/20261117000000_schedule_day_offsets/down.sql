BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.work_segments
        WHERE schedule_revision_id IS NOT NULL
           OR start_day_offset IS NOT NULL
           OR end_day_offset IS NOT NULL
           OR shift_group_key IS NOT NULL
           OR start_utc_offset_minutes IS NOT NULL
           OR end_utc_offset_minutes IS NOT NULL
    ) OR EXISTS (
        SELECT 1
        FROM public.shift_default_intervals
        WHERE start_day_offset <> 0
           OR end_day_offset <> 0
           OR start_utc_offset_minutes IS NOT NULL
           OR end_utc_offset_minutes IS NOT NULL
    ) OR EXISTS (
        SELECT 1
        FROM public.schedule_day_revisions
        WHERE state IN ('published', 'explicit_off', 'superseded')
    ) THEN
        RAISE EXCEPTION 'schedule day offsets rollback blocked: official schedule or offset evidence exists';
    END IF;
END;
$$;

ALTER TABLE public.work_segments
    DROP CONSTRAINT IF EXISTS work_segments_utc_offsets_check,
    DROP CONSTRAINT IF EXISTS work_segments_day_offsets_check,
    DROP COLUMN IF EXISTS end_utc_offset_minutes,
    DROP COLUMN IF EXISTS start_utc_offset_minutes,
    DROP COLUMN IF EXISTS shift_group_key,
    DROP COLUMN IF EXISTS end_day_offset,
    DROP COLUMN IF EXISTS start_day_offset;

ALTER TABLE public.shift_default_intervals
    DROP CONSTRAINT IF EXISTS shift_default_intervals_utc_offsets_check,
    DROP CONSTRAINT IF EXISTS shift_default_intervals_day_offsets_check,
    DROP COLUMN IF EXISTS end_utc_offset_minutes,
    DROP COLUMN IF EXISTS start_utc_offset_minutes,
    DROP COLUMN IF EXISTS end_day_offset,
    DROP COLUMN IF EXISTS start_day_offset;

DELETE FROM public.schema_migrations
WHERE version = '20261117000000_schedule_day_offsets';

COMMIT;
