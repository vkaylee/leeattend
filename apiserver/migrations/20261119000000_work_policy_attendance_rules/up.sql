BEGIN;

ALTER TABLE public.work_credit_policies
    ADD COLUMN late_arrival_grace_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN early_ot_enabled boolean NOT NULL DEFAULT false,
    ADD COLUMN early_ot_qualification_grace_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN early_ot_max_minutes integer,
    ADD COLUMN break_auto_deduct boolean NOT NULL DEFAULT true,
    ADD COLUMN break_is_overtime boolean NOT NULL DEFAULT false,
    ADD COLUMN ot_aggregation_mode text NOT NULL DEFAULT 'aggregate',
    ADD CONSTRAINT work_credit_policies_late_grace_check CHECK (late_arrival_grace_minutes >= 0),
    ADD CONSTRAINT work_credit_policies_early_grace_check CHECK (early_ot_qualification_grace_minutes >= 0),
    ADD CONSTRAINT work_credit_policies_early_max_check CHECK (early_ot_max_minutes IS NULL OR early_ot_max_minutes >= 0),
    ADD CONSTRAINT work_credit_policies_ot_aggregation_check CHECK (ot_aggregation_mode IN ('aggregate', 'per_segment'));

ALTER TABLE public.shift_default_intervals
    DROP CONSTRAINT IF EXISTS shift_default_intervals_early_punch_in_grace_period_mi_not_null;

ALTER TABLE public.shift_default_intervals
    DROP COLUMN early_punch_in_grace_period_minutes,
    DROP COLUMN allow_early_ot,
    DROP COLUMN max_early_ot_minutes,
    DROP COLUMN ot_aggregation_mode,
    DROP COLUMN is_break_auto_deduct,
    DROP COLUMN break_is_overtime,
    DROP COLUMN is_night_shift;

INSERT INTO public.schema_migrations(version)
VALUES ('20261119000000_work_policy_attendance_rules');

COMMIT;
