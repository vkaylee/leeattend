ALTER TABLE public.leave_policy_assignments DROP CONSTRAINT IF EXISTS leave_policy_assignments_workspace_id_assignee_type_coales_excl;

ALTER TABLE public.leave_policy_assignments
ADD CONSTRAINT leave_policy_assignments_workspace_id_assignee_type_coales_excl
EXCLUDE USING gist (
    workspace_id WITH =,
    assignee_type WITH =,
    COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =,
    daterange(effective_from, effective_to, '[)'::text) WITH &&
);
