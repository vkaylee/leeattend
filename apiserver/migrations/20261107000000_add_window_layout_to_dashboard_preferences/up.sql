BEGIN;

ALTER TABLE public.dashboard_preferences
    ADD COLUMN IF NOT EXISTS window_layout jsonb DEFAULT '{}'::jsonb NOT NULL;

COMMIT;
