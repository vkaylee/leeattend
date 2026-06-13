-- Phase 2: Migrate employee_phases.employee_id → employment_id

-- 1. Add new column
ALTER TABLE public.employee_phases ADD COLUMN employment_id UUID;

-- 2. Backfill from active employments
UPDATE public.employee_phases ep
SET employment_id = (
    SELECT e.id FROM public.employments e
    WHERE e.employee_id = ep.employee_id AND e.status = 'active'
    ORDER BY e.start_date DESC LIMIT 1
);

-- Fallback for employees without active employment
UPDATE public.employee_phases ep
SET employment_id = (
    SELECT e.id FROM public.employments e
    WHERE e.employee_id = ep.employee_id
    ORDER BY e.start_date DESC LIMIT 1
)
WHERE ep.employment_id IS NULL;

-- 3. Make NOT NULL
ALTER TABLE public.employee_phases ALTER COLUMN employment_id SET NOT NULL;

-- 4. Add foreign key
ALTER TABLE public.employee_phases
    ADD CONSTRAINT employee_phases_employment_id_fkey
    FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;

-- 5. Drop old indexes
DROP INDEX IF EXISTS public.idx_employee_phases_active_lookup;
DROP INDEX IF EXISTS public.idx_employee_phases_employee;

-- 6. Create new indexes
CREATE INDEX idx_employee_phases_active_lookup ON public.employee_phases USING btree (employment_id, status, start_date, end_date) WHERE (status = 'active'::text);
CREATE INDEX idx_employee_phases_employment ON public.employee_phases USING btree (employment_id);

-- 7. Drop old constraint and column
ALTER TABLE public.employee_phases DROP CONSTRAINT IF EXISTS employee_phases_employee_id_fkey;
ALTER TABLE public.employee_phases DROP COLUMN employee_id;
