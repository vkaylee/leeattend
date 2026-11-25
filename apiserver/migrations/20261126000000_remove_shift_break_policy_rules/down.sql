BEGIN;

ALTER TABLE public.shift_default_intervals
    ADD COLUMN IF NOT EXISTS is_break_auto_deduct boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS break_is_overtime boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS break_credit_mode text NOT NULL DEFAULT 'scheduled';

UPDATE public.shift_default_intervals
SET break_credit_mode = CASE
    WHEN break_punch_mode = 'required' THEN 'actual'
    WHEN is_break_auto_deduct THEN 'scheduled'
    ELSE 'none'
END;

ALTER TABLE public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_break_credit_mode_check
        CHECK (break_credit_mode IN ('scheduled', 'actual', 'none')),
    ADD CONSTRAINT shift_default_intervals_break_policy_check
        CHECK (NOT (break_punch_mode = 'required' AND break_credit_mode = 'scheduled'));

DELETE FROM public.schema_migrations
WHERE version = '20261126000000_remove_shift_break_policy_rules';

COMMIT;
