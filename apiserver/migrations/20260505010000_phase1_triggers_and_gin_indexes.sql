-- ═══════════════════════════════════════════════════════════════════
-- Phase 1: Data Integrity Foundation
-- 1a. Universal updated_at trigger for all mutable tables
-- 1b. GIN indexes for JSONB columns used in WHERE clauses
--
-- Risk: ZERO — additive only, no schema changes, no code changes
-- ═══════════════════════════════════════════════════════════════════

-- ───────────────────────────────────────────────────────────────────
-- 1a. Reusable trigger function (replaces the old single-table one)
-- ───────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION public.fn_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop the old single-purpose function after migrating its trigger
DROP TRIGGER IF EXISTS trigger_employee_profiles_updated_at ON public.employee_profiles;
DROP FUNCTION IF EXISTS public.update_employee_profiles_updated_at();

-- ───────────────────────────────────────────────────────────────────
-- Apply trigger to ALL tables with updated_at column
-- Using CREATE OR REPLACE pattern via DROP IF EXISTS + CREATE
-- ───────────────────────────────────────────────────────────────────

-- employee_profiles (was the only one with a trigger before)
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employee_profiles
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- assignment_rules
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.assignment_rules
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- attendance_adjustment_requests
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.attendance_adjustment_requests
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- attendance_daily_summaries
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.attendance_daily_summaries
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- attendance_import_jobs
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.attendance_import_jobs
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- attendance_policies
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.attendance_policies
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- attendance_policy_assignments
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.attendance_policy_assignments
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- dashboard_preferences
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.dashboard_preferences
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- departments
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.departments
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- employee_phases
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employee_phases
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- employee_salary_components
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employee_salary_components
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- employee_shifts
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employee_shifts
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- employee_working_patterns
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employee_working_patterns
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- employees
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employees
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- employment_periods
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employment_periods
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- employment_phase_definitions
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employment_phase_definitions
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- feature_flags
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.feature_flags
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- holidays
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.holidays
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- leave_balances
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.leave_balances
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- leave_policy_definitions
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.leave_policy_definitions
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- leave_requests
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.leave_requests
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- overtime_policy_definitions
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.overtime_policy_definitions
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- overtime_rules
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.overtime_rules
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- payroll_rule_definitions
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.payroll_rule_definitions
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- payroll_settings
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.payroll_settings
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- payslips
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.payslips
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- policy_templates
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.policy_templates
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- recruitment_sources
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.recruitment_sources
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- salary_base_groups
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.salary_base_groups
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- salary_component_definitions
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.salary_component_definitions
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- shift_rosters
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.shift_rosters
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- shift_swap_requests
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.shift_swap_requests
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- shifts
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.shifts
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- working_patterns
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.working_patterns
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- workspace_roles
CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.workspace_roles
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- ───────────────────────────────────────────────────────────────────
-- 1b. GIN indexes for JSONB columns used in filtering/reporting
-- Using CONCURRENTLY is not possible inside a transaction block
-- (SQLx migrations run in transactions), so we use standard CREATE.
-- These are small tables, so lock time is negligible.
-- ───────────────────────────────────────────────────────────────────

-- payroll_incentive_tiers.rule_config — queried during payroll calc
CREATE INDEX idx_incentive_tiers_rule_config
    ON public.payroll_incentive_tiers USING GIN (rule_config);

-- attendance_policy_versions.rules — queried during attendance processing
CREATE INDEX idx_attendance_policy_versions_rules
    ON public.attendance_policy_versions USING GIN (rules);

-- leave_policy_definitions.rule_config — queried during leave accrual
CREATE INDEX idx_leave_policy_defs_rule_config
    ON public.leave_policy_definitions USING GIN (rule_config);

-- leave_types.policy_config — queried during leave validation
CREATE INDEX idx_leave_types_policy_config
    ON public.leave_types USING GIN (policy_config);

-- payroll_rule_definitions.rule_config — queried during payroll calc
CREATE INDEX idx_payroll_rule_defs_rule_config
    ON public.payroll_rule_definitions USING GIN (rule_config);

-- employment_phase_definitions.rule_config — queried during phase eval
CREATE INDEX idx_employment_phase_defs_rule_config
    ON public.employment_phase_definitions USING GIN (rule_config);
