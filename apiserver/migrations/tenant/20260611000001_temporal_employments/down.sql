CREATE TABLE IF NOT EXISTS public.employee_department_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    department_id uuid NOT NULL,
    effective_date date NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

-- Check and add primary key constraint if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'employee_department_history_pkey') THEN
        ALTER TABLE ONLY public.employee_department_history
            ADD CONSTRAINT employee_department_history_pkey PRIMARY KEY (id);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_emp_dept_hist_department_id ON public.employee_department_history USING btree (department_id);
CREATE INDEX IF NOT EXISTS idx_emp_dept_hist_emp_date ON public.employee_department_history USING btree (employee_id, effective_date DESC);
CREATE INDEX IF NOT EXISTS idx_emp_dept_hist_employee_id ON public.employee_department_history USING btree (employee_id);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'employee_department_history_department_id_fkey') THEN
        ALTER TABLE ONLY public.employee_department_history
            ADD CONSTRAINT employee_department_history_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id) ON DELETE CASCADE;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'employee_department_history_employee_id_fkey') THEN
        ALTER TABLE ONLY public.employee_department_history
            ADD CONSTRAINT employee_department_history_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Lossless Rollback: Restore data back from employments
INSERT INTO public.employee_department_history (id, employee_id, department_id, effective_date, notes, created_at)
SELECT gen_random_uuid(), employee_id, department_id, start_date, notes, created_at
FROM public.employments
WHERE department_id IS NOT NULL AND deleted_at IS NULL;

-- Drop new indices and columns
DROP INDEX IF EXISTS public.idx_employments_query;

ALTER TABLE public.employments DROP COLUMN IF EXISTS notes;

ALTER TABLE public.departments DROP CONSTRAINT IF EXISTS departments_default_working_pattern_id_fkey;
ALTER TABLE public.departments DROP COLUMN IF EXISTS default_working_pattern_id;

ALTER TABLE public.employees DROP COLUMN IF EXISTS hire_date;
ALTER TABLE public.employees DROP COLUMN IF EXISTS original_hire_date;
ALTER TABLE public.employees DROP COLUMN IF EXISTS termination_date;
ALTER TABLE public.employees DROP COLUMN IF EXISTS is_active;
