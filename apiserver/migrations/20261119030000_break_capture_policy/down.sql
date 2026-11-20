BEGIN;

DROP INDEX IF EXISTS attendance_action_idempotency_created_idx;
DROP TABLE IF EXISTS public.attendance_action_idempotency;
DROP INDEX IF EXISTS time_entry_breaks_one_open_idx;
DROP INDEX IF EXISTS time_entry_breaks_time_entry_idx;
DROP TABLE IF EXISTS public.time_entry_breaks;

ALTER TABLE public.attendance_daily_summaries
    DROP CONSTRAINT IF EXISTS attendance_daily_summaries_break_exception_check,
    DROP CONSTRAINT IF EXISTS attendance_daily_summaries_break_minutes_check,
    DROP COLUMN IF EXISTS scheduled_break_minutes,
    DROP COLUMN IF EXISTS actual_break_minutes,
    DROP COLUMN IF EXISTS deducted_break_minutes,
    DROP COLUMN IF EXISTS net_work_minutes,
    DROP COLUMN IF EXISTS net_paid_minutes,
    DROP COLUMN IF EXISTS break_provisional,
    DROP COLUMN IF EXISTS break_exception_code;

ALTER TABLE public.shift_default_intervals
    DROP CONSTRAINT IF EXISTS shift_default_intervals_break_policy_check,
    DROP CONSTRAINT IF EXISTS shift_default_intervals_break_credit_mode_check,
    DROP CONSTRAINT IF EXISTS shift_default_intervals_break_punch_mode_check,
    DROP COLUMN IF EXISTS break_credit_mode,
    DROP COLUMN IF EXISTS break_punch_mode;

DELETE FROM public.schema_migrations
WHERE version = '20261119030000_break_capture_policy';

COMMIT;
