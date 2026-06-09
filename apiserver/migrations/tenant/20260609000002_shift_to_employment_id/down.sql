-- 1. employee_salary_components
ALTER TABLE public.employee_salary_components ADD COLUMN employee_id UUID;

UPDATE public.employee_salary_components esc
SET employee_id = (
    SELECT emp.employee_id 
    FROM public.employments emp 
    WHERE emp.id = esc.employment_id 
    LIMIT 1
);

DELETE FROM public.employee_salary_components WHERE employee_id IS NULL;
ALTER TABLE public.employee_salary_components ALTER COLUMN employee_id SET NOT NULL;

ALTER TABLE public.employee_salary_components DROP CONSTRAINT employee_salary_components_employment_id_fkey;
DROP INDEX IF EXISTS public.idx_employee_salary_components_employment_id;
ALTER TABLE public.employee_salary_components DROP COLUMN employment_id;

ALTER TABLE public.employee_salary_components 
    ADD CONSTRAINT employee_salary_components_employee_id_fkey 
    FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;
CREATE INDEX idx_employee_salary_components_employee_id ON public.employee_salary_components USING btree (employee_id);


-- 2. employee_shifts
ALTER TABLE public.employee_shifts ADD COLUMN employee_id UUID;

UPDATE public.employee_shifts es
SET employee_id = (
    SELECT emp.employee_id 
    FROM public.employments emp 
    WHERE emp.id = es.employment_id 
    LIMIT 1
);

DELETE FROM public.employee_shifts WHERE employee_id IS NULL;
ALTER TABLE public.employee_shifts ALTER COLUMN employee_id SET NOT NULL;

ALTER TABLE public.employee_shifts DROP CONSTRAINT employee_shifts_new_employment_id_fkey;
DROP INDEX IF EXISTS public.idx_employee_shifts_employment_id;
ALTER TABLE public.employee_shifts DROP COLUMN employment_id;

ALTER TABLE public.employee_shifts 
    ADD CONSTRAINT employee_shifts_new_employee_id_fkey 
    FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;
CREATE INDEX idx_employee_shifts_employee_id ON public.employee_shifts USING btree (employee_id);


-- 3. employee_working_patterns
ALTER TABLE public.employee_working_patterns ADD COLUMN employee_id UUID;

UPDATE public.employee_working_patterns ewp
SET employee_id = (
    SELECT emp.employee_id 
    FROM public.employments emp 
    WHERE emp.id = ewp.employment_id 
    LIMIT 1
);

DELETE FROM public.employee_working_patterns WHERE employee_id IS NULL;
ALTER TABLE public.employee_working_patterns ALTER COLUMN employee_id SET NOT NULL;

ALTER TABLE public.employee_working_patterns DROP CONSTRAINT employee_working_patterns_employment_id_fkey;
DROP INDEX IF EXISTS public.idx_employee_working_patterns_employment_date;
ALTER TABLE public.employee_working_patterns DROP COLUMN employment_id;

ALTER TABLE public.employee_working_patterns 
    ADD CONSTRAINT employee_working_patterns_employee_id_fkey 
    FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;
CREATE INDEX idx_employee_working_patterns_employee_date ON public.employee_working_patterns USING btree (employee_id, start_date);


-- 4. leave_accrual_logs
ALTER TABLE public.leave_accrual_logs ADD COLUMN employee_id UUID;

UPDATE public.leave_accrual_logs lal
SET employee_id = (
    SELECT emp.employee_id 
    FROM public.employments emp 
    WHERE emp.id = lal.employment_id 
    LIMIT 1
);

DELETE FROM public.leave_accrual_logs WHERE employee_id IS NULL;
ALTER TABLE public.leave_accrual_logs ALTER COLUMN employee_id SET NOT NULL;

ALTER TABLE public.leave_accrual_logs DROP CONSTRAINT leave_accrual_logs_employment_id_fkey;
ALTER TABLE public.leave_accrual_logs DROP CONSTRAINT leave_accrual_logs_employment_leave_type_id_date_key;
ALTER TABLE public.leave_accrual_logs DROP COLUMN employment_id;

ALTER TABLE public.leave_accrual_logs 
    ADD CONSTRAINT leave_accrual_logs_employee_id_fkey 
    FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;
ALTER TABLE public.leave_accrual_logs 
    ADD CONSTRAINT leave_accrual_logs_employee_leave_type_id_date_key 
    UNIQUE (employee_id, leave_type_id, accrual_date);
