-- Phase 1: Data Integrity Foundation (System tables)
-- Reusable updated_at trigger function for system tables
-- Triggers are created in the respective table migrations

CREATE OR REPLACE FUNCTION public.fn_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
