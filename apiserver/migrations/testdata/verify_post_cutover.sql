-- Post-cutover schema assertion check
-- Validates zero workspace_id columns and exact expected final table names.

DO $$
DECLARE
    v_ws_col_count integer;
    v_ws_tbl_count integer;
BEGIN
    SELECT count(*) INTO v_ws_col_count
    FROM pg_attribute a
    JOIN pg_class c ON c.oid = a.attrelid
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = 'public'
      AND a.attname = 'workspace_id'
      AND NOT a.attisdropped;

    IF v_ws_col_count > 0 THEN
        RAISE EXCEPTION 'post-cutover schema check failed: % workspace_id columns remain', v_ws_col_count;
    END IF;

    SELECT count(*) INTO v_ws_tbl_count
    FROM pg_class c
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = 'public'
      AND c.relkind = 'r'
      AND c.relname IN (
          'workspaces',
          'workspace_members',
          'workspace_roles',
          'workspace_role_permissions',
          'member_extra_roles',
          'workspace_member_extra_permissions',
          'workspace_files',
          'workspace_compliance_rules',
          'workspace_config_entries'
      );

    IF v_ws_tbl_count > 0 THEN
        RAISE EXCEPTION 'post-cutover schema check failed: % legacy workspace_* tables remain', v_ws_tbl_count;
    END IF;

    -- Validate required singleton tables exist
    IF to_regclass('public.organization_settings') IS NULL
       OR to_regclass('public.roles') IS NULL
       OR to_regclass('public.user_roles') IS NULL
       OR to_regclass('public.role_permissions') IS NULL
       OR to_regclass('public.files') IS NULL
       OR to_regclass('public.compliance_rules') IS NULL
       OR to_regclass('public.config_entries') IS NULL
       OR to_regclass('public.user_extra_roles') IS NULL
       OR to_regclass('public.user_extra_permissions') IS NULL THEN
        RAISE EXCEPTION 'post-cutover schema check failed: one or more final single-organization tables missing';
    END IF;

    RAISE NOTICE 'post-cutover schema check: PASS (0 workspace_id columns, 0 legacy tables, all final single-organization tables verified)';
END;
$$;
