-- up.sql for rename_behalf_assignments

-- Rename the table
ALTER TABLE public.ot_claim_behalf_assignments RENAME TO behalf_assignments;

-- Add assignment_type column (nullable first to populate)
ALTER TABLE public.behalf_assignments ADD COLUMN assignment_type VARCHAR(50);

-- Populate existing rows with 'overtime_claim'
UPDATE public.behalf_assignments SET assignment_type = 'overtime_claim';

-- Set assignment_type to NOT NULL
ALTER TABLE public.behalf_assignments ALTER COLUMN assignment_type SET NOT NULL;

-- Remove old unique constraint
ALTER TABLE public.behalf_assignments DROP CONSTRAINT uniq_behalf_assignment;

-- Add new unique constraint including assignment_type
ALTER TABLE public.behalf_assignments ADD CONSTRAINT uniq_behalf_assignment UNIQUE (workspace_id, assignee_employee_id, target_employee_id, target_department_id, assignment_type);

-- Drop old index and create new one
DROP INDEX IF EXISTS public.idx_ot_claim_behalf_assignee;
CREATE INDEX idx_behalf_assignments_assignee ON public.behalf_assignments (workspace_id, assignee_employee_id, assignment_type);
