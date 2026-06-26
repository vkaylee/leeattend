-- Reverse: Move time/break/policy config back from shift_default_intervals to shifts

-- Step 1: Recreate columns on shifts
ALTER TABLE public.shifts
    ADD COLUMN start_time time without time zone NOT NULL DEFAULT '08:00:00',
    ADD COLUMN end_time time without time zone NOT NULL DEFAULT '17:00:00',
    ADD COLUMN break_duration_minutes integer NOT NULL DEFAULT 60,
    ADD COLUMN break_start time without time zone,
    ADD COLUMN break_end time without time zone,
    ADD COLUMN is_break_auto_deduct boolean NOT NULL DEFAULT true,
    ADD COLUMN break_is_overtime boolean NOT NULL DEFAULT false,
    ADD COLUMN early_punch_in_grace_period_minutes integer NOT NULL DEFAULT 0,
    ADD COLUMN allow_early_ot boolean NOT NULL DEFAULT false,
    ADD COLUMN max_early_ot_minutes integer,
    ADD COLUMN ot_aggregation_mode public.ot_aggregation_mode_type DEFAULT 'aggregate' NOT NULL,
    ADD COLUMN is_night_shift boolean DEFAULT false NOT NULL;

-- Step 2: Copy data back from the first interval of each shift
UPDATE public.shifts s
SET
    start_time = sdi.start_time,
    end_time = sdi.end_time,
    break_duration_minutes = COALESCE(sdi.break_duration_minutes, 60),
    break_start = sdi.break_start,
    break_end = sdi.break_end,
    is_break_auto_deduct = COALESCE(sdi.is_break_auto_deduct, true),
    break_is_overtime = COALESCE(sdi.break_is_overtime, false),
    early_punch_in_grace_period_minutes = COALESCE(sdi.early_punch_in_grace_period_minutes, 0),
    allow_early_ot = COALESCE(sdi.allow_early_ot, false),
    max_early_ot_minutes = sdi.max_early_ot_minutes,
    ot_aggregation_mode = COALESCE(sdi.ot_aggregation_mode, 'aggregate'),
    is_night_shift = COALESCE(sdi.is_night_shift, false)
FROM public.shift_default_intervals sdi
WHERE sdi.shift_id = s.id
  AND sdi.sort_order = 0;

-- Step 3: Remove defaults on shifts columns (they were only needed for the ADD COLUMN)
ALTER TABLE public.shifts
    ALTER COLUMN start_time DROP DEFAULT,
    ALTER COLUMN end_time DROP DEFAULT,
    ALTER COLUMN break_duration_minutes DROP DEFAULT,
    ALTER COLUMN is_break_auto_deduct DROP DEFAULT,
    ALTER COLUMN break_is_overtime DROP DEFAULT,
    ALTER COLUMN early_punch_in_grace_period_minutes DROP DEFAULT,
    ALTER COLUMN allow_early_ot DROP DEFAULT,
    ALTER COLUMN ot_aggregation_mode DROP DEFAULT,
    ALTER COLUMN is_night_shift DROP DEFAULT;

-- Step 4: Revert interval columns to nullable
ALTER TABLE public.shift_default_intervals
    ALTER COLUMN break_duration_minutes DROP NOT NULL,
    ALTER COLUMN break_duration_minutes DROP DEFAULT,
    ALTER COLUMN is_break_auto_deduct DROP NOT NULL,
    ALTER COLUMN is_break_auto_deduct DROP DEFAULT,
    ALTER COLUMN break_is_overtime DROP NOT NULL,
    ALTER COLUMN break_is_overtime DROP DEFAULT;

-- Step 5: Drop new columns from shift_default_intervals
ALTER TABLE public.shift_default_intervals
    DROP COLUMN early_punch_in_grace_period_minutes,
    DROP COLUMN allow_early_ot,
    DROP COLUMN max_early_ot_minutes,
    DROP COLUMN ot_aggregation_mode,
    DROP COLUMN is_night_shift;
