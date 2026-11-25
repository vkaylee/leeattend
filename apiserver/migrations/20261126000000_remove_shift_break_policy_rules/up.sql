BEGIN;

ALTER TABLE public.shift_default_intervals
    DROP CONSTRAINT IF EXISTS shift_default_intervals_break_policy_check,
    DROP CONSTRAINT IF EXISTS shift_default_intervals_break_credit_mode_check,
    DROP COLUMN IF EXISTS is_break_auto_deduct,
    DROP COLUMN IF EXISTS break_is_overtime,
    DROP COLUMN IF EXISTS break_credit_mode;

INSERT INTO public.schema_migrations(version)
VALUES ('20261126000000_remove_shift_break_policy_rules');

COMMIT;
