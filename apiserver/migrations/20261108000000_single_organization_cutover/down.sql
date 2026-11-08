-- Honest rollback refusal for single_organization_cutover.
-- Stripping workspace_id globally cannot automatically partition or restore
-- multi-tenant rows without point-in-time snapshot recovery.
-- This script safely fails and instructs operator on standard recovery.

BEGIN;

DO $$
BEGIN
    RAISE EXCEPTION 'MIGRATION ROLLBACK REFUSED: single-organization cutover (20261108000000) permanently removes workspace scoping and cannot reconstruct previous multi-tenant boundaries. Restore pre-cutover PostgreSQL backup from maintenance snapshot.';
END;
$$;

ROLLBACK;
