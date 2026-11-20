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
          'department_payroll_reconciliation',
          'schema_migrations'
      );
    IF required_count <> 16 THEN
        RAISE EXCEPTION 'phased schema check failed: required objects missing, found %', required_count;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'shift_rosters' AND column_name = 'shift_version_id' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'time_entries' AND column_name = 'shift_version_id' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'time_entries' AND column_name = 'breaks_snapshot' AND is_nullable = 'YES')
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'shift_rosters_shift_version_id_fkey')
       OR NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'time_entries_shift_version_id_fkey') THEN
        RAISE EXCEPTION 'phased schema check failed: shift policy snapshot columns or foreign keys missing';
    END IF;

    IF to_regclass('public.payroll_calculation_runs_policy_snapshot_idx') IS NOT NULL THEN
        RAISE EXCEPTION 'phased schema check failed: superseded policy snapshot index remains';
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
