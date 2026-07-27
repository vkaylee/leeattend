-- ============================================================================
-- ADR-001: Layer overrides L2a / L2b / L2c
--
--   L2a: attendance_policy_day_type_overrides       — scope key: day_type (VARCHAR)
--   L2b: attendance_policy_shift_template_overrides — scope key: shift_template_id
--   L2c: attendance_policy_shift_version_overrides  — scope key: shift_version_id
--
-- All three carry a `contributions JSONB` — a `PartialPolicyRules` payload with
-- Option<T> fields. Empty {} is legal (means: layer contributes nothing).
--
-- Overrides are only mutable while the parent policy version is DRAFT.
-- Enforced by trigger on INSERT/UPDATE/DELETE.
-- ============================================================================

-- ---------------------------------------------------------------------------
-- Guard function: block any write when parent policy_version is not DRAFT.
-- ---------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION public.enforce_policy_override_only_when_draft()
RETURNS TRIGGER AS $$
DECLARE
    v_status public.policy_version_status_type;
    v_version_id UUID;
BEGIN
    v_version_id := COALESCE(NEW.policy_version_id, OLD.policy_version_id);

    SELECT status INTO v_status
    FROM public.attendance_policy_versions
    WHERE id = v_version_id;

    IF v_status IS NULL THEN
        RAISE EXCEPTION 'POLICY_VERSION_NOT_FOUND: %', v_version_id
            USING ERRCODE = 'foreign_key_violation';
    END IF;

    IF v_status <> 'DRAFT' THEN
        RAISE EXCEPTION 'POLICY_OVERRIDE_LOCKED: cannot modify overrides on % version %', v_status, v_version_id
            USING ERRCODE = 'check_violation';
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- ---------------------------------------------------------------------------
-- L2a: day_type override
-- ---------------------------------------------------------------------------
CREATE TABLE public.attendance_policy_day_type_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_version_id UUID NOT NULL
        REFERENCES public.attendance_policy_versions(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL,
    day_type VARCHAR(50) NOT NULL,
    contributions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (policy_version_id, day_type)
);

CREATE INDEX idx_apdt_overrides_workspace
    ON public.attendance_policy_day_type_overrides (workspace_id);

CREATE TRIGGER trg_apdt_overrides_draft_only
    BEFORE INSERT OR UPDATE OR DELETE
    ON public.attendance_policy_day_type_overrides
    FOR EACH ROW EXECUTE FUNCTION public.enforce_policy_override_only_when_draft();

COMMENT ON TABLE public.attendance_policy_day_type_overrides IS
    'L2a — override fields keyed by day_type (weekday / saturday / sunday / holiday_national / ...).';
COMMENT ON COLUMN public.attendance_policy_day_type_overrides.contributions IS
    'PartialPolicyRules JSON. Option<T> fields — omitted keys mean "no override at this layer".';

-- ---------------------------------------------------------------------------
-- L2b: shift_template override (the shift "type" — stable concept identity)
-- ---------------------------------------------------------------------------
CREATE TABLE public.attendance_policy_shift_template_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_version_id UUID NOT NULL
        REFERENCES public.attendance_policy_versions(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL,
    shift_template_id UUID NOT NULL
        REFERENCES public.shift_templates(id) ON DELETE CASCADE,
    contributions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (policy_version_id, shift_template_id)
);

CREATE INDEX idx_apst_overrides_workspace
    ON public.attendance_policy_shift_template_overrides (workspace_id);
CREATE INDEX idx_apst_overrides_shift_template
    ON public.attendance_policy_shift_template_overrides (shift_template_id);

CREATE TRIGGER trg_apst_overrides_draft_only
    BEFORE INSERT OR UPDATE OR DELETE
    ON public.attendance_policy_shift_template_overrides
    FOR EACH ROW EXECUTE FUNCTION public.enforce_policy_override_only_when_draft();

COMMENT ON TABLE public.attendance_policy_shift_template_overrides IS
    'L2b — override fields keyed by shift_template (the shift "type", stable across versions).';

-- ---------------------------------------------------------------------------
-- L2c: shift_version override (a specific SCD-2 version instance)
-- ---------------------------------------------------------------------------
CREATE TABLE public.attendance_policy_shift_version_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_version_id UUID NOT NULL
        REFERENCES public.attendance_policy_versions(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL,
    shift_version_id UUID NOT NULL
        REFERENCES public.shift_versions(id) ON DELETE CASCADE,
    contributions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (policy_version_id, shift_version_id)
);

CREATE INDEX idx_apsv_overrides_workspace
    ON public.attendance_policy_shift_version_overrides (workspace_id);
CREATE INDEX idx_apsv_overrides_shift_version
    ON public.attendance_policy_shift_version_overrides (shift_version_id);

CREATE TRIGGER trg_apsv_overrides_draft_only
    BEFORE INSERT OR UPDATE OR DELETE
    ON public.attendance_policy_shift_version_overrides
    FOR EACH ROW EXECUTE FUNCTION public.enforce_policy_override_only_when_draft();

COMMENT ON TABLE public.attendance_policy_shift_version_overrides IS
    'L2c — override fields keyed by shift_version (a specific instance, finer than L2b).';
