BEGIN;

ALTER TABLE public.dashboard_preferences
    DROP COLUMN IF EXISTS window_layout;

COMMIT;
