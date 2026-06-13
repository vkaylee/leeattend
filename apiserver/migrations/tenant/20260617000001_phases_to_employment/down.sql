-- Phase 2 rollback

-- 1. Add old column
ALTER TABLE public.employee_phases ADD COLUMN employee_id UUID;

-- 2. Backfill employee_id from employment_id
UPDATE public.employee_phases ep
SET employee_id = (
    SELECT e.employee_id FROM public.employments e
    WHERE e.id = ep.employment_id
);

-- 3. Make NOT NULL
ALTER TABLE public.employee_phases ALTER COLUMN employee_id SET NOT NULL;

-- 4. Add old foreign key constraint
ALTER TABLE public.employee_phases
    ADD CONSTRAINT employee_phases_employee_id_fkey
    FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

-- 5. Drop new indexes
DROP INDEX IF EXISTS public.idx_employee_phases_active_lookup;
DROP INDEX IF EXISTS public.idx_employee_phases_employment;

-- 6. Recreate old indexes
CREATE INDEX idx_employee_phases_active_lookup ON public.employee_phases USING btree (employee_id, status, start_date, end_date) WHERE (status = 'active'::text);
CREATE INDEX idx_employee_phases_employee ON public.employee_phases USING btree (employee_id);

-- 7. Drop new constraint and column
ALTER TABLE public.employee_phases DROP CONSTRAINT IF EXISTS employee_phases_employment_id_fkey;
ALTER TABLE public.employee_phases DROP COLUMN employment_id;
