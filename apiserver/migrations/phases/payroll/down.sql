BEGIN;

-- Fresh-install-only rollback for payroll; run phases in reverse order.

-- This is destructive and must not run against production data.

DROP TRIGGER IF EXISTS trg_employment_relationships_policy_bounds_guard ON public.employment_relationships;

ALTER TABLE IF EXISTS public.attendance_daily_summaries
    DROP CONSTRAINT IF EXISTS attendance_daily_summaries_work_policy_version_id_fkey;

DROP INDEX IF EXISTS public.attendance_daily_summaries_policy_provenance_idx;

DROP TABLE IF EXISTS public.payslips CASCADE;

DROP TABLE IF EXISTS public.payslip_segments CASCADE;

DROP TABLE IF EXISTS public.payslip_rule_evaluations CASCADE;

DROP TABLE IF EXISTS public.payslip_incentives CASCADE;

DROP TABLE IF EXISTS public.payroll_rule_tiers CASCADE;

DROP TABLE IF EXISTS public.payroll_rule_definitions CASCADE;

DROP TABLE IF EXISTS public.payroll_rule_assignments CASCADE;

DROP TABLE IF EXISTS public.payroll_metric_definitions CASCADE;

DROP TABLE IF EXISTS public.payroll_inputs CASCADE;

DROP TABLE IF EXISTS public.payroll_input_corrections CASCADE;

DROP TABLE IF EXISTS public.payroll_input_batches CASCADE;

DROP TABLE IF EXISTS public.payroll_incentive_tiers CASCADE;

DROP TABLE IF EXISTS public.payroll_incentive_policies CASCADE;

DROP TABLE IF EXISTS public.payroll_incentive_assignments CASCADE;

DROP TABLE IF EXISTS public.payroll_formula_rules CASCADE;

DROP TABLE IF EXISTS public.payroll_formula_rule_versions CASCADE;

DROP TABLE IF EXISTS public.payroll_calculation_runs CASCADE;

DROP TABLE IF EXISTS public.work_policy_memberships CASCADE;

DROP TABLE IF EXISTS public.work_policy_versions CASCADE;

DROP TABLE IF EXISTS public.work_policy_groups CASCADE;

DROP FUNCTION IF EXISTS public.fn_validate_relationship_policy_bounds_guard();

DROP FUNCTION IF EXISTS public.fn_validate_work_policy_membership_rel_bounds();

DROP FUNCTION IF EXISTS public.fn_guard_work_policy_version_immutability();

DELETE FROM public.schema_migrations WHERE version = 'fresh_payroll_v1';

COMMIT;
