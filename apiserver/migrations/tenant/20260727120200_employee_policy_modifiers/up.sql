-- ============================================================================
-- ADR-001 §L2.5: employee_policy_modifiers
--
-- Long-running per-employee modifier. Applied AFTER L2c and BEFORE L3 during
-- cascade. Multiple modifiers on the same employee are allowed in disjoint
-- time ranges OR distinct priority_rank buckets.
--
-- priority_rank INT NOT NULL — lower number = applied later in cascade = wins
-- on per-field conflict at this layer. Explicit; no default value.
--
-- EXCLUDE constraint: no two rows for (employee_id, priority_rank) with
-- overlapping [effective_from, effective_to) tstzrange.
-- ============================================================================

CREATE TABLE public.employee_policy_modifiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    employee_id UUID NOT NULL
        REFERENCES public.employees(id) ON DELETE CASCADE,
    policy_version_id UUID NOT NULL
        REFERENCES public.attendance_policy_versions(id),

    priority_rank INT NOT NULL,

    effective_from TIMESTAMPTZ NOT NULL,
    effective_to TIMESTAMPTZ,   -- NULL = open-ended

    contributions JSONB NOT NULL DEFAULT '{}'::jsonb,

    reason TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_effective_range
        CHECK (effective_to IS NULL OR effective_to > effective_from),

    CONSTRAINT no_overlapping_priority
        EXCLUDE USING gist (
            employee_id WITH =,
            priority_rank WITH =,
            tstzrange(effective_from, effective_to, '[)') WITH &&
        )
);

CREATE INDEX idx_emp_policy_mod_workspace
    ON public.employee_policy_modifiers (workspace_id);
CREATE INDEX idx_emp_policy_mod_employee_active
    ON public.employee_policy_modifiers (employee_id, effective_from);
CREATE INDEX idx_emp_policy_mod_policy_version
    ON public.employee_policy_modifiers (policy_version_id);

COMMENT ON TABLE public.employee_policy_modifiers IS
    'ADR-001 L2.5 — long-running per-employee modifier (e.g. maternity, probation). Cascade rank explicit, no ghost defaults.';
COMMENT ON COLUMN public.employee_policy_modifiers.priority_rank IS
    'Lower number = later in cascade = wins on per-field conflict at this layer. Set explicitly by HR.';
COMMENT ON COLUMN public.employee_policy_modifiers.contributions IS
    'PartialPolicyRules JSON. Option<T> fields.';
