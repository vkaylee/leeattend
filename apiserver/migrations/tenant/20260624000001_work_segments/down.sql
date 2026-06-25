-- ============================================================================
-- Revert Work Segments Architecture
-- ============================================================================

-- 1. Remove segment_id from time_entries
ALTER TABLE public.time_entries
    DROP CONSTRAINT IF EXISTS time_entries_segment_id_fkey;
ALTER TABLE public.time_entries
    DROP COLUMN IF EXISTS segment_id;

-- 2. Drop work_segments table and its dependencies
DROP TRIGGER IF EXISTS trg_updated_at ON public.work_segments;
DROP TABLE IF EXISTS public.work_segments CASCADE;

-- 3. Drop shift_default_intervals table and its dependencies
DROP TRIGGER IF EXISTS trg_updated_at ON public.shift_default_intervals;
DROP TABLE IF EXISTS public.shift_default_intervals CASCADE;

-- 4. Remove new columns from shifts
ALTER TABLE public.shifts
    DROP COLUMN IF EXISTS is_night_shift;
ALTER TABLE public.shifts
    DROP COLUMN IF EXISTS ot_aggregation_mode;

-- 5. Drop ENUM types
DROP TYPE IF EXISTS public.ot_aggregation_mode_type;
DROP TYPE IF EXISTS public.segment_status_type;
DROP TYPE IF EXISTS public.gap_compensation_type;
DROP TYPE IF EXISTS public.segment_type;
