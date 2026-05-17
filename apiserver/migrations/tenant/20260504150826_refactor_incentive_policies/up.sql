-- Add rule_config JSONB column and drop old hardcoded columns
ALTER TABLE public.payroll_incentive_tiers 
ADD COLUMN rule_config jsonb DEFAULT '{}'::jsonb NOT NULL;

ALTER TABLE public.payroll_incentive_tiers DROP COLUMN max_late_early_count;

ALTER TABLE public.payroll_incentive_tiers DROP COLUMN max_unpaid_leave_days;

ALTER TABLE public.payroll_incentive_tiers DROP COLUMN max_sick_leave_days;

ALTER TABLE public.payroll_incentive_tiers DROP COLUMN max_total_leave_days;

-- Create payroll_incentive_assignments table
CREATE TABLE public.payroll_incentive_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    policy_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_incentive_assignments_pkey PRIMARY KEY (id),
    CONSTRAINT payroll_incentive_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE,
    CONSTRAINT payroll_incentive_assignments_assignee_type_check 
        CHECK (assignee_type IN ('workspace', 'department', 'employee'))
);

CREATE INDEX idx_payroll_incentive_assignments_lookup 
ON public.payroll_incentive_assignments USING btree (workspace_id, assignee_type, assignee_id);