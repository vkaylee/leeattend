-- ============================================================================
-- Move time/break/policy config from shifts to shift_default_intervals
--
-- After this migration:
--   - shift_default_intervals is the sole source of truth for all time,
--     break, and policy configuration
--   - Every shift will have at least 1 interval (auto-created for shifts
--     that previously had none)
--   - The shifts table retains only identity, scope, and grouping fields
-- ============================================================================

-- Step 1: Add new columns to shift_default_intervals
ALTER TABLE public.shift_default_intervals
    ADD COLUMN early_punch_in_grace_period_minutes integer DEFAULT 0 NOT NULL,
    ADD COLUMN allow_early_ot boolean DEFAULT false NOT NULL,
    ADD COLUMN max_early_ot_minutes integer,
    ADD COLUMN ot_aggregation_mode character varying(30) DEFAULT 'aggregate' NOT NULL,
    ADD COLUMN is_night_shift boolean DEFAULT false NOT NULL;

COMMENT ON COLUMN public.shift_default_intervals.early_punch_in_grace_period_minutes IS 'Grace period (minutes) before scheduled start without late penalty';
COMMENT ON COLUMN public.shift_default_intervals.allow_early_ot IS 'Whether work before scheduled start counts as overtime';
COMMENT ON COLUMN public.shift_default_intervals.max_early_ot_minutes IS 'Maximum minutes of early overtime allowed';
COMMENT ON COLUMN public.shift_default_intervals.ot_aggregation_mode IS 'aggregate=sum all segments then apply OT threshold, per_segment=OT calculated independently per segment';
COMMENT ON COLUMN public.shift_default_intervals.is_night_shift IS 'When true, interval hours crossing midnight are attributed to the start day';

-- Step 2: For shifts with NO existing intervals, INSERT a default interval
-- copying all time/break/policy fields from the shifts table
INSERT INTO public.shift_default_intervals (
    shift_id,
    sort_order,
    interval_name,
    start_time,
    end_time,
    break_duration_minutes,
    break_start,
    break_end,
    is_break_auto_deduct,
    break_is_overtime,
    early_punch_in_grace_period_minutes,
    allow_early_ot,
    max_early_ot_minutes,
    ot_aggregation_mode,
    is_night_shift
)
SELECT
    s.id AS shift_id,
    0 AS sort_order,
    NULL AS interval_name,
    s.start_time,
    s.end_time,
    s.break_duration_minutes,
    s.break_start,
    s.break_end,
    s.is_break_auto_deduct,
    s.break_is_overtime,
    s.early_punch_in_grace_period_minutes,
    s.allow_early_ot,
    s.max_early_ot_minutes,
    COALESCE(s.ot_aggregation_mode, 'aggregate'),
    COALESCE(s.is_night_shift, false)
FROM public.shifts s
WHERE NOT EXISTS (
    SELECT 1 FROM public.shift_default_intervals sdi WHERE sdi.shift_id = s.id
);

-- Step 3: For shifts that ALREADY have intervals, backfill any NULL
-- break/policy fields from the parent shift
UPDATE public.shift_default_intervals sdi
SET
    break_duration_minutes = COALESCE(sdi.break_duration_minutes, s.break_duration_minutes),
    break_start = COALESCE(sdi.break_start, s.break_start),
    break_end = COALESCE(sdi.break_end, s.break_end),
    is_break_auto_deduct = COALESCE(sdi.is_break_auto_deduct, s.is_break_auto_deduct),
    break_is_overtime = COALESCE(sdi.break_is_overtime, s.break_is_overtime),
    early_punch_in_grace_period_minutes = COALESCE(sdi.early_punch_in_grace_period_minutes, s.early_punch_in_grace_period_minutes),
    allow_early_ot = COALESCE(sdi.allow_early_ot, s.allow_early_ot),
    max_early_ot_minutes = COALESCE(sdi.max_early_ot_minutes, s.max_early_ot_minutes)
FROM public.shifts s
WHERE sdi.shift_id = s.id
  AND (
      sdi.break_duration_minutes IS NULL
      OR sdi.break_start IS NULL
      OR sdi.break_end IS NULL
      OR sdi.is_break_auto_deduct IS NULL
      OR sdi.break_is_overtime IS NULL
  );

-- Step 4: Make break/policy columns NOT NULL with defaults
-- (after backfilling, all rows should have non-null values)
ALTER TABLE public.shift_default_intervals
    ALTER COLUMN break_duration_minutes SET DEFAULT 0,
    ALTER COLUMN break_duration_minutes SET NOT NULL,
    ALTER COLUMN is_break_auto_deduct SET DEFAULT true,
    ALTER COLUMN is_break_auto_deduct SET NOT NULL,
    ALTER COLUMN break_is_overtime SET DEFAULT false,
    ALTER COLUMN break_is_overtime SET NOT NULL;

-- Step 5: Drop time/break/policy columns from shifts
ALTER TABLE public.shifts
    DROP COLUMN start_time,
    DROP COLUMN end_time,
    DROP COLUMN break_duration_minutes,
    DROP COLUMN break_start,
    DROP COLUMN break_end,
    DROP COLUMN is_break_auto_deduct,
    DROP COLUMN break_is_overtime,
    DROP COLUMN early_punch_in_grace_period_minutes,
    DROP COLUMN allow_early_ot,
    DROP COLUMN max_early_ot_minutes,
    DROP COLUMN ot_aggregation_mode,
    DROP COLUMN is_night_shift;

-- Step 6: Verify every active shift has at least 1 interval
-- (This is a sanity check — the INSERT in Step 2 should guarantee this)
DO $$
DECLARE
    orphan_count integer;
BEGIN
    SELECT count(*) INTO orphan_count
    FROM public.shifts s
    WHERE s.is_active = true
      AND NOT EXISTS (
          SELECT 1 FROM public.shift_default_intervals sdi WHERE sdi.shift_id = s.id
      );

    IF orphan_count > 0 THEN
        RAISE EXCEPTION 'DATA INTEGRITY ERROR: % active shifts have no intervals after migration', orphan_count;
    END IF;
END $$;
