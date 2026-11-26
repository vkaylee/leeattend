-- Fresh phased-install schema contract.
-- Run after foundation, hr_core, attendance_schedule, payroll, runtime seeds.

DO $$
DECLARE
    marker_count integer;
    legacy_count integer;
    required_count integer;
BEGIN
    SELECT count(*) INTO marker_count
    FROM public.schema_migrations
    WHERE version IN (
        'fresh_foundation_v1',
        'fresh_hr_core_v1',
        'fresh_attendance_schedule_v1',
        'fresh_payroll_v1',
        'fresh_runtime_seeds_v1'
    );
    IF marker_count <> 5 THEN
        RAISE EXCEPTION 'phased schema check failed: expected 5 phase markers, found %', marker_count;
    END IF;

    SELECT count(*) INTO legacy_count
    FROM pg_class c
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = 'public'
      AND c.relname LIKE 'workspace%';
    IF legacy_count <> 0 THEN
        RAISE EXCEPTION 'phased schema check failed: workspace objects remain: %', legacy_count;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM (VALUES
            ('employment_segments'), ('employment_phases'),
            ('employment_location_assignments'), ('working_pattern_assignment_rules'),
            ('attendance_sources'), ('attendance_source_pull_states'),
            ('external_attendance_events'), ('payroll_incentive_assignments')
        ) AS required(name)
        WHERE NOT EXISTS (
            SELECT 1 FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public' AND c.relname = required.name AND c.relkind IN ('r', 'p')
        )
    ) THEN
        RAISE EXCEPTION 'phased schema check failed: renamed or roadmap table missing';
    END IF;

    IF to_regclass('public.employments') IS NOT NULL
       OR to_regclass('public.employee_phases') IS NOT NULL
       OR to_regclass('public.employee_assignments') IS NOT NULL
       OR to_regclass('public.assignment_rules') IS NOT NULL
       OR to_regclass('public.shifts') IS NOT NULL
       OR to_regclass('public.authorized_agents') IS NOT NULL
       OR to_regclass('public.policy_templates') IS NOT NULL
       OR to_regclass('public.system_audit_logs') IS NOT NULL
       OR to_regclass('public.system_files') IS NOT NULL THEN
        RAISE EXCEPTION 'phased schema check failed: retired or old table name remains';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.users'::regclass
          AND confrelid = 'public.files'::regclass
          AND conname = 'users_avatar_file_id_fkey'
          AND contype = 'f'
    ) THEN
        RAISE EXCEPTION 'phased schema check failed: users_avatar_file_id_fkey must reference public.files';
    END IF;

    IF to_regnamespace('salary_base_retirement_backup') IS NOT NULL THEN
        RAISE EXCEPTION 'phased schema check failed: upgrade-only retirement schema remains';
    END IF;

    SELECT count(*) INTO required_count
    FROM pg_class c
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = 'public'
      AND c.relname IN (
          'organization_settings',
          'users',
          'roles',
          'config_entries',
          'employment_relationships',
          'employee_tenures',
          'attendance_periods',
          'attendance_closures',
          'schedule_day_revisions',
          'payroll_periods',
          'payroll_metric_definitions',
          'payroll_formula_rules',
          'payroll_formula_rule_versions',
          'payroll_calculation_runs',
          'overtime_policy_versions',
          'work_policy_groups',
          'work_policy_versions',
          'work_policy_memberships',
          'schema_migrations'
      );
    IF required_count <> 19 THEN
        RAISE EXCEPTION 'phased schema check failed: required objects missing, found %', required_count;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'shift_rosters' AND column_name = 'shift_version_id' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'time_entries' AND column_name = 'shift_version_id' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'time_entries' AND column_name = 'breaks_snapshot' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'shift_rosters_shift_version_id_fkey')
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'time_entries_shift_version_id_fkey')
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid = 'public.attendance_daily_summaries'::regclass AND confrelid = 'public.shift_versions'::regclass AND conname = 'attendance_daily_summaries_shift_version_id_fkey' AND contype = 'f')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'attendance_closure_days' AND column_name = 'time_entry_snapshot' AND is_nullable = 'NO')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'shift_default_intervals' AND column_name = 'break_punch_mode')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'attendance_daily_summaries' AND column_name = 'scheduled_break_minutes')
       OR to_regclass('public.attendance_punch_events') IS NULL
       OR to_regclass('public.schedule_day_overrides') IS NULL
       OR to_regclass('public.attendance_adjustment_evidence') IS NULL
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'attendance_adjustment_requests' AND column_name = 'adjustment_type')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'attendance_adjustment_requests' AND column_name = 'details') THEN
        RAISE EXCEPTION 'phased schema check failed: shift policy snapshot columns, typed adjustments, or foreign keys missing';
    END IF;

    IF to_regclass('public.payroll_calculation_runs_policy_snapshot_idx') IS NOT NULL THEN
        RAISE EXCEPTION 'phased schema check failed: superseded policy snapshot index remains';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employee_profiles' AND column_name = 'national_id_lookup_hash' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employee_profiles' AND column_name = 'phone_lookup_hash' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid = 'public.employee_profiles'::regclass AND conname = 'employee_profiles_identity_lookup_hash_check' AND contype = 'c')
       OR NOT EXISTS (SELECT 1 FROM pg_index WHERE indexrelid = to_regclass('public.idx_employee_profiles_national_id_lookup_hash') AND NOT indisunique)
       OR NOT EXISTS (SELECT 1 FROM pg_index WHERE indexrelid = to_regclass('public.idx_employee_profiles_phone_lookup_hash') AND NOT indisunique) THEN
        RAISE EXCEPTION 'phased schema check failed: employee identity lookup hash columns, check, or non-unique partial indexes missing';
    END IF;

    -- Employee Tenures v2 single-source-of-truth invariants
    IF to_regclass('public.employee_tenure_reconciliation') IS NOT NULL THEN
        RAISE EXCEPTION 'phased schema check failed: transitional employee_tenure_reconciliation table remains';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'employees'
          AND column_name IN ('user_id', 'employee_id_number', 'hire_date', 'original_hire_date', 'termination_date', 'is_active')
    ) THEN
        RAISE EXCEPTION 'phased schema check failed: legacy dual-write columns remain on employees';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employee_tenures' AND column_name = 'employee_number_normalized' AND is_nullable = 'NO')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employee_tenures' AND column_name = 'hire_type' AND is_nullable = 'NO')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employee_tenures' AND column_name = 'prior_service_credit_days' AND is_nullable = 'NO')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employee_tenures' AND column_name = 'rehire_eligible' AND is_nullable = 'NO')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employment_relationships' AND column_name = 'tenure_id' AND is_nullable = 'NO')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employment_segments' AND column_name = 'relationship_id' AND is_nullable = 'NO')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'employee_documents' AND column_name = 'tenure_id')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'file_acknowledgements' AND column_name = 'tenure_id')
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid = 'public.employee_tenures'::regclass AND conname = 'employee_tenures_permanent_code_owner' AND contype = 'x')
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid = 'public.employment_relationships'::regclass AND tgname = 'trg_validate_relationship_tenure_bounds' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid = 'public.employment_segments'::regclass AND tgname = 'trg_validate_segment_relationship_bounds' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid = 'public.employee_tenures'::regclass AND tgname = 'trg_validate_tenure_bounds_guard' AND NOT tgisinternal) THEN
        RAISE EXCEPTION 'phased schema check failed: employee tenures v2 columns, constraints, or containment triggers missing';
    END IF;

    -- Work Policy v2 single-source-of-truth invariants
    IF to_regclass('public.labor_policy_groups') IS NOT NULL
       OR to_regclass('public.labor_policy_group_versions') IS NOT NULL
       OR to_regclass('public.labor_policy_memberships') IS NOT NULL
       OR to_regclass('public.work_credit_policies') IS NOT NULL
       OR to_regclass('public.pay_basis_policies') IS NOT NULL
       OR to_regclass('public.attendance_policies') IS NOT NULL
       OR to_regclass('public.attendance_policy_versions') IS NOT NULL
       OR to_regclass('public.attendance_policy_assignments') IS NOT NULL
       OR to_regclass('public.overtime_policy_assignments') IS NOT NULL
       OR to_regclass('public.overtime_rules') IS NOT NULL
       OR to_regclass('public.department_payroll_reconciliation') IS NOT NULL THEN
        RAISE EXCEPTION 'phased schema check failed: legacy competing work policy tables or views remain';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid = 'public.work_policy_versions'::regclass AND conname = 'work_policy_versions_published_no_overlap' AND contype = 'x')
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid = 'public.work_policy_memberships'::regclass AND conname = 'work_policy_memberships_effective_no_overlap' AND contype = 'x')
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid = 'public.overtime_policy_versions'::regclass AND tgname = 'trg_guard_overtime_policy_version_immutability' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid = 'public.work_policy_versions'::regclass AND tgname = 'trg_guard_work_policy_version_immutability' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid = 'public.work_policy_memberships'::regclass AND tgname = 'trg_work_policy_memberships_rel_bounds' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid = 'public.employment_relationships'::regclass AND tgname = 'trg_employment_relationships_policy_bounds_guard' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid = 'public.attendance_daily_summaries'::regclass AND conname = 'attendance_daily_summaries_work_policy_version_id_fkey' AND contype = 'f')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'payslip_segments' AND column_name = 'work_policy_version_id')
       OR to_regclass('public.payslip_segments_work_policy_version_idx') IS NULL THEN
        RAISE EXCEPTION 'phased schema check failed: work policy v2 constraints, triggers, or provenance links missing';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM public.feature_flags WHERE flag_key = 'ff_econtracts' AND enabled = false)
       OR NOT EXISTS (SELECT 1 FROM public.feature_flags WHERE flag_key = 'ff_ess' AND enabled = false) THEN
        RAISE EXCEPTION 'phased schema check failed: default feature flags missing';
    END IF;

    IF (SELECT count(*) FROM public.payroll_metric_definitions WHERE code IN (
        'production.output', 'production.target_output', 'production.quality_rate'
    )) <> 3 THEN
        RAISE EXCEPTION 'phased schema check failed: payroll metric seeds missing';
    END IF;

    RAISE NOTICE 'phased schema contract: PASS';
END;
$$;
