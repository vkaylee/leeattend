-- up.sql for behalf_assignments_use_user_id

-- 1. Add assignee_user_id column (nullable first to populate)
ALTER TABLE public.behalf_assignments ADD COLUMN assignee_user_id UUID;

-- 2. Populate assignee_user_id from employees.user_id
UPDATE public.behalf_assignments ba
SET assignee_user_id = e.user_id
FROM public.employees e
WHERE ba.assignee_employee_id = e.id;

-- 3. Delete any records where assignee_user_id is NULL (e.g. employee without linked user account)
DELETE FROM public.behalf_assignments WHERE assignee_user_id IS NULL;

-- 4. Alter assignee_user_id to be NOT NULL
ALTER TABLE public.behalf_assignments ALTER COLUMN assignee_user_id SET NOT NULL;

-- 5. Drop assignee_employee_id column
ALTER TABLE public.behalf_assignments DROP COLUMN assignee_employee_id;

-- 6. Drop the old unique constraint uniq_behalf_assignment and add the new one
ALTER TABLE public.behalf_assignments DROP CONSTRAINT IF EXISTS uniq_behalf_assignment;
ALTER TABLE public.behalf_assignments ADD CONSTRAINT uniq_behalf_assignment UNIQUE (workspace_id, assignee_user_id, target_employee_id, target_department_id, assignment_type);

-- 7. Drop old index and create the new one
DROP INDEX IF EXISTS public.idx_behalf_assignments_assignee;
CREATE INDEX idx_behalf_assignments_assignee ON public.behalf_assignments (workspace_id, assignee_user_id, assignment_type);
