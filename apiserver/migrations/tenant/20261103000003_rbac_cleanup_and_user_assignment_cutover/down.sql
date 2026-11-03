-- Revert constraints
ALTER TABLE public.leave_policy_assignments DROP CONSTRAINT IF EXISTS leave_policy_assignments_assignee_type_check;
ALTER TABLE public.leave_policy_assignments ADD CONSTRAINT leave_policy_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text])));

ALTER TABLE public.overtime_policy_assignments DROP CONSTRAINT IF EXISTS overtime_policy_assignments_assignee_type_check;
ALTER TABLE public.overtime_policy_assignments ADD CONSTRAINT overtime_policy_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text])));

ALTER TABLE public.payroll_incentive_assignments DROP CONSTRAINT IF EXISTS payroll_incentive_assignments_assignee_type_check;
ALTER TABLE public.payroll_incentive_assignments ADD CONSTRAINT payroll_incentive_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text])));

CREATE OR REPLACE FUNCTION public.check_assignment_target_exists() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.assignee_type = 'workspace' THEN
        RETURN NEW;
    END IF;

    IF NEW.assignee_id IS NULL THEN
        RAISE EXCEPTION 'assignee_id cannot be NULL when assignee_type is %', NEW.assignee_type;
    END IF;

    IF NEW.assignee_type = 'department' THEN
        IF NOT EXISTS (SELECT 1 FROM public.departments WHERE id = NEW.assignee_id) THEN
            RAISE EXCEPTION 'Invalid assignee_id: department % does not exist', NEW.assignee_id;
        END IF;
    ELSIF NEW.assignee_type = 'employee' THEN
        IF NOT EXISTS (SELECT 1 FROM public.employees WHERE id = NEW.assignee_id) THEN
            RAISE EXCEPTION 'Invalid assignee_id: employee % does not exist', NEW.assignee_id;
        END IF;
    ELSE
        RAISE EXCEPTION 'Unknown assignee_type: %', NEW.assignee_type;
    END IF;

    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION public.check_attendance_target_exists() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.target_id IS NULL THEN
        RAISE EXCEPTION 'target_id cannot be NULL';
    END IF;

    IF NEW.target_type = 'USER' THEN
        IF NOT EXISTS (SELECT 1 FROM public.employees WHERE id = NEW.target_id) THEN
            RAISE EXCEPTION 'Invalid target_id: employee % does not exist', NEW.target_id;
        END IF;
    ELSIF NEW.target_type = 'GROUP' THEN
        IF NOT EXISTS (SELECT 1 FROM public.departments WHERE id = NEW.target_id) THEN
            RAISE EXCEPTION 'Invalid target_id: department % does not exist', NEW.target_id;
        END IF;
    ELSIF NEW.target_type = 'EMPLOYMENT' THEN
        IF NOT EXISTS (SELECT 1 FROM public.employments WHERE id = NEW.target_id) THEN
            RAISE EXCEPTION 'Invalid target_id: employment % does not exist', NEW.target_id;
        END IF;
    ELSE
        RAISE EXCEPTION 'Unknown target_type: %', NEW.target_type;
    END IF;

    RETURN NEW;
END;
$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'member_role_type') THEN
        CREATE TYPE public.member_role_type AS ENUM (
            'owner',
            'admin',
            'member',
            'hr_manager',
            'shift_manager',
            'team_lead',
            'employee'
        );
    END IF;
END $$;
