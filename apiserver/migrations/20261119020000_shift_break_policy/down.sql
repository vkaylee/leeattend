BEGIN;

ALTER TABLE public.shift_default_intervals
    DROP COLUMN IF EXISTS is_break_auto_deduct,
    DROP COLUMN IF EXISTS break_is_overtime;

DELETE FROM public.schema_migrations
WHERE version = '20261119020000_shift_break_policy';

COMMIT;
