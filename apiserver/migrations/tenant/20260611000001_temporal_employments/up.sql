-- 1. Add new columns as NULLABLE first to avoid failing on existing data
ALTER TABLE public.employees ADD COLUMN IF NOT EXISTS hire_date date;
ALTER TABLE public.employees ADD COLUMN IF NOT EXISTS original_hire_date date;
ALTER TABLE public.employees ADD COLUMN IF NOT EXISTS termination_date date;
ALTER TABLE public.employees ADD COLUMN IF NOT EXISTS is_active boolean DEFAULT true;

ALTER TABLE public.departments ADD COLUMN IF NOT EXISTS default_working_pattern_id uuid;

-- 2. Populate existing rows with fallback/default values
UPDATE public.employees 
SET hire_date = COALESCE(created_at::date, CURRENT_DATE)
WHERE hire_date IS NULL;

UPDATE public.employees 
SET original_hire_date = COALESCE(created_at::date, CURRENT_DATE)
WHERE original_hire_date IS NULL;

UPDATE public.employees 
SET is_active = true
WHERE is_active IS NULL;

-- 3. Set NOT NULL constraints and default values
ALTER TABLE public.employees ALTER COLUMN hire_date SET NOT NULL;
ALTER TABLE public.employees ALTER COLUMN original_hire_date SET NOT NULL;
ALTER TABLE public.employees ALTER COLUMN is_active SET DEFAULT true;
ALTER TABLE public.employees ALTER COLUMN is_active SET NOT NULL;

-- 4. Add constraints
ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_default_working_pattern_id_fkey FOREIGN KEY (default_working_pattern_id) REFERENCES public.working_patterns(id) ON DELETE SET NULL;

-- 5. Add notes column to employments and handle history migration
ALTER TABLE public.employments ADD COLUMN IF NOT EXISTS notes TEXT;

DO $$
DECLARE
    r RECORD;
    emp_rec RECORD;
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'employee_department_history') THEN
        -- First, update all existing employment records with their latest department assignment
        UPDATE public.employments emp
        SET department_id = CASE 
                WHEN hist.department_id IS NOT NULL AND EXISTS (SELECT 1 FROM public.departments d WHERE d.id = hist.department_id) 
                THEN hist.department_id 
                ELSE NULL 
            END,
            notes = hist.notes
        FROM (
            SELECT DISTINCT ON (employee_id) employee_id, department_id, notes
            FROM public.employee_department_history
            ORDER BY employee_id, effective_date DESC
        ) hist
        WHERE emp.employee_id = hist.employee_id;

        -- Next, create historical segments for employees with multiple department transfers
        FOR r IN 
            SELECT h.id, h.employee_id, h.department_id, h.effective_date, h.notes, h.created_at,
                   lead(h.effective_date) OVER (PARTITION BY h.employee_id ORDER BY h.effective_date ASC) as next_effective_date
            FROM public.employee_department_history h
            WHERE h.employee_id IN (
                SELECT employee_id FROM public.employee_department_history GROUP BY employee_id HAVING COUNT(*) > 1
            )
            ORDER BY h.employee_id, h.effective_date ASC
        LOOP
            IF r.next_effective_date IS NOT NULL THEN
                SELECT * INTO emp_rec FROM public.employments WHERE employee_id = r.employee_id LIMIT 1;
                IF FOUND THEN
                    INSERT INTO public.employments (
                        id, workspace_id, employee_id, employer_id, primary_location_id, 
                        department_id, status, start_date, end_date, created_at, updated_at, 
                        job_title, employment_type, manager_employment_id, notes
                    ) VALUES (
                        gen_random_uuid(), emp_rec.workspace_id, emp_rec.employee_id, emp_rec.employer_id, emp_rec.primary_location_id,
                        CASE 
                            WHEN r.department_id IS NOT NULL AND EXISTS (SELECT 1 FROM public.departments d WHERE d.id = r.department_id) 
                            THEN r.department_id 
                            ELSE NULL 
                        END,
                        'active', r.effective_date, r.next_effective_date - INTERVAL '1 day', r.created_at, r.created_at,
                        emp_rec.job_title, emp_rec.employment_type, emp_rec.manager_employment_id, r.notes
                    );
                END IF;
            ELSE
                UPDATE public.employments 
                SET start_date = r.effective_date
                WHERE employee_id = r.employee_id AND end_date IS NULL;
            END IF;
        END LOOP;

        -- Drop the legacy table
        DROP TABLE IF EXISTS public.employee_department_history CASCADE;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_employments_query ON public.employments (workspace_id, employee_id, start_date, end_date) WHERE deleted_at IS NULL;
