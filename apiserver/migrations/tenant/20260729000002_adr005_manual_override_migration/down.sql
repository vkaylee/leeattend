-- ADR-005 rollback: drop manual override columns.

DROP INDEX IF EXISTS public.idx_daily_summary_hr_review;
ALTER TABLE public.attendance_daily_summaries
    DROP COLUMN IF EXISTS manual_override_notes,
    DROP COLUMN IF EXISTS requires_hr_review,
    DROP COLUMN IF EXISTS manual_override_category,
    DROP COLUMN IF EXISTS is_manual_override;
