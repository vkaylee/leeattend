-- Phase 1: Data Integrity Foundation (System tables)
-- Reusable updated_at trigger for workspace_roles and other system tables

CREATE OR REPLACE FUNCTION public.fn_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- workspace_roles
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.workspace_roles
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();
