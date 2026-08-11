-- Restore the trigger/function definition owned by the preceding migration.
-- The old migration remains recorded for upgraded tenants, so rolling back this
-- follow-up must leave the original invalidation behavior installed.

DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_shift_templates ON public.shift_templates;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_shift_versions ON public.shift_versions;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_employee_shifts ON public.employee_shifts;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_employee_working_patterns ON public.employee_working_patterns;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_working_pattern_details ON public.working_pattern_details;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_working_patterns ON public.working_patterns;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_assignment_rules ON public.assignment_rules;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_employments ON public.employments;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_employees ON public.employees;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_shift_default_intervals ON public.shift_default_intervals;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_work_segments ON public.work_segments;

CREATE OR REPLACE FUNCTION public.enqueue_roster_cache_invalidation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = public
AS $$
DECLARE
    changed_row JSONB;
    workspace_id_value UUID;
    previous_workspace_id UUID;
BEGIN
    FOR changed_row IN
        SELECT value
        FROM jsonb_array_elements(
            CASE
                WHEN TG_OP = 'INSERT' THEN jsonb_build_array(to_jsonb(NEW))
                WHEN TG_OP = 'DELETE' THEN jsonb_build_array(to_jsonb(OLD))
                ELSE jsonb_build_array(to_jsonb(OLD), to_jsonb(NEW))
            END
        )
    LOOP
        workspace_id_value := NULL;

        IF TG_ARGV[0] = 'direct' THEN
            workspace_id_value := NULLIF(changed_row ->> 'workspace_id', '')::UUID;
        ELSIF TG_ARGV[0] = 'employment' THEN
            SELECT e.workspace_id
            INTO workspace_id_value
            FROM public.employments AS e
            WHERE e.id = NULLIF(changed_row ->> 'employment_id', '')::UUID;
        ELSIF TG_ARGV[0] = 'working_pattern' THEN
            SELECT wp.workspace_id
            INTO workspace_id_value
            FROM public.working_patterns AS wp
            WHERE wp.id = NULLIF(changed_row ->> 'pattern_id', '')::UUID;
        ELSIF TG_ARGV[0] = 'shift_interval' THEN
            SELECT st.workspace_id
            INTO workspace_id_value
            FROM public.shift_versions AS sv
            INNER JOIN public.shift_templates AS st ON st.id = sv.template_id
            WHERE sv.id = NULLIF(changed_row ->> 'version_id', '')::UUID;
        END IF;

        IF workspace_id_value IS NOT NULL
            AND workspace_id_value IS DISTINCT FROM previous_workspace_id
        THEN
            INSERT INTO public.outbox_jobs (
                id,
                workspace_id,
                job_type,
                payload,
                status,
                error_message,
                created_at,
                updated_at
            )
            VALUES (
                gen_random_uuid(),
                workspace_id_value,
                'RosterCacheInvalidation',
                '{}'::JSONB,
                'pending',
                NULL,
                CURRENT_TIMESTAMP,
                CURRENT_TIMESTAMP
            );
            previous_workspace_id := workspace_id_value;
        END IF;
    END LOOP;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_roster_cache_invalidation_shift_templates
AFTER INSERT OR UPDATE OR DELETE ON public.shift_templates
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_roster_cache_invalidation_shift_versions
AFTER INSERT OR UPDATE OR DELETE ON public.shift_versions
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_roster_cache_invalidation_employee_shifts
AFTER INSERT OR UPDATE OR DELETE ON public.employee_shifts
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('employment');

CREATE TRIGGER trg_roster_cache_invalidation_employee_working_patterns
AFTER INSERT OR UPDATE OR DELETE ON public.employee_working_patterns
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_roster_cache_invalidation_working_pattern_details
AFTER INSERT OR UPDATE OR DELETE ON public.working_pattern_details
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('working_pattern');

CREATE TRIGGER trg_roster_cache_invalidation_working_patterns
AFTER INSERT OR UPDATE OR DELETE ON public.working_patterns
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_roster_cache_invalidation_assignment_rules
AFTER INSERT OR UPDATE OR DELETE ON public.assignment_rules
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_roster_cache_invalidation_employments
AFTER INSERT OR UPDATE OR DELETE ON public.employments
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_roster_cache_invalidation_employees
AFTER INSERT OR UPDATE OR DELETE ON public.employees
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_roster_cache_invalidation_shift_default_intervals
AFTER INSERT OR UPDATE OR DELETE ON public.shift_default_intervals
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('shift_interval');

CREATE TRIGGER trg_roster_cache_invalidation_work_segments
AFTER INSERT OR UPDATE OR DELETE ON public.work_segments
FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');
