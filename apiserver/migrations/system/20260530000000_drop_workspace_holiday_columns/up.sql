-- Drop holiday config columns from workspaces table
ALTER TABLE public.workspaces DROP COLUMN IF EXISTS holiday_overnight_strategy;
ALTER TABLE public.workspaces DROP COLUMN IF EXISTS default_holiday_ot_multiplier;
