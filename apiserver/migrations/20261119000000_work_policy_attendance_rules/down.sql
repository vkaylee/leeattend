BEGIN;

ALTER TABLE public.shift_default_intervals
    ADD COLUMN is_break_auto_deduct boolean NOT NULL DEFAULT true,
    ADD COLUMN break_is_overtime boolean NOT NULL DEFAULT false,
    ADD COLUMN early_punch_in_grace_period_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN allow_early_ot boolean NOT NULL DEFAULT false,
    ADD COLUMN max_early_ot_minutes integer,
    ADD COLUMN ot_aggregation_mode text NOT NULL DEFAULT 'aggregate',
    ADD COLUMN is_night_shift boolean NOT NULL DEFAULT false;

ALTER TABLE public.work_credit_policies
    DROP CONSTRAINT IF EXISTS work_credit_policies_late_grace_check,
    DROP CONSTRAINT IF EXISTS work_credit_policies_early_grace_check,
    DROP CONSTRAINT IF EXISTS work_credit_policies_early_max_check,
    DROP CONSTRAINT IF EXISTS work_credit_policies_ot_aggregation_check,
    DROP COLUMN late_arrival_grace_minutes,
    DROP COLUMN early_ot_enabled,
    DROP COLUMN early_ot_qualification_grace_minutes,
    DROP COLUMN early_ot_max_minutes,
    DROP COLUMN break_auto_deduct,
    DROP COLUMN break_is_overtime,
    DROP COLUMN ot_aggregation_mode;

DELETE FROM public.schema_migrations
WHERE version = '20261119000000_work_policy_attendance_rules';

COMMIT;
