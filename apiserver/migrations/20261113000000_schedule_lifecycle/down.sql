BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.schedule_previews)
       OR EXISTS (SELECT 1 FROM public.schedule_publication_batches)
       OR EXISTS (SELECT 1 FROM public.schedule_day_revisions)
       OR EXISTS (SELECT 1 FROM public.attendance_periods)
       OR EXISTS (SELECT 1 FROM public.attendance_closures)
       OR EXISTS (SELECT 1 FROM public.attendance_closure_days)
       OR EXISTS (SELECT 1 FROM public.work_segments WHERE schedule_revision_id IS NOT NULL) THEN
        RAISE EXCEPTION 'schedule lifecycle migration rollback blocked: lifecycle evidence exists';
    END IF;
END;
$$;

DROP TRIGGER trg_schedule_work_segment_guard ON public.work_segments;
DROP FUNCTION public.guard_schedule_work_segment();
DROP TRIGGER trg_schedule_day_revision_guard ON public.schedule_day_revisions;
DROP FUNCTION public.guard_schedule_day_revision();

DROP TRIGGER trg_attendance_closure_days_immutable ON public.attendance_closure_days;
DROP TRIGGER trg_attendance_closures_immutable ON public.attendance_closures;
DROP FUNCTION public.guard_attendance_closure_immutable();

DROP TRIGGER trg_attendance_periods_updated_at ON public.attendance_periods;
DROP TRIGGER trg_schedule_publication_batches_updated_at ON public.schedule_publication_batches;

DROP INDEX public.outbox_jobs_type_status_available_idx;
DROP INDEX public.outbox_jobs_dedupe_key_uq;
ALTER TABLE ONLY public.outbox_jobs
    DROP CONSTRAINT outbox_jobs_attempts_check;
ALTER TABLE ONLY public.outbox_jobs
    DROP COLUMN dead_letter_at,
    DROP COLUMN last_error,
    DROP COLUMN available_at,
    DROP COLUMN attempts,
    DROP COLUMN locked_until,
    DROP COLUMN lease_token,
    DROP COLUMN dedupe_key;

DROP INDEX public.work_segments_official_interval_uq;
DROP INDEX public.work_segments_schedule_revision_sort_idx;
DROP INDEX public.work_segments_legacy_interval_uq;
CREATE UNIQUE INDEX idx_work_segments_interval_unique
    ON public.work_segments (employment_id, date, interval_id)
    WHERE interval_id IS NOT NULL;
ALTER TABLE ONLY public.work_segments
    DROP CONSTRAINT work_segments_breaks_snapshot_check,
    DROP CONSTRAINT work_segments_schedule_values_check,
    DROP CONSTRAINT work_segments_source_check,
    ADD CONSTRAINT work_segments_source_check CHECK (((source)::text = ANY (ARRAY['roster'::text, 'pattern'::text, 'rule'::text, 'fixed'::text, 'manual'::text, 'auto'::text]))),
    DROP CONSTRAINT work_segments_selected_shift_version_fkey,
    DROP CONSTRAINT work_segments_schedule_revision_fkey,
    DROP COLUMN breaks_snapshot,
    DROP COLUMN scheduled_end_at,
    DROP COLUMN scheduled_start_at,
    DROP COLUMN selected_shift_version_id,
    DROP COLUMN schedule_revision_id;

ALTER TABLE ONLY public.attendance_periods
    DROP CONSTRAINT attendance_periods_current_closure_fkey;
DROP TABLE public.attendance_closure_days;
DROP TABLE public.attendance_closures;
DROP TABLE public.attendance_periods;

DROP INDEX public.schedule_day_revisions_employer_date_idx;
DROP INDEX public.schedule_day_revisions_employment_date_revision_idx;
DROP INDEX public.schedule_day_revisions_legacy_uq;
DROP INDEX public.schedule_day_revisions_current_uq;
DROP TABLE public.schedule_day_revisions;

DROP INDEX public.schedule_publication_batches_created_at_idx;
DROP INDEX public.schedule_publication_batches_range_mode_idx;
DROP TABLE public.schedule_publication_batches;

DROP INDEX public.schedule_previews_expires_at_idx;
DROP INDEX public.schedule_previews_requested_by_created_at_idx;
DROP TABLE public.schedule_previews;

DELETE FROM public.schema_migrations
WHERE version = '20261113000000_schedule_lifecycle';

COMMIT;
