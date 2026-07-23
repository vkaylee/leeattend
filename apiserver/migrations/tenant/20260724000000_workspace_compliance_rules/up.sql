-- ============================================================================
-- Workspace Compliance Rules: Shift conflict & rest-period validation engine
--
-- One row per workspace. Defaults align with Vietnamese Labor Code:
--   - min_rest_period_hours = 12  (minimum rest between consecutive shifts)
--   - max_weekly_hours = 48       (standard work week cap)
--   - max_monthly_ot_hours = 40   (monthly overtime cap)
--   - allow_manager_override = true
--   - is_enabled = true
-- ============================================================================

CREATE TABLE public.workspace_compliance_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    min_rest_period_hours INT NOT NULL DEFAULT 12,
    max_weekly_hours INT NOT NULL DEFAULT 48,
    max_monthly_ot_hours INT NOT NULL DEFAULT 40,
    allow_manager_override BOOLEAN NOT NULL DEFAULT true,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    UNIQUE(workspace_id)
);

COMMENT ON TABLE public.workspace_compliance_rules IS 'Per-workspace compliance rule thresholds for shift conflict detection and rest-period validation.';
COMMENT ON COLUMN public.workspace_compliance_rules.id IS 'Primary key.';
COMMENT ON COLUMN public.workspace_compliance_rules.workspace_id IS 'Reference to the workspace these rules apply to. One row per workspace enforced by UNIQUE constraint.';
COMMENT ON COLUMN public.workspace_compliance_rules.min_rest_period_hours IS 'Minimum rest hours required between two consecutive shifts. Default 12 per Vietnamese Labor Code.';
COMMENT ON COLUMN public.workspace_compliance_rules.max_weekly_hours IS 'Maximum total working hours allowed per calendar week (Mon-Sun). Default 48.';
COMMENT ON COLUMN public.workspace_compliance_rules.max_monthly_ot_hours IS 'Maximum overtime hours allowed per calendar month. Default 40 per Vietnamese Labor Code.';
COMMENT ON COLUMN public.workspace_compliance_rules.allow_manager_override IS 'When true, managers can override soft compliance warnings by providing a reason.';
COMMENT ON COLUMN public.workspace_compliance_rules.is_enabled IS 'When false, all compliance checks are skipped for this workspace.';

-- Index
CREATE INDEX idx_workspace_compliance_rules_workspace
    ON public.workspace_compliance_rules(workspace_id);

-- Updated-at trigger
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.workspace_compliance_rules
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- Seed default compliance rules for all existing workspaces.
-- The public.workspaces table only exists in the system database, not in
-- individual tenant databases. Wrap in a conditional to make this safe
-- for both tenant and system migration contexts.
DO $$
BEGIN
    IF EXISTS (
        SELECT FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = 'workspaces'
    ) THEN
        INSERT INTO public.workspace_compliance_rules (workspace_id)
        SELECT id FROM public.workspaces
        ON CONFLICT (workspace_id) DO NOTHING;
    END IF;
END $$;
