-- 1. employee_salary_components
ALTER TABLE public.employee_salary_components ADD COLUMN employment_id UUID;

UPDATE public.employee_salary_components esc
SET employment_id = (
    SELECT emp.id 
    FROM public.employments emp 
    WHERE emp.employee_id = esc.employee_id 
    ORDER BY emp.start_date DESC 
    LIMIT 1
);

DELETE FROM public.employee_salary_components WHERE employment_id IS NULL;
ALTER TABLE public.employee_salary_components ALTER COLUMN employment_id SET NOT NULL;

ALTER TABLE public.employee_salary_components DROP CONSTRAINT employee_salary_components_employee_id_fkey;
DROP INDEX IF EXISTS public.idx_employee_salary_components_employee_id;
ALTER TABLE public.employee_salary_components DROP COLUMN employee_id;

ALTER TABLE public.employee_salary_components 
    ADD CONSTRAINT employee_salary_components_employment_id_fkey 
    FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;
CREATE INDEX idx_employee_salary_components_employment_id ON public.employee_salary_components USING btree (employment_id);


-- 2. employee_shifts
ALTER TABLE public.employee_shifts ADD COLUMN employment_id UUID;

UPDATE public.employee_shifts es
SET employment_id = (
    SELECT emp.id 
    FROM public.employments emp 
    WHERE emp.employee_id = es.employee_id 
    ORDER BY emp.start_date DESC 
    LIMIT 1
);

DELETE FROM public.employee_shifts WHERE employment_id IS NULL;
ALTER TABLE public.employee_shifts ALTER COLUMN employment_id SET NOT NULL;

ALTER TABLE public.employee_shifts DROP CONSTRAINT employee_shifts_new_employee_id_fkey;
DROP INDEX IF EXISTS public.idx_employee_shifts_employee_id;
ALTER TABLE public.employee_shifts DROP COLUMN employee_id;

ALTER TABLE public.employee_shifts 
    ADD CONSTRAINT employee_shifts_new_employment_id_fkey 
    FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;
CREATE INDEX idx_employee_shifts_employment_id ON public.employee_shifts USING btree (employment_id);


-- 3. employee_working_patterns
ALTER TABLE public.employee_working_patterns ADD COLUMN employment_id UUID;

UPDATE public.employee_working_patterns ewp
SET employment_id = (
    SELECT emp.id 
    FROM public.employments emp 
    WHERE emp.employee_id = ewp.employee_id 
    ORDER BY emp.start_date DESC 
    LIMIT 1
);

DELETE FROM public.employee_working_patterns WHERE employment_id IS NULL;
ALTER TABLE public.employee_working_patterns ALTER COLUMN employment_id SET NOT NULL;

ALTER TABLE public.employee_working_patterns DROP CONSTRAINT employee_working_patterns_employee_id_fkey;
DROP INDEX IF EXISTS public.idx_employee_working_patterns_employee_date;
ALTER TABLE public.employee_working_patterns DROP COLUMN employee_id;

ALTER TABLE public.employee_working_patterns 
    ADD CONSTRAINT employee_working_patterns_employment_id_fkey 
    FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;
CREATE INDEX idx_employee_working_patterns_employment_date ON public.employee_working_patterns USING btree (employment_id, start_date);


-- 4. leave_accrual_logs
ALTER TABLE public.leave_accrual_logs ADD COLUMN employment_id UUID;

UPDATE public.leave_accrual_logs lal
SET employment_id = (
    SELECT emp.id 
    FROM public.employments emp 
    WHERE emp.employee_id = lal.employee_id 
    ORDER BY emp.start_date DESC 
    LIMIT 1
);

DELETE FROM public.leave_accrual_logs WHERE employment_id IS NULL;
ALTER TABLE public.leave_accrual_logs ALTER COLUMN employment_id SET NOT NULL;

ALTER TABLE public.leave_accrual_logs DROP CONSTRAINT leave_accrual_logs_employee_id_fkey;
ALTER TABLE public.leave_accrual_logs DROP CONSTRAINT leave_accrual_logs_employee_leave_type_id_date_key;
ALTER TABLE public.leave_accrual_logs DROP COLUMN employee_id;

ALTER TABLE public.leave_accrual_logs 
    ADD CONSTRAINT leave_accrual_logs_employment_id_fkey 
    FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;
ALTER TABLE public.leave_accrual_logs 
    ADD CONSTRAINT leave_accrual_logs_employment_leave_type_id_date_key 
    UNIQUE (employment_id, leave_type_id, accrual_date);
