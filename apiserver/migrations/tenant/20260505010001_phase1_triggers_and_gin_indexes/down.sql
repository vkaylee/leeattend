-- ═══════════════════════════════════════════════════════════════════
-- Rollback: Phase 1 triggers and GIN indexes
-- ═══════════════════════════════════════════════════════════════════

-- ───────────────────────────────────────────────────────────────────
-- Drop GIN indexes
-- ───────────────────────────────────────────────────────────────────
DROP INDEX IF EXISTS public.idx_employment_phase_defs_rule_config;

DROP INDEX IF EXISTS public.idx_payroll_rule_defs_rule_config;

DROP INDEX IF EXISTS public.idx_leave_types_policy_config;

DROP INDEX IF EXISTS public.idx_leave_policy_defs_rule_config;

DROP INDEX IF EXISTS public.idx_attendance_policy_versions_rules;

DROP INDEX IF EXISTS public.idx_incentive_tiers_rule_config;

-- ───────────────────────────────────────────────────────────────────
-- Drop all trg_updated_at triggers
-- ───────────────────────────────────────────────────────────────────
DROP TRIGGER IF EXISTS trg_updated_at ON public.working_patterns;

DROP TRIGGER IF EXISTS trg_updated_at ON public.shifts;

DROP TRIGGER IF EXISTS trg_updated_at ON public.shift_swap_requests;

DROP TRIGGER IF EXISTS trg_updated_at ON public.shift_rosters;

DROP TRIGGER IF EXISTS trg_updated_at ON public.salary_component_definitions;

DROP TRIGGER IF EXISTS trg_updated_at ON public.salary_base_groups;

DROP TRIGGER IF EXISTS trg_updated_at ON public.recruitment_sources;

DROP TRIGGER IF EXISTS trg_updated_at ON public.policy_templates;

DROP TRIGGER IF EXISTS trg_updated_at ON public.payslips;

DROP TRIGGER IF EXISTS trg_updated_at ON public.payroll_settings;

DROP TRIGGER IF EXISTS trg_updated_at ON public.payroll_rule_definitions;

DROP TRIGGER IF EXISTS trg_updated_at ON public.overtime_rules;

DROP TRIGGER IF EXISTS trg_updated_at ON public.overtime_policy_definitions;

DROP TRIGGER IF EXISTS trg_updated_at ON public.leave_requests;

DROP TRIGGER IF EXISTS trg_updated_at ON public.leave_policy_definitions;

DROP TRIGGER IF EXISTS trg_updated_at ON public.leave_balances;

DROP TRIGGER IF EXISTS trg_updated_at ON public.holidays;

DROP TRIGGER IF EXISTS trg_updated_at ON public.feature_flags;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employment_phase_definitions;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employment_periods;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employees;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employee_working_patterns;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employee_shifts;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employee_salary_components;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employee_phases;

DROP TRIGGER IF EXISTS trg_updated_at ON public.departments;

DROP TRIGGER IF EXISTS trg_updated_at ON public.dashboard_preferences;

DROP TRIGGER IF EXISTS trg_updated_at ON public.attendance_policy_assignments;

DROP TRIGGER IF EXISTS trg_updated_at ON public.attendance_policies;

DROP TRIGGER IF EXISTS trg_updated_at ON public.attendance_import_jobs;

DROP TRIGGER IF EXISTS trg_updated_at ON public.attendance_daily_summaries;

DROP TRIGGER IF EXISTS trg_updated_at ON public.attendance_adjustment_requests;

DROP TRIGGER IF EXISTS trg_updated_at ON public.assignment_rules;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employee_profiles;

-- ───────────────────────────────────────────────────────────────────
-- Drop the universal function and restore the old one
-- ───────────────────────────────────────────────────────────────────
DROP FUNCTION IF EXISTS public.fn_set_updated_at();

-- Restore original single-purpose function and trigger
CREATE FUNCTION public.update_employee_profiles_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();

RETURN NEW;

END;

$$;

CREATE TRIGGER trigger_employee_profiles_updated_at
    BEFORE UPDATE ON public.employee_profiles
    FOR EACH ROW EXECUTE FUNCTION public.update_employee_profiles_updated_at();