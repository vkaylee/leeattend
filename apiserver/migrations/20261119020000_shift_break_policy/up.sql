BEGIN;

ALTER TABLE public.shift_default_intervals
    ADD COLUMN IF NOT EXISTS is_break_auto_deduct boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS break_is_overtime boolean NOT NULL DEFAULT false;

INSERT INTO public.schema_migrations(version)
VALUES ('20261119020000_shift_break_policy');

COMMIT;
