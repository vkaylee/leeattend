-- Rollback for Phase 5: Database Optimization

-- Remove the composite indexes
DROP INDEX IF EXISTS public.idx_time_entries_employment_punch_in;
DROP INDEX IF EXISTS public.idx_work_segments_employment_date_status;

-- Remove the deprecation comment (reset to empty or original)
COMMENT ON TABLE public.employee_shifts IS NULL;
