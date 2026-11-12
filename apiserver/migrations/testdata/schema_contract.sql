-- Fresh-baseline schema contract check.
-- Run after baseline installation through the repository migration wrapper.

DO $$
DECLARE
    v_legacy_col_count integer;
    v_legacy_object_count integer;
    v_required_table_count integer;
    v_marker_count integer;
BEGIN
    SELECT count(*) INTO v_legacy_col_count
    FROM pg_attribute a
    JOIN pg_class c ON c.oid = a.attrelid
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = 'public'
      AND a.attname LIKE 'workspace%'
      AND NOT a.attisdropped;

    IF v_legacy_col_count > 0 THEN
        RAISE EXCEPTION 'baseline schema check failed: % workspace-prefixed columns remain', v_legacy_col_count;
    END IF;

    SELECT count(*) INTO v_legacy_object_count
    FROM pg_class c
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = 'public'
      AND c.relkind IN ('r', 'p', 'i', 'S', 'v', 'm', 'f')
      AND c.relname LIKE 'workspace%';

    IF v_legacy_object_count > 0 THEN
        RAISE EXCEPTION 'baseline schema check failed: % workspace-prefixed objects remain', v_legacy_object_count;
    END IF;

    SELECT count(*) INTO v_required_table_count
    FROM pg_class c
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = 'public'
      AND c.relkind IN ('r', 'p')
      AND c.relname IN (
          'organization_settings', 'roles', 'user_roles', 'role_permissions',
          'files', 'compliance_rules', 'config_entries',
          'user_extra_roles', 'user_extra_permissions'
      );

    IF v_required_table_count <> 9 THEN
        RAISE EXCEPTION 'baseline schema check failed: expected 9 final organization tables, found %', v_required_table_count;
    END IF;

    IF to_regclass('public.organization_settings_singleton_key') IS NULL THEN
        RAISE EXCEPTION 'baseline schema check failed: organization singleton constraint missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261110000000_single_organization_baseline';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: baseline migration marker is missing';
    END IF;

    IF to_regclass('public.config_entries') IS NULL THEN
        RAISE EXCEPTION 'schema check failed: config_entries table is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.config_entries'::regclass
          AND conname = 'config_entries_effective_date_range_check'
    ) THEN
        RAISE EXCEPTION 'schema check failed: config_entries temporal constraint is missing';
    END IF;

    RAISE NOTICE 'schema contract check: PASS';
END;
$$;
