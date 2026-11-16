BEGIN;

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
