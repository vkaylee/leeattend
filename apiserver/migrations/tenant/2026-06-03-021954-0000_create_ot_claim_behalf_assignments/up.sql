-- up.sql for create_ot_claim_behalf_assignments
CREATE TABLE public.ot_claim_behalf_assignments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id uuid NOT NULL,
    assignee_employee_id uuid NOT NULL REFERENCES public.employees(id) ON DELETE CASCADE,
    target_employee_id uuid REFERENCES public.employees(id) ON DELETE CASCADE,
    target_department_id uuid REFERENCES public.departments(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT check_target CHECK (
        (target_employee_id IS NOT NULL AND target_department_id IS NULL) OR
        (target_employee_id IS NULL AND target_department_id IS NOT NULL)
    ),
    CONSTRAINT uniq_behalf_assignment UNIQUE (workspace_id, assignee_employee_id, target_employee_id, target_department_id)
);

CREATE INDEX idx_ot_claim_behalf_assignee ON public.ot_claim_behalf_assignments (workspace_id, assignee_employee_id);
