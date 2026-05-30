-- Restore holiday config columns to workspaces table
ALTER TABLE public.workspaces ADD COLUMN IF NOT EXISTS holiday_overnight_strategy character varying(30) DEFAULT 'split_at_midnight'::character varying NOT NULL;
ALTER TABLE public.workspaces ADD COLUMN IF NOT EXISTS default_holiday_ot_multiplier numeric(3,1) DEFAULT 3.0 NOT NULL;
