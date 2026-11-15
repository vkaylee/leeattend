BEGIN;

CREATE OR REPLACE FUNCTION public.guard_legacy_schedule_mutation() RETURNS trigger
LANGUAGE plpgsql
SET search_path TO 'public'
AS $$
DECLARE
    blocked boolean := false;
    new_data jsonb;
    old_data jsonb;
    employment_text text;
    start_text text;
    end_text text;
BEGIN
    new_data := CASE WHEN TG_OP IN ('INSERT', 'UPDATE') THEN to_jsonb(NEW) ELSE '{}'::jsonb END;
    old_data := CASE WHEN TG_OP IN ('UPDATE', 'DELETE') THEN to_jsonb(OLD) ELSE '{}'::jsonb END;

    IF TG_TABLE_NAME IN ('shift_rosters', 'employee_shifts', 'employee_working_patterns') THEN
        employment_text := COALESCE(new_data->>'employment_id', old_data->>'employment_id');
        start_text := COALESCE(new_data->>'start_date', new_data->>'date', old_data->>'start_date', old_data->>'date');
        end_text := COALESCE(new_data->>'end_date', new_data->>'date', old_data->>'end_date', old_data->>'date');
        SELECT EXISTS (
            SELECT 1
            FROM schedule_day_revisions r
            WHERE r.state IN ('published', 'explicit_off')
              AND r.employment_id = employment_text::uuid
              AND (start_text IS NULL OR r.business_date >= start_text::date)
              AND (end_text IS NULL OR r.business_date <= end_text::date)
        ) INTO blocked;
    ELSIF TG_TABLE_NAME = 'working_pattern_details' THEN
        SELECT EXISTS (
            SELECT 1
            FROM employee_working_patterns a
            JOIN schedule_day_revisions r ON r.employment_id = a.employment_id
            WHERE a.pattern_id = COALESCE(new_data->>'pattern_id', old_data->>'pattern_id')::uuid
              AND r.state IN ('published', 'explicit_off')
        ) INTO blocked;
    ELSIF TG_TABLE_NAME = 'shift_versions' THEN
        SELECT EXISTS (
            SELECT 1
            FROM work_segments ws
            JOIN schedule_day_revisions r ON r.id = ws.schedule_revision_id
            WHERE ws.template_id = COALESCE(NEW.template_id, OLD.template_id)
              AND r.state IN ('published', 'explicit_off')
        ) INTO blocked;
    ELSIF TG_TABLE_NAME = 'shift_templates' THEN
        SELECT EXISTS (
            SELECT 1
            FROM work_segments ws
            JOIN schedule_day_revisions r ON r.id = ws.schedule_revision_id
            WHERE ws.template_id = COALESCE(NEW.id, OLD.id)
              AND r.state IN ('published', 'explicit_off')
        ) INTO blocked;
    ELSIF TG_TABLE_NAME IN ('assignment_rules', 'working_patterns') THEN
        SELECT EXISTS (
            SELECT 1
            FROM employee_working_patterns ewp
            JOIN schedule_day_revisions r ON r.employment_id = ewp.employment_id
            JOIN employments em ON em.id = ewp.employment_id AND em.deleted_at IS NULL
            JOIN config_entries c
              ON c.scope_type = 'employer'
             AND c.scope_id = em.employer_id
             AND c.config_key = 'schedule.lifecycle.enabled'
             AND c.value_bool = true
             AND c.effective_from <= r.business_date
             AND (c.effective_to IS NULL OR c.effective_to >= r.business_date)
            CROSS JOIN feature_flags ff
            WHERE ff.flag_key = 'schedule_lifecycle'
              AND ff.enabled = true
              AND ewp.pattern_id = (CASE
                    WHEN TG_TABLE_NAME = 'assignment_rules' THEN COALESCE(new_data->>'pattern_id', old_data->>'pattern_id')
                    ELSE COALESCE(new_data->>'id', old_data->>'id')
                  END)::uuid
              AND r.state IN ('published', 'explicit_off')
              AND r.business_date >= ewp.start_date
              AND (ewp.end_date IS NULL OR r.business_date <= ewp.end_date)
        ) INTO blocked;
    END IF;

    IF blocked THEN
        RAISE EXCEPTION 'legacy schedule source mutation blocked after official publication';
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE TRIGGER trg_guard_legacy_shift_rosters
BEFORE INSERT OR UPDATE OR DELETE ON public.shift_rosters
FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();

CREATE TRIGGER trg_guard_legacy_employee_shifts
BEFORE INSERT OR UPDATE OR DELETE ON public.employee_shifts
FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();

CREATE TRIGGER trg_guard_legacy_employee_working_patterns
BEFORE INSERT OR UPDATE OR DELETE ON public.employee_working_patterns
FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();

CREATE TRIGGER trg_guard_legacy_working_pattern_details
BEFORE INSERT OR UPDATE OR DELETE ON public.working_pattern_details
FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();

CREATE TRIGGER trg_guard_legacy_shift_versions
BEFORE INSERT OR UPDATE OR DELETE ON public.shift_versions
FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();

CREATE TRIGGER trg_guard_legacy_shift_templates
BEFORE INSERT OR UPDATE OR DELETE ON public.shift_templates
FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();

CREATE TRIGGER trg_guard_legacy_assignment_rules
BEFORE INSERT OR UPDATE OR DELETE ON public.assignment_rules
FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();

CREATE TRIGGER trg_guard_legacy_working_patterns
BEFORE INSERT OR UPDATE OR DELETE ON public.working_patterns
FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();

INSERT INTO public.schema_migrations(version)
VALUES ('20261115000000_schedule_legacy_mutation_guards');

COMMIT;
