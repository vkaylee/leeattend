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

    IF to_regclass('public.employment_relationships') IS NULL THEN
        RAISE EXCEPTION 'schema check failed: employment_relationships table is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.employment_relationships'::regclass
          AND conname = 'employment_relationships_dates_check'
          AND contype = 'c'
    ) THEN
        RAISE EXCEPTION 'schema check failed: relationship date constraint is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.employment_relationships'::regclass
          AND conname = 'employment_relationships_no_overlap'
          AND contype = 'x'
    ) THEN
        RAISE EXCEPTION 'schema check failed: relationship overlap exclusion constraint is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_attribute
        WHERE attrelid = 'public.employments'::regclass
          AND attname = 'relationship_id'
          AND NOT attisdropped
    ) OR NOT EXISTS (
        SELECT 1
        FROM pg_attribute
        WHERE attrelid = 'public.employee_contracts'::regclass
          AND attname = 'relationship_id'
          AND NOT attisdropped
    ) THEN
        RAISE EXCEPTION 'schema check failed: relationship compatibility columns are missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.employments'::regclass
          AND conname = 'employments_relationship_identity_fkey'
          AND contype = 'f'
    ) THEN
        RAISE EXCEPTION 'schema check failed: segment relationship identity FK is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_trigger
        WHERE tgrelid = 'public.employee_contracts'::regclass
          AND tgname = 'trg_validate_contract_relationship_link'
          AND NOT tgisinternal
    ) OR NOT EXISTS (
        SELECT 1
        FROM pg_trigger
        WHERE tgrelid = 'public.employments'::regclass
          AND tgname = 'trg_validate_segment_relationship_change'
          AND NOT tgisinternal
    ) THEN
        RAISE EXCEPTION 'schema check failed: relationship consistency triggers are missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.employee_contracts'::regclass
          AND conname = 'employee_contracts_relationship_id_fkey'
          AND contype = 'f'
    ) THEN
        RAISE EXCEPTION 'schema check failed: contract relationship FK is missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261112010000_employment_relationships';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: employment relationship migration marker is missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261112020000_employment_relationship_backfill';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: employment relationship backfill marker is missing';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.employments segment
        LEFT JOIN public.employment_relationships relationship
          ON relationship.id = segment.relationship_id
         AND relationship.employee_id = segment.employee_id
         AND relationship.employer_id = segment.employer_id
        WHERE segment.relationship_id IS NOT NULL
          AND relationship.id IS NULL
    ) OR EXISTS (
        SELECT 1
        FROM public.employee_contracts contract
        LEFT JOIN public.employments segment
          ON segment.id = contract.employment_id
         AND segment.relationship_id = contract.relationship_id
        WHERE contract.relationship_id IS NOT NULL
          AND segment.id IS NULL
    ) THEN
        RAISE EXCEPTION 'schema check failed: orphaned or mismatched relationship links exist';
    END IF;

    IF to_regclass('public.payroll_periods') IS NULL THEN
        RAISE EXCEPTION 'schema check failed: payroll_periods table is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.payroll_periods'::regclass
          AND conname = 'payroll_periods_unique_period'
          AND contype = 'u'
    ) THEN
        RAISE EXCEPTION 'schema check failed: payroll period uniqueness constraint is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.payroll_periods'::regclass
          AND conname = 'payroll_periods_status_check'
          AND contype = 'c'
    ) THEN
        RAISE EXCEPTION 'schema check failed: payroll period status constraint is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_trigger
        WHERE tgrelid = 'public.payroll_periods'::regclass
          AND tgname = 'trg_payroll_periods_updated_at'
          AND NOT tgisinternal
    ) THEN
        RAISE EXCEPTION 'schema check failed: payroll period updated_at trigger is missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261112030000_payroll_periods';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: payroll period migration marker is missing';
    END IF;

    RAISE NOTICE 'schema contract check: PASS';
END;
$$;
