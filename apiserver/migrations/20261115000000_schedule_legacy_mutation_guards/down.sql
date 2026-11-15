BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM schedule_day_revisions WHERE state IN ('published', 'explicit_off')) THEN
        RAISE EXCEPTION 'legacy mutation guard rollback blocked: official schedule evidence exists';
    END IF;
END;
$$;

DROP TRIGGER IF EXISTS trg_guard_legacy_shift_rosters ON public.shift_rosters;
DROP TRIGGER IF EXISTS trg_guard_legacy_employee_shifts ON public.employee_shifts;
DROP TRIGGER IF EXISTS trg_guard_legacy_employee_working_patterns ON public.employee_working_patterns;
DROP TRIGGER IF EXISTS trg_guard_legacy_working_pattern_details ON public.working_pattern_details;
DROP TRIGGER IF EXISTS trg_guard_legacy_shift_versions ON public.shift_versions;
DROP TRIGGER IF EXISTS trg_guard_legacy_shift_templates ON public.shift_templates;
DROP TRIGGER IF EXISTS trg_guard_legacy_assignment_rules ON public.assignment_rules;
DROP TRIGGER IF EXISTS trg_guard_legacy_working_patterns ON public.working_patterns;
DROP FUNCTION IF EXISTS public.guard_legacy_schedule_mutation();

DELETE FROM public.schema_migrations
WHERE version = '20261115000000_schedule_legacy_mutation_guards';

COMMIT;
