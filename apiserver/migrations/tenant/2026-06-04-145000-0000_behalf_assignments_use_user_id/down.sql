-- down.sql for behalf_assignments_use_user_id

-- 1. Add assignee_employee_id column
ALTER TABLE public.behalf_assignments ADD COLUMN assignee_employee_id UUID;

-- 2. Populate assignee_employee_id from employees.id matching user_id and workspace_id
UPDATE public.behalf_assignments ba
SET assignee_employee_id = e.id
FROM public.employees e
WHERE ba.assignee_user_id = e.user_id AND ba.workspace_id = e.workspace_id;

-- 3. Delete any records where assignee_employee_id is NULL
DELETE FROM public.behalf_assignments WHERE assignee_employee_id IS NULL;

-- 4. Alter assignee_employee_id to be NOT NULL and add foreign key reference to employees(id)
ALTER TABLE public.behalf_assignments ALTER COLUMN assignee_employee_id SET NOT NULL;
ALTER TABLE public.behalf_assignments ADD CONSTRAINT behalf_assignments_assignee_employee_id_fkey FOREIGN KEY (assignee_employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

-- 5. Drop assignee_user_id column
ALTER TABLE public.behalf_assignments DROP COLUMN assignee_user_id;

-- 6. Drop the new unique constraint and recreate the old one
ALTER TABLE public.behalf_assignments DROP CONSTRAINT IF EXISTS uniq_behalf_assignment;
ALTER TABLE public.behalf_assignments ADD CONSTRAINT uniq_behalf_assignment UNIQUE (workspace_id, assignee_employee_id, target_employee_id, target_department_id, assignment_type);

-- 7. Drop new index and recreate the old one
DROP INDEX IF EXISTS public.idx_behalf_assignments_assignee;
CREATE INDEX idx_behalf_assignments_assignee ON public.behalf_assignments (workspace_id, assignee_employee_id, assignment_type);
