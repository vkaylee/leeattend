-- ============================================================================
-- Phase 5: Database Optimization
-- Add composite indexes for common query patterns and mark deprecated tables
-- ============================================================================

-- 1. Composite index on time_entries for queries filtering by employment + time range
-- Common query pattern: fetch all time_entries for an employment within a month
-- This composite index is more efficient than using two separate indexes
CREATE INDEX idx_time_entries_employment_punch_in
    ON public.time_entries (employment_id, punch_in_time);

-- 2. Composite index on work_segments for queries filtering by employment + date + status
-- Common query pattern: fetch scheduled/completed segments for an employment in a date range
-- Combines the most frequently filtered columns into a single index
CREATE INDEX idx_work_segments_employment_date_status
    ON public.work_segments (employment_id, date, segment_status);

-- 3. Mark employee_shifts table as deprecated
-- This table has been superseded by employee_working_patterns + work_segments architecture
-- The table is kept for backward compatibility but should not be used in new code
COMMENT ON TABLE public.employee_shifts IS 'DEPRECATED: This table has been replaced by employee_working_patterns (for pattern assignment) and work_segments (for scheduled work instances). Do not use for new features. Kept only for backward compatibility during migration period.';
