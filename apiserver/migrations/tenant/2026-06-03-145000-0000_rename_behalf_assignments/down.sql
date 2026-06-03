-- down.sql for rename_behalf_assignments

-- Revert unique constraint
ALTER TABLE public.behalf_assignments DROP CONSTRAINT uniq_behalf_assignment;
ALTER TABLE public.behalf_assignments ADD CONSTRAINT uniq_behalf_assignment UNIQUE (workspace_id, assignee_employee_id, target_employee_id, target_department_id);

-- Drop new index and recreate old one
DROP INDEX IF EXISTS public.idx_behalf_assignments_assignee;
CREATE INDEX idx_ot_claim_behalf_assignee ON public.behalf_assignments (workspace_id, assignee_employee_id);

-- Drop column assignment_type
ALTER TABLE public.behalf_assignments DROP COLUMN assignment_type;

-- Rename table back
ALTER TABLE public.behalf_assignments RENAME TO ot_claim_behalf_assignments;
