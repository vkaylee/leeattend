CREATE TABLE public.employment_periods (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    hire_date date NOT NULL,
    termination_date date,
    is_rehire boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT employment_periods_pkey PRIMARY KEY (id)
);

ALTER TABLE public.employment_periods
    ADD CONSTRAINT fk_employment_periods_employee_id FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

INSERT INTO employment_periods (employee_id, hire_date, termination_date)
SELECT id, hire_date, termination_date FROM employees;

ALTER TABLE employees 
DROP COLUMN hire_date,
DROP COLUMN termination_date;
