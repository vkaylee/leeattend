-- ============================================================================
-- Reverse: Shift Versioning → Original shifts table
-- ============================================================================

-- Step 1: Restore old shift_id columns on consumer tables
ALTER TABLE public.employee_shifts ADD COLUMN shift_id UUID;
ALTER TABLE public.shift_rosters ADD COLUMN shift_id UUID;
ALTER TABLE public.shift_swap_requests ADD COLUMN requester_shift_id UUID;
ALTER TABLE public.shift_swap_requests ADD COLUMN target_shift_id UUID;
ALTER TABLE public.time_entries ADD COLUMN shift_id UUID;
ALTER TABLE public.work_segments ADD COLUMN shift_id UUID;
ALTER TABLE public.working_pattern_details ADD COLUMN shift_id UUID;
ALTER TABLE public.shift_default_intervals ADD COLUMN shift_id UUID;

-- Step 2: Backfill shift_id from template_id → current_version_id
-- For each template, resolve the current version and use its id as shift_id.
UPDATE public.employee_shifts es
SET shift_id = st.current_version_id
FROM public.shift_templates st
WHERE es.template_id = st.id AND st.current_version_id IS NOT NULL;

UPDATE public.shift_rosters sr
SET shift_id = st.current_version_id
FROM public.shift_templates st
WHERE sr.template_id = st.id AND st.current_version_id IS NOT NULL;

UPDATE public.shift_swap_requests ssr
SET requester_shift_id = st.current_version_id
FROM public.shift_templates st
WHERE ssr.requester_template_id = st.id AND st.current_version_id IS NOT NULL;

UPDATE public.shift_swap_requests ssr
SET target_shift_id = st.current_version_id
FROM public.shift_templates st
WHERE ssr.target_template_id = st.id AND st.current_version_id IS NOT NULL;

UPDATE public.time_entries te
SET shift_id = st.current_version_id
FROM public.shift_templates st
WHERE te.template_id = st.id AND st.current_version_id IS NOT NULL;

UPDATE public.work_segments ws
SET shift_id = st.current_version_id
FROM public.shift_templates st
WHERE ws.template_id = st.id AND st.current_version_id IS NOT NULL;

UPDATE public.working_pattern_details wpd
SET shift_id = st.current_version_id
FROM public.shift_templates st
WHERE wpd.template_id = st.id AND st.current_version_id IS NOT NULL;

UPDATE public.shift_default_intervals sdi
SET shift_id = sdi.version_id;  -- version_id = original shift_id

-- Step 3: Drop new FK constraints
ALTER TABLE public.employee_shifts DROP CONSTRAINT IF EXISTS employee_shifts_template_id_fkey;
ALTER TABLE public.shift_rosters DROP CONSTRAINT IF EXISTS shift_rosters_template_id_fkey;
ALTER TABLE public.shift_swap_requests DROP CONSTRAINT IF EXISTS shift_swap_requests_requester_template_id_fkey;
ALTER TABLE public.shift_swap_requests DROP CONSTRAINT IF EXISTS shift_swap_requests_target_template_id_fkey;
ALTER TABLE public.time_entries DROP CONSTRAINT IF EXISTS time_entries_template_id_fkey;
ALTER TABLE public.work_segments DROP CONSTRAINT IF EXISTS work_segments_template_id_fkey;
ALTER TABLE public.working_pattern_details DROP CONSTRAINT IF EXISTS working_pattern_details_template_id_fkey;
ALTER TABLE public.shift_default_intervals DROP CONSTRAINT IF EXISTS shift_default_intervals_version_id_fkey;
ALTER TABLE public.shift_default_intervals DROP CONSTRAINT IF EXISTS shift_default_intervals_version_id_sort_order_key;

-- Step 4: Drop template_id/version_id columns
ALTER TABLE public.employee_shifts DROP COLUMN template_id;
ALTER TABLE public.shift_rosters DROP COLUMN template_id;
ALTER TABLE public.shift_swap_requests DROP COLUMN requester_template_id;
ALTER TABLE public.shift_swap_requests DROP COLUMN target_template_id;
ALTER TABLE public.time_entries DROP COLUMN template_id;
ALTER TABLE public.work_segments DROP COLUMN template_id;
ALTER TABLE public.working_pattern_details DROP COLUMN template_id;
ALTER TABLE public.shift_default_intervals DROP COLUMN version_id;

-- Step 5: Restore old FK constraints
ALTER TABLE public.employee_shifts
    ADD CONSTRAINT employee_shifts_new_shift_id_fkey
    FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE CASCADE;

ALTER TABLE public.shift_rosters
    ADD CONSTRAINT shift_rosters_shift_id_fkey
    FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requester_shift_id_fkey
    FOREIGN KEY (requester_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;
ALTER TABLE public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_target_shift_id_fkey
    FOREIGN KEY (target_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE public.time_entries
    ADD CONSTRAINT time_entries_shift_id_fkey
    FOREIGN KEY (shift_id) REFERENCES public.shifts(id);

ALTER TABLE public.work_segments
    ADD CONSTRAINT work_segments_shift_id_fkey
    FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE RESTRICT;

ALTER TABLE public.working_pattern_details
    ADD CONSTRAINT working_pattern_details_shift_id_fkey
    FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_shift_id_fkey
    FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE CASCADE;
ALTER TABLE public.shift_default_intervals
    ADD CONSTRAINT shift_default_intervals_shift_id_sort_order_key
    UNIQUE (shift_id, sort_order);

-- Step 6: Restore shifts table FKs
ALTER TABLE public.shifts
    ADD CONSTRAINT shifts_parent_shift_id_fkey
    FOREIGN KEY (parent_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;
ALTER TABLE public.shifts
    ADD CONSTRAINT shifts_employer_id_fkey
    FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE SET NULL;
ALTER TABLE public.shifts
    ADD CONSTRAINT shifts_location_id_fkey
    FOREIGN KEY (location_id) REFERENCES public.work_locations(id) ON DELETE SET NULL;

-- Step 7: Drop new tables
DROP TABLE IF EXISTS public.shift_audit_log;
DROP TABLE IF EXISTS public.shift_versions;
DROP TABLE IF EXISTS public.shift_templates;
