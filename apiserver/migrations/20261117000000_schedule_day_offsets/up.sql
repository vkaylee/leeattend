BEGIN;

ALTER TABLE public.shift_default_intervals
    ADD COLUMN start_day_offset integer NOT NULL DEFAULT 0,
    ADD COLUMN end_day_offset integer NOT NULL DEFAULT 0,
    ADD COLUMN start_utc_offset_minutes integer,
    ADD COLUMN end_utc_offset_minutes integer;

ALTER TABLE public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_day_offsets_check
        CHECK (start_day_offset >= 0 AND end_day_offset >= start_day_offset),
    ADD CONSTRAINT shift_default_intervals_utc_offsets_check
        CHECK ((start_utc_offset_minutes IS NULL OR (start_utc_offset_minutes BETWEEN -840 AND 840 AND start_utc_offset_minutes % 15 = 0))
           AND (end_utc_offset_minutes IS NULL OR (end_utc_offset_minutes BETWEEN -840 AND 840 AND end_utc_offset_minutes % 15 = 0)));

ALTER TABLE public.work_segments
    ADD COLUMN start_day_offset integer,
    ADD COLUMN end_day_offset integer,
    ADD COLUMN shift_group_key varchar(128),
    ADD COLUMN start_utc_offset_minutes integer,
    ADD COLUMN end_utc_offset_minutes integer;

ALTER TABLE public.work_segments
    ADD CONSTRAINT work_segments_day_offsets_check
        CHECK (start_day_offset IS NULL OR (start_day_offset >= 0 AND end_day_offset >= start_day_offset)),
    ADD CONSTRAINT work_segments_utc_offsets_check
        CHECK ((start_utc_offset_minutes IS NULL OR (start_utc_offset_minutes BETWEEN -840 AND 840 AND start_utc_offset_minutes % 15 = 0))
           AND (end_utc_offset_minutes IS NULL OR (end_utc_offset_minutes BETWEEN -840 AND 840 AND end_utc_offset_minutes % 15 = 0)));

INSERT INTO public.schema_migrations(version)
VALUES ('20261117000000_schedule_day_offsets');

COMMIT;
