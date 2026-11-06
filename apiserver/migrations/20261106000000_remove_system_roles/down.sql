-- Legacy platform tables contain data that cannot be reconstructed from the
-- standalone workspace-RBAC schema. Do not run this file as a live rollback.
-- Restore the complete encrypted pre-migration PostgreSQL backup instead.
DO $$
BEGIN
    RAISE EXCEPTION 'irreversible platform-table contraction: restore the pre-migration database backup';
END
$$;
