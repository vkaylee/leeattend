-- Fresh-baseline schema contract check.
-- Run after baseline installation through the repository migration wrapper.

DO $$
DECLARE
    v_legacy_col_count integer;
    v_legacy_object_count integer;
    v_required_table_count integer;
    v_required_table_name text;
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

    IF EXISTS (
        SELECT 1
        FROM pg_attribute a
        WHERE a.attrelid = 'public.organization_settings'::regclass
          AND a.attname = 'custom_db_url_encrypted'
          AND NOT a.attisdropped
    ) THEN
        RAISE EXCEPTION 'baseline schema check failed: obsolete custom database routing field remains';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261110000000_single_organization_baseline';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: baseline migration marker is missing';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'shift_rosters' AND column_name = 'shift_version_id' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'time_entries' AND column_name = 'shift_version_id' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'time_entries' AND column_name = 'breaks_snapshot' AND is_nullable = 'YES') THEN
        RAISE EXCEPTION 'baseline schema check failed: shift policy snapshot columns missing';
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

    IF to_regclass('public.employee_tenures') IS NULL
       OR to_regclass('public.employee_tenure_reconciliation') IS NULL THEN
        RAISE EXCEPTION 'schema check failed: employee tenure tables are missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.employee_tenures'::regclass
          AND conname = 'employee_tenures_dates_check'
          AND contype = 'c'
    ) OR NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.employee_tenures'::regclass
          AND conname = 'employee_tenures_no_overlap_person'
          AND contype = 'x'
    ) OR NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.employee_tenures'::regclass
          AND conname = 'employee_tenures_no_overlap_number'
          AND contype = 'x'
    ) THEN
        RAISE EXCEPTION 'schema check failed: employee tenure temporal constraints are missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_attribute
        WHERE attrelid = 'public.employment_relationships'::regclass
          AND attname = 'tenure_id'
          AND NOT attisdropped
    ) OR NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.employment_relationships'::regclass
          AND conname = 'employment_relationships_tenure_identity_fkey'
          AND contype = 'f'
    ) THEN
        RAISE EXCEPTION 'schema check failed: relationship tenure link is missing';
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

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261115030000_employee_tenures';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: employee tenure migration marker is missing';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.employee_tenure_reconciliation
        WHERE status NOT IN ('pending', 'resolved', 'blocked')
    ) THEN
        RAISE EXCEPTION 'schema check failed: invalid tenure reconciliation status exists';
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

    FOREACH v_required_table_name IN ARRAY ARRAY[
        'schedule_day_revisions'::text,
        'schedule_publication_batches'::text,
        'schedule_legacy_mutation_evidence'::text
    ] LOOP
        IF to_regclass('public.' || v_required_table_name) IS NULL THEN
            RAISE EXCEPTION 'schema check failed: schedule lifecycle table % is missing', v_required_table_name;
        END IF;
    END LOOP;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261115000000_schedule_legacy_mutation_guards';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: legacy schedule guard migration marker is missing';
    END IF;

    FOREACH v_required_table_name IN ARRAY ARRAY[
        'trg_guard_legacy_shift_rosters'::text,
        'trg_guard_legacy_employee_shifts'::text,
        'trg_guard_legacy_employee_working_patterns'::text,
        'trg_guard_legacy_working_pattern_details'::text,
        'trg_guard_legacy_shift_versions'::text,
        'trg_guard_legacy_shift_templates'::text,
        'trg_guard_legacy_assignment_rules'::text,
        'trg_guard_legacy_working_patterns'::text
    ] LOOP
        IF NOT EXISTS (
            SELECT 1 FROM pg_trigger
            WHERE tgname = v_required_table_name
              AND NOT tgisinternal
        ) THEN
            RAISE EXCEPTION 'schema check failed: legacy schedule trigger % is missing', v_required_table_name;
        END IF;
    END LOOP;

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

    FOREACH v_required_table_name IN ARRAY ARRAY[
        'work_credit_policies'::text,
        'pay_basis_policies'::text,
        'labor_policy_groups'::text,
        'labor_policy_group_versions'::text,
        'labor_policy_memberships'::text
    ] LOOP
        IF to_regclass('public.' || v_required_table_name) IS NULL THEN
            RAISE EXCEPTION 'schema check failed: work policy table % is missing', v_required_table_name;
        END IF;
    END LOOP;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.labor_policy_group_versions'::regclass
          AND conname = 'labor_policy_group_versions_effective_no_overlap'
          AND contype = 'x'
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.labor_policy_memberships'::regclass
          AND conname = 'labor_policy_memberships_effective_no_overlap'
          AND contype = 'x'
    ) THEN
        RAISE EXCEPTION 'schema check failed: work policy overlap constraints are missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261114080000_labor_policy_groups';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: labor policy migration marker is missing';
    END IF;

    IF to_regclass('public.department_payroll_reconciliation') IS NULL THEN
        RAISE EXCEPTION 'schema check failed: department payroll reconciliation view is missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261115010000_department_payroll_reconciliation';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: department payroll reconciliation migration marker is missing';
    END IF;

    IF to_regclass('public.payroll_calculation_runs_policy_snapshot_idx') IS NOT NULL THEN
        RAISE EXCEPTION 'schema check failed: policy-only payroll snapshot index must be removed for append-only history';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261114090000_payroll_calculation_run_policy_snapshot';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: payroll policy snapshot migration marker is missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261115050000_append_only_payroll_policy_snapshots';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: append-only payroll policy snapshot migration marker is missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_attribute
        WHERE attrelid = 'public.payslip_segments'::regclass
          AND attname = 'labor_policy_group_version_id'
          AND NOT attisdropped
    ) OR to_regclass('public.payslip_segments_labor_policy_version_idx') IS NULL THEN
        RAISE EXCEPTION 'schema check failed: payslip segment labor policy snapshot storage is missing';
    END IF;

    IF to_regclass('public.shift_swap_requests_official_revisions_idx') IS NULL
       OR NOT EXISTS (
           SELECT 1 FROM pg_constraint
           WHERE conrelid = 'public.shift_swap_requests'::regclass
             AND conname = 'shift_swap_requests_requester_revision_fkey'
             AND contype = 'f'
       )
       OR NOT EXISTS (
           SELECT 1 FROM pg_constraint
           WHERE conrelid = 'public.shift_swap_requests'::regclass
             AND conname = 'shift_swap_requests_target_revision_fkey'
             AND contype = 'f'
       ) THEN
        RAISE EXCEPTION 'schema check failed: official shift swap revision contract is missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261112030000_payroll_periods';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: payroll period migration marker is missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261116000000_atomic_official_shift_swaps';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: official shift swap migration marker is missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261120000000_shift_policy_snapshots';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: shift policy snapshot migration marker is missing';
    END IF;

    IF to_regclass('public.schedule_day_overrides') IS NULL
       OR to_regclass('public.attendance_adjustment_evidence') IS NULL
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'attendance_adjustment_requests' AND column_name = 'adjustment_type')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'attendance_adjustment_requests' AND column_name = 'details') THEN
        RAISE EXCEPTION 'schema check failed: schedule override or typed attendance adjustment tables are missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'attendance_closure_days'
          AND column_name = 'time_entry_snapshot' AND is_nullable = 'NO'
    ) OR to_regclass('public.attendance_punch_events') IS NULL OR NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'shift_default_intervals'
          AND column_name = 'break_punch_mode'
    ) THEN
        RAISE EXCEPTION 'schema check failed: attendance policy snapshot columns missing';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'employee_profiles'
          AND column_name = 'national_id_lookup_hash' AND is_nullable = 'YES'
    ) OR NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'employee_profiles'
          AND column_name = 'phone_lookup_hash' AND is_nullable = 'YES'
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'public.employee_profiles'::regclass
          AND conname = 'employee_profiles_identity_lookup_hash_check'
          AND contype = 'c'
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_index
        WHERE indexrelid = to_regclass('public.idx_employee_profiles_national_id_lookup_hash')
          AND NOT indisunique
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_index
        WHERE indexrelid = to_regclass('public.idx_employee_profiles_phone_lookup_hash')
          AND NOT indisunique
    ) THEN
        RAISE EXCEPTION 'schema check failed: employee identity lookup hash columns, check, or non-unique partial indexes missing';
    END IF;

    SELECT count(*) INTO v_marker_count
    FROM public.schema_migrations
    WHERE version = '20261125000000_employee_identity_matching';

    IF v_marker_count <> 1 THEN
        RAISE EXCEPTION 'schema check failed: employee identity matching migration marker is missing';
    END IF;

    RAISE NOTICE 'schema contract check: PASS';
END;
$$;
