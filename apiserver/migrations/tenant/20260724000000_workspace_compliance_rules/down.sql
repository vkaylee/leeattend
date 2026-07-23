-- ============================================================================
-- Reverse: Drop workspace_compliance_rules
-- ============================================================================

DROP TRIGGER IF EXISTS trg_updated_at ON public.workspace_compliance_rules;
DROP TABLE IF EXISTS public.workspace_compliance_rules;
