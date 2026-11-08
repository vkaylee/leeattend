-- Baseline normalization for empty fresh databases.
-- Replaces immutable 20261106000000_remove_system_roles on unpopulated setups
-- where 060 preflight would fail due to zero organizations.
-- Enforces empty-state invariant before dropping legacy tables.

BEGIN;

DO $$
DECLARE
    v_has_legacy boolean;
BEGIN
    SELECT EXISTS (
        SELECT 1 FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public' AND c.relname IN ('system_roles', 'user_system_roles', 'system_db_clusters')
    ) INTO v_has_legacy;

    IF NOT v_has_legacy THEN
        RETURN;
    END IF;

    -- Baseline contains all tables listed; lock before checking current state.
    LOCK TABLE public.workspaces, public.workspace_roles, public.workspace_members,
        public.users, public.user_system_roles, public.system_roles,
        public.system_db_clusters IN ACCESS EXCLUSIVE MODE;

    IF (SELECT count(*) FROM public.workspaces) > 0 THEN
        RAISE EXCEPTION 'fresh baseline normalization refused: public.workspaces contains data; use standard populated migration path';
    END IF;

    DROP TABLE public.user_system_roles;
    DROP TABLE public.system_roles;
    DROP TABLE public.system_db_clusters;
END;
$$;

COMMIT;
