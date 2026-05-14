-- Add migration script here
ALTER TABLE public.employees 
ADD COLUMN original_hire_date date;

-- Migrate existing data
UPDATE public.employees 
SET original_hire_date = hire_date 
WHERE original_hire_date IS NULL;

-- Enforce NOT NULL
ALTER TABLE public.employees 
ALTER COLUMN original_hire_date SET NOT NULL;
