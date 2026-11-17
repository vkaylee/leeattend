BEGIN;

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;


-- Fresh install phase: foundation

-- Name: btree_gist; Type: EXTENSION; Schema: -; Owner: -

--



CREATE EXTENSION IF NOT EXISTS btree_gist WITH SCHEMA public;





--

-- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -

--



CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;





--

-- Name: adjustment_status_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.adjustment_status_type AS ENUM (

    'pending',

    'approved',

    'rejected'

);





--

-- Name: assignment_history_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.assignment_history_type AS ENUM (

    'roster',

    'pattern',

    'department_pattern',

    'fixed',

    'department_fixed'

);





--

-- Name: assignment_status; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.assignment_status AS ENUM (

    'active',

    'completed',

    'cancelled'

);





--

-- Name: contract_status_enum; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.contract_status_enum AS ENUM (

    'draft',

    'active',

    'expired',

    'terminated'

);





--

-- Name: contract_type_enum; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.contract_type_enum AS ENUM (

    'probation',

    'training',

    'fixed_term',

    'indefinite'

);





--

-- Name: early_arrival_strategy_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.early_arrival_strategy_type AS ENUM (

    'allow',

    'block',

    'clamp'

);





--

-- Name: gap_compensation_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.gap_compensation_type AS ENUM (

    'unpaid',

    'standby',

    'break'

);





--

-- Name: import_job_status_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.import_job_status_type AS ENUM (

    'pending',

    'processing',

    'completed',

    'failed',

    'partially_failed'

);





--

-- Name: leave_status_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.leave_status_type AS ENUM (

    'pending',

    'approved',

    'rejected',

    'cancelled',

    'pre_approved'

);





--

-- Name: leave_transaction_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.leave_transaction_type AS ENUM (

    'accrual',

    'usage',

    'adjustment',

    'carry_over'

);





--

-- Name: log_source_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.log_source_type AS ENUM (

    'import',

    'manual'

);





--

-- Name: ot_aggregation_mode_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.ot_aggregation_mode_type AS ENUM (

    'aggregate',

    'per_segment'

);





--

-- Name: ot_request_status_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.ot_request_status_type AS ENUM (

    'draft',

    'pending_approval',

    'approved',

    'partially_accepted',

    'active',

    'completed',

    'cancelled',

    'rejected'

);





--

-- Name: ot_status_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.ot_status_type AS ENUM (

    'none',

    'pending_employee',

    'pending_manager',

    'approved',

    'rejected'

);





--

-- Name: recruitment_source_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.recruitment_source_type AS ENUM (

    'direct',

    'referral',

    'agency'

);





--

-- Name: segment_status_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.segment_status_type AS ENUM (

    'scheduled',

    'in_progress',

    'completed',

    'cancelled'

);





--

-- Name: segment_type; Type: TYPE; Schema: public; Owner: -

--



CREATE TYPE public.segment_type AS ENUM (

    'work',

    'standby',

    'on_call',

    'training',

    'travel'

);





--

-- Name: check_assignment_target_exists(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.check_assignment_target_exists() RETURNS trigger

    LANGUAGE plpgsql

    AS $$



    BEGIN



        IF NEW.assignee_type = 'organization' THEN



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



        ELSIF NEW.assignee_type = 'user' THEN



            RETURN NEW;



        ELSE



            RAISE EXCEPTION 'Unknown assignee_type: %', NEW.assignee_type;



        END IF;



        RETURN NEW;



    END;



    $$;





--

-- Name: check_attendance_target_exists(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.check_attendance_target_exists() RETURNS trigger

    LANGUAGE plpgsql

    AS $$



BEGIN



    IF NEW.target_id IS NULL THEN



        RAISE EXCEPTION 'target_id cannot be NULL';



    END IF;







    IF NEW.target_type = 'USER' THEN



        -- User ID belongs to System DB, validated by application layer



        RETURN NEW;



    ELSIF NEW.target_type = 'EMPLOYEE' THEN



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





--

-- Name: enqueue_roster_cache_invalidation(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.enqueue_roster_cache_invalidation() RETURNS trigger

    LANGUAGE plpgsql

    SET search_path TO 'public'

    AS $$



    BEGIN



        INSERT INTO public.outbox_jobs (



            id, job_type, payload, status, error_message, created_at, updated_at



        ) VALUES (



            gen_random_uuid(), 'RosterCacheInvalidation', '{}'::jsonb, 'pending', NULL,



            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP



        );



        IF TG_OP = 'DELETE' THEN



            RETURN OLD;



        END IF;



        RETURN NEW;



    END;



    $$;





--

-- Name: fn_cascade_delete_department_assignments(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.fn_cascade_delete_department_assignments() RETURNS trigger

    LANGUAGE plpgsql

    AS $$



BEGIN



    DELETE FROM public.payroll_rule_assignments



        WHERE assignee_type = 'department' AND assignee_id = OLD.id;







DELETE FROM public.payroll_incentive_assignments



        WHERE assignee_type = 'department' AND assignee_id = OLD.id;







DELETE FROM public.leave_policy_assignments



        WHERE assignee_type = 'department' AND assignee_id = OLD.id;







DELETE FROM public.overtime_policy_assignments



        WHERE assignee_type = 'department' AND assignee_id = OLD.id;







DELETE FROM public.attendance_policy_assignments



        WHERE target_type = 'GROUP' AND target_id = OLD.id;







RETURN OLD;







END;







$$;





--

-- Name: fn_cascade_delete_employee_assignments(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.fn_cascade_delete_employee_assignments() RETURNS trigger

    LANGUAGE plpgsql

    AS $$



BEGIN



    DELETE FROM public.payroll_rule_assignments



        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;







DELETE FROM public.payroll_incentive_assignments



        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;







DELETE FROM public.leave_policy_assignments



        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;







DELETE FROM public.overtime_policy_assignments



        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;







DELETE FROM public.attendance_policy_assignments



        WHERE target_type = 'USER' AND target_id = OLD.id;







RETURN OLD;







END;







$$;





--

-- Name: fn_payroll_periods_updated_at(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.fn_payroll_periods_updated_at() RETURNS trigger

    LANGUAGE plpgsql

    AS $$

BEGIN

    NEW.updated_at = CURRENT_TIMESTAMP;

    RETURN NEW;

END;

$$;





--

-- Name: fn_set_updated_at(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.fn_set_updated_at() RETURNS trigger

    LANGUAGE plpgsql

    AS $$



BEGIN



    NEW.updated_at = NOW();







RETURN NEW;







END;







$$;





--

-- Name: fn_validate_assignee_fk(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.fn_validate_assignee_fk() RETURNS trigger

    LANGUAGE plpgsql

    AS $$



    BEGIN



        IF NEW.assignee_type = 'organization' THEN



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





--

-- Name: fn_validate_contract_relationship_link(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.fn_validate_contract_relationship_link() RETURNS trigger

    LANGUAGE plpgsql

    AS $$

BEGIN

    IF NEW.relationship_id IS NULL THEN

        RETURN NEW;

    END IF;



    IF NOT EXISTS (

        SELECT 1

        FROM public.employments segment

        WHERE segment.id = NEW.employment_id

          AND segment.relationship_id = NEW.relationship_id

          AND segment.deleted_at IS NULL

    ) THEN

        RAISE EXCEPTION

            'employee contract relationship_id must match its employment segment'

            USING ERRCODE = '23514';

    END IF;



    RETURN NEW;

END;

$$;





--

-- Name: fn_validate_segment_relationship_change(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.fn_validate_segment_relationship_change() RETURNS trigger

    LANGUAGE plpgsql

    AS $$

BEGIN

    IF NEW.relationship_id IS DISTINCT FROM OLD.relationship_id

       AND EXISTS (

           SELECT 1

           FROM public.employee_contracts contract

           WHERE contract.employment_id = NEW.id

             AND contract.relationship_id IS NOT NULL

             AND contract.relationship_id IS DISTINCT FROM NEW.relationship_id

             AND contract.deleted_at IS NULL

       ) THEN

        RAISE EXCEPTION

            'employment segment relationship_id cannot change while linked contracts exist'

            USING ERRCODE = '23514';

    END IF;



    RETURN NEW;

END;

$$;





--

-- Name: fn_validate_target_fk(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.fn_validate_target_fk() RETURNS trigger

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





--

-- Name: guard_attendance_closure_immutable(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.guard_attendance_closure_immutable() RETURNS trigger

    LANGUAGE plpgsql

    SET search_path TO 'public'

    AS $$

BEGIN

    RAISE EXCEPTION 'attendance closure evidence is immutable';

END;

$$;





--

-- Name: guard_legacy_schedule_mutation(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.guard_legacy_schedule_mutation() RETURNS trigger

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

            JOIN employments em ON em.id = r.employment_id AND em.deleted_at IS NULL

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

              AND r.state IN ('published', 'explicit_off')

              AND r.employment_id = employment_text::uuid

              AND (start_text IS NULL OR r.business_date >= start_text::date)

              AND (end_text IS NULL OR r.business_date <= end_text::date)

        ) INTO blocked;

    ELSIF TG_TABLE_NAME = 'working_pattern_details' THEN

        SELECT EXISTS (

            SELECT 1

            FROM employee_working_patterns a

            JOIN schedule_day_revisions r ON r.employment_id = a.employment_id

            JOIN employments em ON em.id = r.employment_id AND em.deleted_at IS NULL

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

              AND a.pattern_id = COALESCE(new_data->>'pattern_id', old_data->>'pattern_id')::uuid

              AND r.state IN ('published', 'explicit_off')

        ) INTO blocked;

    ELSIF TG_TABLE_NAME IN ('shift_versions', 'shift_templates') THEN

        SELECT EXISTS (

            SELECT 1

            FROM work_segments ws

            JOIN schedule_day_revisions r ON r.id = ws.schedule_revision_id

            JOIN employments em ON em.id = r.employment_id AND em.deleted_at IS NULL

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

              AND ws.template_id = CASE

                    WHEN TG_TABLE_NAME = 'shift_versions' THEN COALESCE((new_data->>'template_id')::uuid, (old_data->>'template_id')::uuid)

                    ELSE COALESCE((new_data->>'id')::uuid, (old_data->>'id')::uuid)

                  END

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

    IF TG_OP = 'DELETE' THEN

        RETURN OLD;

    END IF;

    RETURN NEW;

END;

$$;





--

-- Name: guard_schedule_day_revision(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.guard_schedule_day_revision() RETURNS trigger

    LANGUAGE plpgsql

    SET search_path TO 'public'

    AS $$

BEGIN

    IF TG_OP = 'DELETE' THEN

        IF OLD.state = 'draft' THEN

            RETURN OLD;

        END IF;

        RAISE EXCEPTION 'schedule day revision is immutable';

    END IF;



    IF OLD.state IN ('published', 'explicit_off')

       AND NEW.state = 'superseded'

       AND NEW.id = OLD.id

       AND NEW.employment_id = OLD.employment_id

       AND NEW.employer_id = OLD.employer_id

       AND NEW.business_date = OLD.business_date

       AND NEW.timezone = OLD.timezone

       AND NEW.revision_number = OLD.revision_number

       AND NEW.previous_revision_id IS NOT DISTINCT FROM OLD.previous_revision_id

       AND NEW.publication_batch_id IS NOT DISTINCT FROM OLD.publication_batch_id

       AND NEW.source = OLD.source

       AND NEW.source_snapshot = OLD.source_snapshot

       AND NEW.resolver_version = OLD.resolver_version

       AND NEW.reason = OLD.reason

       AND NEW.published_by IS NOT DISTINCT FROM OLD.published_by

       AND NEW.published_actor_type = OLD.published_actor_type

       AND NEW.published_actor_name IS NOT DISTINCT FROM OLD.published_actor_name

       AND NEW.published_at = OLD.published_at

       AND NEW.created_at = OLD.created_at THEN

        RETURN NEW;

    END IF;



    IF OLD.state = 'draft'

       AND NEW.state IN ('published', 'explicit_off')

       AND NEW.id = OLD.id

       AND NEW.employment_id = OLD.employment_id

       AND NEW.employer_id = OLD.employer_id

       AND NEW.business_date = OLD.business_date

       AND NEW.timezone = OLD.timezone

       AND NEW.revision_number = OLD.revision_number

       AND NEW.previous_revision_id IS NOT DISTINCT FROM OLD.previous_revision_id

       AND NEW.publication_batch_id IS NOT DISTINCT FROM OLD.publication_batch_id

       AND NEW.source = OLD.source

       AND NEW.source_snapshot = OLD.source_snapshot

       AND NEW.resolver_version = OLD.resolver_version

       AND NEW.reason = OLD.reason

       AND NEW.published_by IS NOT DISTINCT FROM OLD.published_by

       AND NEW.published_actor_type = OLD.published_actor_type

       AND NEW.published_actor_name IS NOT DISTINCT FROM OLD.published_actor_name

       AND NEW.published_at IS NOT NULL

       AND NEW.created_at = OLD.created_at THEN

        RETURN NEW;

    END IF;



    RAISE EXCEPTION 'schedule day revision is immutable outside publication transition';

END;

$$;





--

-- Name: guard_schedule_work_segment(); Type: FUNCTION; Schema: public; Owner: -

--



CREATE FUNCTION public.guard_schedule_work_segment() RETURNS trigger

    LANGUAGE plpgsql

    SET search_path TO 'public'

    AS $$

DECLARE

    revision_state character varying(16);

BEGIN

    IF TG_OP IN ('UPDATE', 'DELETE') AND OLD.schedule_revision_id IS NOT NULL THEN

        RAISE EXCEPTION 'official work segment is immutable';

    END IF;



    IF TG_OP = 'DELETE' THEN

        IF EXISTS (

            SELECT 1

            FROM public.schedule_day_revisions

            WHERE employment_id = OLD.employment_id

              AND business_date = OLD.date

              AND state IN ('published', 'explicit_off')

        ) THEN

            RAISE EXCEPTION 'legacy work segment mutation blocked for official schedule date';

        END IF;

        RETURN OLD;

    END IF;



    IF NEW.schedule_revision_id IS NOT NULL THEN

        SELECT state INTO revision_state

        FROM public.schedule_day_revisions

        WHERE id = NEW.schedule_revision_id;



        IF revision_state IS NULL THEN

            RAISE EXCEPTION 'schedule revision does not exist';

        END IF;

        IF revision_state <> 'draft'

           AND COALESCE(current_setting('schedule.lifecycle_publication', true), '') <> 'on' THEN

            RAISE EXCEPTION 'official work segment insert requires draft revision or publication transition';

        END IF;

        IF NOT EXISTS (

            SELECT 1

            FROM public.schedule_day_revisions

            WHERE id = NEW.schedule_revision_id

              AND employment_id = NEW.employment_id

              AND business_date = NEW.date

        ) THEN

            RAISE EXCEPTION 'work segment employment/date does not match schedule revision';

        END IF;

        RETURN NEW;

    END IF;



    IF EXISTS (

        SELECT 1

        FROM public.schedule_day_revisions

        WHERE employment_id = NEW.employment_id

          AND business_date = NEW.date

          AND state IN ('published', 'explicit_off')

    ) THEN

        RAISE EXCEPTION 'legacy work segment mutation blocked for official schedule date';

    END IF;

    RETURN NEW;

END;

$$;





SET default_tablespace = '';



SET default_table_access_method = heap;



--

-- Name: approval_action_log; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.approval_action_log (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    entity_type character varying(50) NOT NULL,

    entity_id uuid NOT NULL,

    step_order integer NOT NULL,

    step_name character varying(100),

    acted_by_user_id uuid NOT NULL,

    action character varying(20) NOT NULL,

    note text,

    acted_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT approval_action_log_action_check CHECK (((action)::text = ANY (ARRAY[('approved'::character varying)::text, ('rejected'::character varying)::text, ('skipped'::character varying)::text, ('returned'::character varying)::text])))

);





--

-- Name: approval_workflow_instances; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.approval_workflow_instances (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    workflow_id uuid,

    entity_type character varying(50) NOT NULL,

    entity_id uuid NOT NULL,

    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,

    current_step_order integer DEFAULT 1 NOT NULL,

    steps_snapshot jsonb DEFAULT '[]'::jsonb NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: approval_workflow_steps; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.approval_workflow_steps (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    workflow_id uuid NOT NULL,

    step_order integer NOT NULL,

    step_name character varying(100) NOT NULL,

    approver_type character varying(30) NOT NULL,

    approver_config jsonb DEFAULT '{}'::jsonb NOT NULL,

    action_required character varying(20) DEFAULT 'approve'::character varying NOT NULL,

    can_skip boolean DEFAULT false NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT approval_workflow_steps_action_required_check CHECK (((action_required)::text = ANY (ARRAY[('approve'::character varying)::text, ('review'::character varying)::text, ('acknowledge'::character varying)::text]))),

    CONSTRAINT approval_workflow_steps_approver_type_check CHECK (((approver_type)::text = ANY (ARRAY[('department_manager'::character varying)::text, ('role'::character varying)::text, ('permission'::character varying)::text, ('specific_user'::character varying)::text]))),

    CONSTRAINT approval_workflow_steps_step_order_check CHECK ((step_order >= 1))

);





--

-- Name: approval_workflow_tasks; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.approval_workflow_tasks (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    instance_id uuid NOT NULL,

    step_order integer NOT NULL,

    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,

    assignee_type character varying(30) NOT NULL,

    assignee_target_id uuid,

    assignee_target_name character varying(100),

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    completed_at timestamp with time zone,

    completed_by uuid

);





--

-- Name: approval_workflows; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.approval_workflows (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    entity_type character varying(50) NOT NULL,

    name character varying(255) NOT NULL,

    description text,

    total_steps integer NOT NULL,

    is_active boolean DEFAULT true NOT NULL,

    created_by uuid,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT approval_workflows_total_steps_check CHECK ((total_steps >= 1))

);





--

-- Name: audit_logs; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

)

PARTITION BY RANGE (created_at);





--

-- Name: audit_logs_default; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_default (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m01; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m01 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m02; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m02 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m03; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m03 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m04; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m04 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m05; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m05 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m06; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m06 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m07; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m07 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m08; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m08 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m09; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m09 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m10; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m10 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m11; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m11 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: audit_logs_y2026m12; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.audit_logs_y2026m12 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,

    actor_id uuid,

    target_user_id uuid,

    target_resource_id uuid,

    action character varying(255) CONSTRAINT audit_logs_action_not_null NOT NULL,

    details jsonb,

    ip_address text,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT audit_logs_created_at_not_null NOT NULL,

    actor_type character varying(50),

    actor_name character varying(255),

    actor_email character varying(255)

);





--

-- Name: authorized_agents; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.authorized_agents (

    id character varying(50) NOT NULL,

    name character varying(100) NOT NULL,

    token character varying(255) NOT NULL,

    is_trusted boolean DEFAULT false,

    last_ip character varying(45),

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    is_active boolean DEFAULT true NOT NULL,

    last_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    max_concurrent_workers integer DEFAULT 2 NOT NULL,

    success_cooldown_secs integer DEFAULT 300 NOT NULL,

    retention_days integer DEFAULT 365 NOT NULL

);





--

-- Name: behalf_assignments; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.behalf_assignments (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT ot_claim_behalf_assignments_id_not_null NOT NULL,

    target_employee_id uuid,

    target_department_id uuid,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT ot_claim_behalf_assignments_created_at_not_null NOT NULL,

    updated_at timestamp with time zone DEFAULT now() CONSTRAINT ot_claim_behalf_assignments_updated_at_not_null NOT NULL,

    assignment_type character varying(50) NOT NULL,

    assignee_user_id uuid NOT NULL,

    employer_id uuid,

    CONSTRAINT check_target CHECK ((((target_employee_id IS NOT NULL) AND (target_department_id IS NULL)) OR ((target_employee_id IS NULL) AND (target_department_id IS NOT NULL))))

);





--

-- Name: compliance_rules; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.compliance_rules (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT organization_compliance_rules_id_not_null NOT NULL,

    min_rest_period_hours integer DEFAULT 12 CONSTRAINT organization_compliance_rules_min_rest_period_hours_not_null NOT NULL,

    max_weekly_hours integer DEFAULT 48 CONSTRAINT organization_compliance_rules_max_weekly_hours_not_null NOT NULL,

    max_monthly_ot_hours integer DEFAULT 40 CONSTRAINT organization_compliance_rules_max_monthly_ot_hours_not_null NOT NULL,

    allow_manager_override boolean DEFAULT true CONSTRAINT organization_compliance_rules_allow_manager_override_not_null NOT NULL,

    is_enabled boolean DEFAULT true CONSTRAINT organization_compliance_rules_is_enabled_not_null NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT organization_compliance_rules_created_at_not_null NOT NULL,

    updated_at timestamp with time zone DEFAULT now() CONSTRAINT organization_compliance_rules_updated_at_not_null NOT NULL

);





--

-- Name: condition_type_definitions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.condition_type_definitions (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    code character varying(50) NOT NULL,

    name_i18n jsonb DEFAULT '{}'::jsonb NOT NULL,

    category character varying(50) DEFAULT 'general'::character varying NOT NULL,

    default_minute_allowance integer DEFAULT 0 NOT NULL,

    default_is_paid boolean DEFAULT false NOT NULL,

    affects_rules jsonb DEFAULT '{}'::jsonb NOT NULL,

    icon character varying(50),

    color character varying(20),

    is_system boolean DEFAULT false NOT NULL,

    sort_order integer DEFAULT 0 NOT NULL,

    is_active boolean DEFAULT true NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT condition_type_definitions_valid_code CHECK (((code)::text ~ '^[A-Z][A-Z0-9_]*$'::text))

);





--

-- Name: config_entries; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.config_entries (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT organization_config_entries_id_not_null NOT NULL,

    scope_type character varying(20) DEFAULT 'organization'::character varying CONSTRAINT organization_config_entries_scope_type_not_null NOT NULL,

    scope_id uuid,

    config_key character varying(100) CONSTRAINT organization_config_entries_config_key_not_null NOT NULL,

    value_int integer,

    value_decimal numeric(12,4),

    value_text text,

    value_bool boolean,

    value_json jsonb,

    effective_from date CONSTRAINT organization_config_entries_effective_from_not_null NOT NULL,

    effective_to date,

    created_by uuid,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT organization_config_entries_created_at_not_null NOT NULL,

    updated_at timestamp with time zone DEFAULT now() CONSTRAINT organization_config_entries_updated_at_not_null NOT NULL,

    CONSTRAINT config_entries_effective_date_range_check CHECK (((effective_to IS NULL) OR (effective_to >= effective_from))),

    CONSTRAINT config_entries_scope_type_check CHECK (((scope_type)::text = ANY (ARRAY[('organization'::character varying)::text, ('employer'::character varying)::text, ('department'::character varying)::text, ('employee'::character varying)::text, ('employment'::character varying)::text]))),

    CONSTRAINT organization_config_entries_check CHECK ((((((((value_int IS NOT NULL))::integer + ((value_decimal IS NOT NULL))::integer) + ((value_text IS NOT NULL))::integer) + ((value_bool IS NOT NULL))::integer) + ((value_json IS NOT NULL))::integer) = 1))

);





--

-- Name: custom_field_definitions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.custom_field_definitions (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    entity_type character varying(50) NOT NULL,

    field_key character varying(100) NOT NULL,

    field_label text NOT NULL,

    field_type character varying(20) NOT NULL,

    options jsonb DEFAULT '[]'::jsonb,

    default_value text,

    is_required boolean DEFAULT false NOT NULL,

    is_visible_to_employee boolean DEFAULT false NOT NULL,

    is_filterable boolean DEFAULT false NOT NULL,

    is_exportable boolean DEFAULT true NOT NULL,

    display_order integer DEFAULT 0 NOT NULL,

    section_name character varying(100),

    validation_rules jsonb,

    deleted_at timestamp with time zone,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: custom_field_values; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.custom_field_values (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    definition_id uuid NOT NULL,

    entity_id uuid NOT NULL,

    value_text text,

    value_number numeric,

    value_date date,

    value_bool boolean,

    value_json jsonb,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: dashboard_preferences; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.dashboard_preferences (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    user_id uuid NOT NULL,

    active_view text DEFAULT 'overview'::text NOT NULL,

    visible_widgets jsonb DEFAULT '{}'::jsonb NOT NULL,

    widget_order jsonb DEFAULT '[]'::jsonb NOT NULL,

    collapsed_sections jsonb DEFAULT '{}'::jsonb NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    window_layout jsonb DEFAULT '{}'::jsonb NOT NULL

);





--

-- Name: document_folders; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.document_folders (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name character varying(255) NOT NULL,

    parent_folder_id uuid,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: feature_flags; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.feature_flags (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    flag_key character varying(128) NOT NULL,

    enabled boolean DEFAULT false NOT NULL,

    description text,

    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: file_acknowledgements; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.file_acknowledgements (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    document_id uuid NOT NULL,

    employee_id uuid NOT NULL,

    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,

    acknowledged_at timestamp with time zone,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: files; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.files (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT organization_files_id_not_null NOT NULL,

    file_name character varying(255) CONSTRAINT organization_files_file_name_not_null NOT NULL,

    storage_key character varying(500) CONSTRAINT organization_files_storage_key_not_null NOT NULL,

    file_size bigint CONSTRAINT organization_files_file_size_not_null NOT NULL,

    mime_type character varying(100) CONSTRAINT organization_files_mime_type_not_null NOT NULL,

    content_hash character varying(64),

    category character varying(50) CONSTRAINT organization_files_category_not_null NOT NULL,

    folder_id uuid,

    owner_type character varying(50) CONSTRAINT organization_files_owner_type_not_null NOT NULL,

    owner_id uuid CONSTRAINT organization_files_owner_id_not_null NOT NULL,

    uploaded_by uuid,

    is_active boolean DEFAULT true CONSTRAINT organization_files_is_active_not_null NOT NULL,

    expires_at timestamp with time zone,

    scan_status character varying(20) DEFAULT 'pending'::character varying CONSTRAINT organization_files_scan_status_not_null NOT NULL,

    scanned_at timestamp with time zone,

    version integer DEFAULT 1 CONSTRAINT organization_files_version_not_null NOT NULL,

    parent_file_id uuid,

    is_latest boolean DEFAULT true CONSTRAINT organization_files_is_latest_not_null NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT organization_files_created_at_not_null NOT NULL,

    updated_at timestamp with time zone DEFAULT now() CONSTRAINT organization_files_updated_at_not_null NOT NULL

);





--

-- Name: infra_job_events; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.infra_job_events (

    id uuid NOT NULL,

    job_id uuid NOT NULL,

    event_type character varying(40) NOT NULL,

    from_state character varying(30),

    to_state character varying(30) NOT NULL,

    attempt_count integer DEFAULT 0 NOT NULL,

    lease_token uuid,

    worker_id character varying(100),

    request_id uuid,

    trace_id character varying(128),

    reason_code character varying(100),

    reason_summary text,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT infra_job_events_attempt_count_check CHECK ((attempt_count >= 0))

);





--

-- Name: infra_job_records; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.infra_job_records (

    id uuid NOT NULL,

    parent_job_id uuid,

    task_type character varying(100) NOT NULL,

    priority character varying(20) NOT NULL,

    state character varying(30) NOT NULL,

    attempt_count integer DEFAULT 0 NOT NULL,

    lease_owner character varying(100),

    lease_token uuid,

    lease_expires_at timestamp with time zone,

    next_attempt_at timestamp with time zone,

    request_id uuid,

    trace_id character varying(128),

    payload_hash character varying(128),

    error_code character varying(100),

    error_summary text,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    queued_at timestamp with time zone,

    started_at timestamp with time zone,

    finished_at timestamp with time zone,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT infra_job_records_attempt_count_check CHECK ((attempt_count >= 0)),

    CONSTRAINT infra_job_records_state_check CHECK (((state)::text = ANY (ARRAY[('dispatching'::character varying)::text, ('queued'::character varying)::text, ('processing'::character varying)::text, ('completed'::character varying)::text, ('failed'::character varying)::text, ('retryable'::character varying)::text, ('dead_letter'::character varying)::text, ('lost'::character varying)::text])))

);





--

-- Name: lifecycle_checklist_items; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.lifecycle_checklist_items (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    checklist_id uuid NOT NULL,

    template_item_id uuid,

    title text NOT NULL,

    description text,

    category character varying(50) NOT NULL,

    assignee_user_id uuid,

    is_required boolean DEFAULT true NOT NULL,

    due_date date,

    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,

    completed_at timestamp with time zone,

    completed_by uuid,

    notes text,

    display_order integer DEFAULT 0 NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: lifecycle_checklist_template_items; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.lifecycle_checklist_template_items (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    template_id uuid NOT NULL,

    title text NOT NULL,

    description text,

    category character varying(50) NOT NULL,

    assignee_type character varying(20) NOT NULL,

    assignee_user_id uuid,

    is_required boolean DEFAULT true NOT NULL,

    due_days_offset integer DEFAULT 0 NOT NULL,

    display_order integer DEFAULT 0 NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: lifecycle_checklist_templates; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.lifecycle_checklist_templates (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name text NOT NULL,

    description text,

    checklist_type character varying(20) NOT NULL,

    is_default boolean DEFAULT false NOT NULL,

    is_active boolean DEFAULT true NOT NULL,

    deleted_at timestamp with time zone,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: lifecycle_checklists; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.lifecycle_checklists (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employment_id uuid NOT NULL,

    template_id uuid,

    checklist_type character varying(20) NOT NULL,

    status character varying(20) DEFAULT 'in_progress'::character varying NOT NULL,

    progress_percent integer DEFAULT 0 NOT NULL,

    started_at timestamp with time zone DEFAULT now() NOT NULL,

    completed_at timestamp with time zone,

    created_by uuid,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: monitor_alert_events; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.monitor_alert_events (

    id uuid NOT NULL,

    alert_id uuid NOT NULL,

    event_type character varying(40) NOT NULL,

    severity character varying(20) NOT NULL,

    summary text NOT NULL,

    occurred_at timestamp with time zone NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT monitor_alert_events_severity_check CHECK (((severity)::text = ANY (ARRAY[('info'::character varying)::text, ('warning'::character varying)::text, ('critical'::character varying)::text])))

);





--

-- Name: monitor_alerts; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.monitor_alerts (

    id uuid NOT NULL,

    fingerprint character varying(128) NOT NULL,

    alert_type character varying(100) NOT NULL,

    severity character varying(20) NOT NULL,

    status character varying(20) DEFAULT 'active'::character varying NOT NULL,

    title character varying(255) NOT NULL,

    summary text NOT NULL,

    source character varying(100) NOT NULL,

    first_seen_at timestamp with time zone NOT NULL,

    last_seen_at timestamp with time zone NOT NULL,

    occurrence_count integer DEFAULT 1 NOT NULL,

    resolved_at timestamp with time zone,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT monitor_alerts_occurrence_count_check CHECK ((occurrence_count > 0)),

    CONSTRAINT monitor_alerts_severity_check CHECK (((severity)::text = ANY (ARRAY[('info'::character varying)::text, ('warning'::character varying)::text, ('critical'::character varying)::text]))),

    CONSTRAINT monitor_alerts_status_check CHECK (((status)::text = ANY (ARRAY[('active'::character varying)::text, ('resolved'::character varying)::text])))

);





--

-- Name: notifications; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.notifications (

    id uuid NOT NULL,

    user_id uuid NOT NULL,

    type character varying(50) NOT NULL,

    title character varying(255) NOT NULL,

    content text NOT NULL,

    is_read boolean DEFAULT false NOT NULL,

    link character varying(255),

    created_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: organization_settings; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.organization_settings (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT organizations_id_not_null NOT NULL,

    name character varying(100) CONSTRAINT organizations_name_not_null NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    date_format character varying(20) DEFAULT 'DD/MM/YYYY'::character varying CONSTRAINT organizations_date_format_not_null NOT NULL,

    time_format character varying(10) DEFAULT '24h'::character varying CONSTRAINT organizations_time_format_not_null NOT NULL,

    timezone character varying(50) DEFAULT 'Asia/Ho_Chi_Minh'::character varying CONSTRAINT organizations_timezone_not_null NOT NULL,

    theme_primary_color text DEFAULT '#7c3aed'::text,

    default_language character varying(10) DEFAULT 'vi'::character varying CONSTRAINT organizations_default_language_not_null NOT NULL,

    logo_url text,

    tagline text,

    custom_db_url_encrypted text,

    CONSTRAINT chk_organization_name CHECK ((length((name)::text) > 0))

);





--

-- Name: outbox_jobs; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.outbox_jobs (

    id uuid NOT NULL,

    job_type character varying(50) NOT NULL,

    payload jsonb NOT NULL,

    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,

    error_message text,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    dedupe_key text,

    lease_token uuid,

    locked_until timestamp with time zone,

    attempts integer DEFAULT 0 NOT NULL,

    available_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    last_error text,

    dead_letter_at timestamp with time zone,

    CONSTRAINT outbox_jobs_attempts_check CHECK ((attempts >= 0))

);





--

-- Name: policy_templates; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.policy_templates (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name text NOT NULL,

    description text,

    country_code character varying(3),

    category text DEFAULT 'full'::text NOT NULL,

    template_data jsonb DEFAULT '{}'::jsonb NOT NULL,

    version integer DEFAULT 1 NOT NULL,

    is_active boolean DEFAULT true NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL

);





--

-- Name: role_permissions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.role_permissions (

    role_id uuid CONSTRAINT organization_role_permissions_role_id_not_null NOT NULL,

    permission_name text CONSTRAINT organization_role_permissions_permission_name_not_null NOT NULL

);





--

-- Name: roles; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.roles (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT organization_roles_id_not_null NOT NULL,

    name text CONSTRAINT organization_roles_name_not_null NOT NULL,

    description text,

    color text DEFAULT 'slate'::text,

    is_system boolean DEFAULT false,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    scope_type character varying(16) DEFAULT 'scoped'::character varying CONSTRAINT organization_roles_scope_type_not_null NOT NULL,

    CONSTRAINT organization_roles_scope_type_check CHECK (((scope_type)::text = ANY (ARRAY[('global'::character varying)::text, ('scoped'::character varying)::text])))

);





--

-- Name: schema_migrations; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.schema_migrations (

    version text NOT NULL,

    applied_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: sessions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.sessions (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    user_id uuid NOT NULL,

    token text NOT NULL,

    expires_at timestamp with time zone NOT NULL,

    created_at timestamp with time zone DEFAULT now(),

    revoked_at timestamp with time zone

);





--

-- Name: signature_requests; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.signature_requests (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    document_id uuid NOT NULL,

    signer_id uuid NOT NULL,

    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,

    signed_at timestamp with time zone,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: storage_configs; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.storage_configs (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    endpoint_url character varying(255) NOT NULL,

    region character varying(50) NOT NULL,

    bucket_name character varying(100) NOT NULL,

    path_prefix character varying(255),

    access_key_enc character varying(500) NOT NULL,

    secret_key_enc character varying(500) NOT NULL,

    encryption_key_version character varying(50) NOT NULL,

    use_path_style boolean DEFAULT true NOT NULL,

    presigned_ttl_secs integer DEFAULT 900 NOT NULL,

    max_file_size bigint DEFAULT 10485760 NOT NULL,

    is_active boolean DEFAULT true NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: system_audit_logs; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.system_audit_logs (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    actor_id uuid,

    action character varying(100) NOT NULL,

    details jsonb,

    ip_address inet,

    user_agent text,

    created_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: system_files; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.system_files (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    file_name character varying(255) NOT NULL,

    storage_key character varying(500) NOT NULL,

    file_size bigint NOT NULL,

    mime_type character varying(100) NOT NULL,

    content_hash character varying(64),

    category character varying(50) NOT NULL,

    owner_type character varying(50) NOT NULL,

    owner_id uuid NOT NULL,

    uploaded_by uuid,

    is_active boolean DEFAULT true NOT NULL,

    scan_status character varying(20) DEFAULT 'pending'::character varying NOT NULL,

    version integer DEFAULT 1 NOT NULL,

    parent_file_id uuid,

    is_latest boolean DEFAULT true NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: system_settings; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.system_settings (

    key character varying(100) NOT NULL,

    value_json jsonb NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: user_extra_permissions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.user_extra_permissions (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT organization_member_extra_permissions_id_not_null NOT NULL,

    user_id uuid CONSTRAINT organization_member_extra_permissions_user_id_not_null NOT NULL,

    permission text CONSTRAINT organization_member_extra_permissions_permission_not_null NOT NULL,

    assigned_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    expires_at timestamp with time zone CONSTRAINT organization_member_extra_permissions_expires_at_not_null NOT NULL,

    assigned_by uuid

);





--

-- Name: user_extra_roles; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.user_extra_roles (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT member_extra_roles_id_not_null NOT NULL,

    user_id uuid CONSTRAINT member_extra_roles_user_id_not_null NOT NULL,

    assigned_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    expires_at timestamp with time zone CONSTRAINT member_extra_roles_expires_at_not_null NOT NULL,

    role_id uuid CONSTRAINT member_extra_roles_role_id_not_null NOT NULL

);





--

-- Name: user_roles; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.user_roles (

    user_id uuid CONSTRAINT organization_members_user_id_not_null NOT NULL,

    role_id uuid CONSTRAINT organization_members_role_id_not_null NOT NULL

);





--

-- Name: users; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.users (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    email character varying(255) NOT NULL,

    password_hash character varying(255) NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    full_name character varying(100),

    avatar_url text,

    bio text,

    preferred_language character varying(10),

    avatar_file_id uuid

);





--

-- Name: work_locations; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.work_locations (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name character varying(255) NOT NULL,

    address text,

    timezone character varying(50) DEFAULT 'UTC'::character varying NOT NULL,

    is_headquarters boolean DEFAULT false NOT NULL,

    status character varying(20) DEFAULT 'active'::character varying NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: working_conditions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.working_conditions (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    type character varying(50) NOT NULL,

    minute_allowance integer NOT NULL,

    is_paid boolean DEFAULT false NOT NULL,

    start_date date NOT NULL,

    end_date date,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    employment_id uuid NOT NULL,

    condition_type_id uuid

);





--

-- Name: audit_logs_default; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_default DEFAULT;





--

-- Name: audit_logs_y2026m01; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m01 FOR VALUES FROM ('2026-01-01 07:00:00+07') TO ('2026-02-01 07:00:00+07');





--

-- Name: audit_logs_y2026m02; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m02 FOR VALUES FROM ('2026-02-01 07:00:00+07') TO ('2026-03-01 07:00:00+07');





--

-- Name: audit_logs_y2026m03; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m03 FOR VALUES FROM ('2026-03-01 07:00:00+07') TO ('2026-04-01 07:00:00+07');





--

-- Name: audit_logs_y2026m04; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m04 FOR VALUES FROM ('2026-04-01 07:00:00+07') TO ('2026-05-01 07:00:00+07');





--

-- Name: audit_logs_y2026m05; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m05 FOR VALUES FROM ('2026-05-01 07:00:00+07') TO ('2026-06-01 07:00:00+07');





--

-- Name: audit_logs_y2026m06; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m06 FOR VALUES FROM ('2026-06-01 07:00:00+07') TO ('2026-07-01 07:00:00+07');





--

-- Name: audit_logs_y2026m07; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m07 FOR VALUES FROM ('2026-07-01 07:00:00+07') TO ('2026-08-01 07:00:00+07');





--

-- Name: audit_logs_y2026m08; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m08 FOR VALUES FROM ('2026-08-01 07:00:00+07') TO ('2026-09-01 07:00:00+07');





--

-- Name: audit_logs_y2026m09; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m09 FOR VALUES FROM ('2026-09-01 07:00:00+07') TO ('2026-10-01 07:00:00+07');





--

-- Name: audit_logs_y2026m10; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m10 FOR VALUES FROM ('2026-10-01 07:00:00+07') TO ('2026-11-01 07:00:00+07');





--

-- Name: audit_logs_y2026m11; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m11 FOR VALUES FROM ('2026-11-01 07:00:00+07') TO ('2026-12-01 07:00:00+07');





--

-- Name: audit_logs_y2026m12; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m12 FOR VALUES FROM ('2026-12-01 07:00:00+07') TO ('2027-01-01 07:00:00+07');





--

-- Name: approval_action_log approval_action_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_action_log

    ADD CONSTRAINT approval_action_log_pkey PRIMARY KEY (id);





--

-- Name: approval_workflow_instances approval_workflow_instances_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_workflow_instances

    ADD CONSTRAINT approval_workflow_instances_pkey PRIMARY KEY (id);





--

-- Name: approval_workflow_steps approval_workflow_steps_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_workflow_steps

    ADD CONSTRAINT approval_workflow_steps_pkey PRIMARY KEY (id);





--

-- Name: approval_workflow_steps approval_workflow_steps_workflow_id_step_order_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_workflow_steps

    ADD CONSTRAINT approval_workflow_steps_workflow_id_step_order_key UNIQUE (workflow_id, step_order);





--

-- Name: approval_workflow_tasks approval_workflow_tasks_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_workflow_tasks

    ADD CONSTRAINT approval_workflow_tasks_pkey PRIMARY KEY (id);





--

-- Name: approval_workflows approval_workflows_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_workflows

    ADD CONSTRAINT approval_workflows_pkey PRIMARY KEY (id);





--

-- Name: audit_logs audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs

    ADD CONSTRAINT audit_logs_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_default audit_logs_default_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_default

    ADD CONSTRAINT audit_logs_default_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m01 audit_logs_y2026m01_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m01

    ADD CONSTRAINT audit_logs_y2026m01_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m02 audit_logs_y2026m02_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m02

    ADD CONSTRAINT audit_logs_y2026m02_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m03 audit_logs_y2026m03_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m03

    ADD CONSTRAINT audit_logs_y2026m03_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m04 audit_logs_y2026m04_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m04

    ADD CONSTRAINT audit_logs_y2026m04_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m05 audit_logs_y2026m05_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m05

    ADD CONSTRAINT audit_logs_y2026m05_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m06 audit_logs_y2026m06_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m06

    ADD CONSTRAINT audit_logs_y2026m06_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m07 audit_logs_y2026m07_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m07

    ADD CONSTRAINT audit_logs_y2026m07_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m08 audit_logs_y2026m08_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m08

    ADD CONSTRAINT audit_logs_y2026m08_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m09 audit_logs_y2026m09_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m09

    ADD CONSTRAINT audit_logs_y2026m09_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m10 audit_logs_y2026m10_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m10

    ADD CONSTRAINT audit_logs_y2026m10_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m11 audit_logs_y2026m11_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m11

    ADD CONSTRAINT audit_logs_y2026m11_pkey PRIMARY KEY (id, created_at);





--

-- Name: audit_logs_y2026m12 audit_logs_y2026m12_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.audit_logs_y2026m12

    ADD CONSTRAINT audit_logs_y2026m12_pkey PRIMARY KEY (id, created_at);





--

-- Name: authorized_agents authorized_agents_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.authorized_agents

    ADD CONSTRAINT authorized_agents_pkey PRIMARY KEY (id);





--

-- Name: behalf_assignments ot_claim_behalf_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.behalf_assignments

    ADD CONSTRAINT ot_claim_behalf_assignments_pkey PRIMARY KEY (id);





--

-- Name: behalf_assignments uniq_behalf_assignment; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.behalf_assignments

    ADD CONSTRAINT uniq_behalf_assignment UNIQUE (assignee_user_id, target_employee_id, target_department_id, assignment_type);





--

-- Name: compliance_rules organization_compliance_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.compliance_rules

    ADD CONSTRAINT organization_compliance_rules_pkey PRIMARY KEY (id);





--

-- Name: condition_type_definitions condition_type_definitions_organization_code_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.condition_type_definitions

    ADD CONSTRAINT condition_type_definitions_organization_code_key UNIQUE (code);





--

-- Name: condition_type_definitions condition_type_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.condition_type_definitions

    ADD CONSTRAINT condition_type_definitions_pkey PRIMARY KEY (id);





--

-- Name: config_entries config_entries_schedule_lifecycle_effective_no_overlap; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.config_entries

    ADD CONSTRAINT config_entries_schedule_lifecycle_effective_no_overlap EXCLUDE USING gist (scope_type WITH =, scope_id WITH =, config_key WITH =, daterange(effective_from, COALESCE((effective_to + 1), 'infinity'::date), '[)'::text) WITH &&) WHERE ((((scope_type)::text = 'employer'::text) AND ((config_key)::text = 'schedule.lifecycle.enabled'::text)));





--

-- Name: config_entries organization_config_entries_organization_scope_type_scope_id_c_; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.config_entries

    ADD CONSTRAINT organization_config_entries_organization_scope_type_scope_id_c_ UNIQUE NULLS NOT DISTINCT (scope_type, scope_id, config_key, effective_from);





--

-- Name: config_entries organization_config_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.config_entries

    ADD CONSTRAINT organization_config_entries_pkey PRIMARY KEY (id);





--

-- Name: custom_field_definitions custom_field_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.custom_field_definitions

    ADD CONSTRAINT custom_field_definitions_pkey PRIMARY KEY (id);





--

-- Name: custom_field_definitions uq_organization_entity_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.custom_field_definitions

    ADD CONSTRAINT uq_organization_entity_key UNIQUE (entity_type, field_key);





--

-- Name: custom_field_values custom_field_values_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.custom_field_values

    ADD CONSTRAINT custom_field_values_pkey PRIMARY KEY (id);





--

-- Name: custom_field_values uq_definition_entity; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.custom_field_values

    ADD CONSTRAINT uq_definition_entity UNIQUE (definition_id, entity_id);





--

-- Name: dashboard_preferences dashboard_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.dashboard_preferences

    ADD CONSTRAINT dashboard_preferences_pkey PRIMARY KEY (id);





--

-- Name: dashboard_preferences dashboard_preferences_user_id_organization_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.dashboard_preferences

    ADD CONSTRAINT dashboard_preferences_user_id_organization_key UNIQUE (user_id);





--

-- Name: document_folders document_folders_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.document_folders

    ADD CONSTRAINT document_folders_pkey PRIMARY KEY (id);





--

-- Name: feature_flags feature_flags_organization_flag_key_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.feature_flags

    ADD CONSTRAINT feature_flags_organization_flag_key_key UNIQUE (flag_key);





--

-- Name: feature_flags feature_flags_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.feature_flags

    ADD CONSTRAINT feature_flags_pkey PRIMARY KEY (id);





--

-- Name: file_acknowledgements file_acknowledgements_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.file_acknowledgements

    ADD CONSTRAINT file_acknowledgements_pkey PRIMARY KEY (id);





--

-- Name: files organization_files_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.files

    ADD CONSTRAINT organization_files_pkey PRIMARY KEY (id);





--

-- Name: files organization_files_storage_key_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.files

    ADD CONSTRAINT organization_files_storage_key_key UNIQUE (storage_key);





--

-- Name: infra_job_events infra_job_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.infra_job_events

    ADD CONSTRAINT infra_job_events_pkey PRIMARY KEY (id);





--

-- Name: infra_job_records infra_job_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.infra_job_records

    ADD CONSTRAINT infra_job_records_pkey PRIMARY KEY (id);





--

-- Name: lifecycle_checklist_items lifecycle_checklist_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.lifecycle_checklist_items

    ADD CONSTRAINT lifecycle_checklist_items_pkey PRIMARY KEY (id);





--

-- Name: lifecycle_checklist_template_items lifecycle_checklist_template_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.lifecycle_checklist_template_items

    ADD CONSTRAINT lifecycle_checklist_template_items_pkey PRIMARY KEY (id);





--

-- Name: lifecycle_checklist_templates lifecycle_checklist_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.lifecycle_checklist_templates

    ADD CONSTRAINT lifecycle_checklist_templates_pkey PRIMARY KEY (id);





--

-- Name: lifecycle_checklists lifecycle_checklists_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.lifecycle_checklists

    ADD CONSTRAINT lifecycle_checklists_pkey PRIMARY KEY (id);





--

-- Name: monitor_alert_events monitor_alert_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.monitor_alert_events

    ADD CONSTRAINT monitor_alert_events_pkey PRIMARY KEY (id);





--

-- Name: monitor_alerts monitor_alerts_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.monitor_alerts

    ADD CONSTRAINT monitor_alerts_pkey PRIMARY KEY (id);





--

-- Name: notifications notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.notifications

    ADD CONSTRAINT notifications_pkey PRIMARY KEY (id);





--

-- Name: organization_settings organizations_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.organization_settings

    ADD CONSTRAINT organizations_pkey PRIMARY KEY (id);





--

-- Name: outbox_jobs outbox_jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.outbox_jobs

    ADD CONSTRAINT outbox_jobs_pkey PRIMARY KEY (id);





--

-- Name: policy_templates policy_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.policy_templates

    ADD CONSTRAINT policy_templates_pkey PRIMARY KEY (id);





--

-- Name: role_permissions organization_role_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.role_permissions

    ADD CONSTRAINT organization_role_permissions_pkey PRIMARY KEY (role_id, permission_name);





--

-- Name: roles organization_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.roles

    ADD CONSTRAINT organization_roles_pkey PRIMARY KEY (id);





--

-- Name: schema_migrations schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schema_migrations

    ADD CONSTRAINT schema_migrations_pkey PRIMARY KEY (version);





--

-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.sessions

    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);





--

-- Name: sessions sessions_token_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.sessions

    ADD CONSTRAINT sessions_token_key UNIQUE (token);





--

-- Name: signature_requests signature_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.signature_requests

    ADD CONSTRAINT signature_requests_pkey PRIMARY KEY (id);





--

-- Name: storage_configs storage_configs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.storage_configs

    ADD CONSTRAINT storage_configs_pkey PRIMARY KEY (id);





--

-- Name: system_audit_logs system_audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.system_audit_logs

    ADD CONSTRAINT system_audit_logs_pkey PRIMARY KEY (id);





--

-- Name: system_files system_files_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.system_files

    ADD CONSTRAINT system_files_pkey PRIMARY KEY (id);





--

-- Name: system_files system_files_storage_key_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.system_files

    ADD CONSTRAINT system_files_storage_key_key UNIQUE (storage_key);





--

-- Name: system_settings system_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.system_settings

    ADD CONSTRAINT system_settings_pkey PRIMARY KEY (key);





--

-- Name: user_extra_permissions organization_member_extra_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.user_extra_permissions

    ADD CONSTRAINT organization_member_extra_permissions_pkey PRIMARY KEY (id);





--

-- Name: user_extra_roles member_extra_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.user_extra_roles

    ADD CONSTRAINT member_extra_roles_pkey PRIMARY KEY (id);





--

-- Name: user_roles organization_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.user_roles

    ADD CONSTRAINT organization_members_pkey PRIMARY KEY (user_id);





--

-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.users

    ADD CONSTRAINT users_email_key UNIQUE (email);





--

-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.users

    ADD CONSTRAINT users_pkey PRIMARY KEY (id);





--

-- Name: work_locations work_locations_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_locations

    ADD CONSTRAINT work_locations_pkey PRIMARY KEY (id);





--

-- Name: working_conditions working_conditions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.working_conditions

    ADD CONSTRAINT working_conditions_pkey PRIMARY KEY (id);





--

-- Name: audit_logs_default_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_default_action_idx ON public.audit_logs_default USING btree (action);





--

-- Name: audit_logs_default_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_default_created_at_idx ON public.audit_logs_default USING btree (created_at);





--

-- Name: audit_logs_default_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_default_target_user_id_idx ON public.audit_logs_default USING btree (target_user_id);





--

-- Name: audit_logs_y2026m01_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m01_action_idx ON public.audit_logs_y2026m01 USING btree (action);





--

-- Name: audit_logs_y2026m01_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m01_created_at_idx ON public.audit_logs_y2026m01 USING btree (created_at);





--

-- Name: audit_logs_y2026m01_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m01_target_user_id_idx ON public.audit_logs_y2026m01 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m02_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m02_action_idx ON public.audit_logs_y2026m02 USING btree (action);





--

-- Name: audit_logs_y2026m02_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m02_created_at_idx ON public.audit_logs_y2026m02 USING btree (created_at);





--

-- Name: audit_logs_y2026m02_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m02_target_user_id_idx ON public.audit_logs_y2026m02 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m03_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m03_action_idx ON public.audit_logs_y2026m03 USING btree (action);





--

-- Name: audit_logs_y2026m03_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m03_created_at_idx ON public.audit_logs_y2026m03 USING btree (created_at);





--

-- Name: audit_logs_y2026m03_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m03_target_user_id_idx ON public.audit_logs_y2026m03 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m04_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m04_action_idx ON public.audit_logs_y2026m04 USING btree (action);





--

-- Name: audit_logs_y2026m04_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m04_created_at_idx ON public.audit_logs_y2026m04 USING btree (created_at);





--

-- Name: audit_logs_y2026m04_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m04_target_user_id_idx ON public.audit_logs_y2026m04 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m05_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m05_action_idx ON public.audit_logs_y2026m05 USING btree (action);





--

-- Name: audit_logs_y2026m05_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m05_created_at_idx ON public.audit_logs_y2026m05 USING btree (created_at);





--

-- Name: audit_logs_y2026m05_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m05_target_user_id_idx ON public.audit_logs_y2026m05 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m06_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m06_action_idx ON public.audit_logs_y2026m06 USING btree (action);





--

-- Name: audit_logs_y2026m06_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m06_created_at_idx ON public.audit_logs_y2026m06 USING btree (created_at);





--

-- Name: audit_logs_y2026m06_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m06_target_user_id_idx ON public.audit_logs_y2026m06 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m07_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m07_action_idx ON public.audit_logs_y2026m07 USING btree (action);





--

-- Name: audit_logs_y2026m07_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m07_created_at_idx ON public.audit_logs_y2026m07 USING btree (created_at);





--

-- Name: audit_logs_y2026m07_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m07_target_user_id_idx ON public.audit_logs_y2026m07 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m08_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m08_action_idx ON public.audit_logs_y2026m08 USING btree (action);





--

-- Name: audit_logs_y2026m08_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m08_created_at_idx ON public.audit_logs_y2026m08 USING btree (created_at);





--

-- Name: audit_logs_y2026m08_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m08_target_user_id_idx ON public.audit_logs_y2026m08 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m09_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m09_action_idx ON public.audit_logs_y2026m09 USING btree (action);





--

-- Name: audit_logs_y2026m09_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m09_created_at_idx ON public.audit_logs_y2026m09 USING btree (created_at);





--

-- Name: audit_logs_y2026m09_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m09_target_user_id_idx ON public.audit_logs_y2026m09 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m10_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m10_action_idx ON public.audit_logs_y2026m10 USING btree (action);





--

-- Name: audit_logs_y2026m10_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m10_created_at_idx ON public.audit_logs_y2026m10 USING btree (created_at);





--

-- Name: audit_logs_y2026m10_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m10_target_user_id_idx ON public.audit_logs_y2026m10 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m11_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m11_action_idx ON public.audit_logs_y2026m11 USING btree (action);





--

-- Name: audit_logs_y2026m11_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m11_created_at_idx ON public.audit_logs_y2026m11 USING btree (created_at);





--

-- Name: audit_logs_y2026m11_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m11_target_user_id_idx ON public.audit_logs_y2026m11 USING btree (target_user_id);





--

-- Name: audit_logs_y2026m12_action_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m12_action_idx ON public.audit_logs_y2026m12 USING btree (action);





--

-- Name: audit_logs_y2026m12_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m12_created_at_idx ON public.audit_logs_y2026m12 USING btree (created_at);





--

-- Name: audit_logs_y2026m12_target_user_id_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX audit_logs_y2026m12_target_user_id_idx ON public.audit_logs_y2026m12 USING btree (target_user_id);





--

-- Name: compliance_rules_organization_singleton_key; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX compliance_rules_organization_singleton_key ON public.compliance_rules USING btree ((true));





--

-- Name: dashboard_preferences_user_organization_unique; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX dashboard_preferences_user_organization_unique ON public.dashboard_preferences USING btree (user_id);





--

-- Name: idx_aal_entity; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_aal_entity ON public.approval_action_log USING btree (entity_type, entity_id);





--

-- Name: idx_aal_organization; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_aal_organization ON public.approval_action_log USING btree (acted_at);





--

-- Name: idx_audit_logs_action; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_audit_logs_action ON ONLY public.audit_logs USING btree (action);





--

-- Name: idx_audit_logs_organization_created; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_audit_logs_organization_created ON ONLY public.audit_logs USING btree (created_at);





--

-- Name: idx_audit_logs_target_user; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_audit_logs_target_user ON ONLY public.audit_logs USING btree (target_user_id);





--

-- Name: idx_aw_active_unique; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX idx_aw_active_unique ON public.approval_workflows USING btree (entity_type) WHERE (is_active = true);





--

-- Name: idx_awi_entity; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_awi_entity ON public.approval_workflow_instances USING btree (entity_type, entity_id);





--

-- Name: idx_aws_workflow; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_aws_workflow ON public.approval_workflow_steps USING btree (workflow_id, step_order);





--

-- Name: idx_awt_assignee; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_awt_assignee ON public.approval_workflow_tasks USING btree (assignee_type, assignee_target_id);





--

-- Name: idx_awt_instance; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_awt_instance ON public.approval_workflow_tasks USING btree (instance_id);





--

-- Name: idx_awt_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_awt_status ON public.approval_workflow_tasks USING btree (status);





--

-- Name: idx_behalf_assignments_assignee; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_behalf_assignments_assignee ON public.behalf_assignments USING btree (assignee_user_id, assignment_type);





--

-- Name: idx_behalf_assignments_employer; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_behalf_assignments_employer ON public.behalf_assignments USING btree (employer_id);





--

-- Name: idx_cfd_filterable; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_cfd_filterable ON public.custom_field_definitions USING btree (entity_type, is_filterable) WHERE ((deleted_at IS NULL) AND (is_filterable = true));





--

-- Name: idx_cfd_organization_entity; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_cfd_organization_entity ON public.custom_field_definitions USING btree (entity_type) WHERE (deleted_at IS NULL);





--

-- Name: idx_cfv_definition; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_cfv_definition ON public.custom_field_values USING btree (definition_id);





--

-- Name: idx_cfv_entity; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_cfv_entity ON public.custom_field_values USING btree (entity_id);





--

-- Name: idx_cfv_number_search; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_cfv_number_search ON public.custom_field_values USING btree (definition_id, value_number) WHERE (value_number IS NOT NULL);





--

-- Name: idx_cfv_text_search; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_cfv_text_search ON public.custom_field_values USING btree (definition_id, value_text) WHERE (value_text IS NOT NULL);





--

-- Name: idx_config_entries_active; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_config_entries_active ON public.config_entries USING btree (config_key) WHERE (effective_to IS NULL);





--

-- Name: idx_config_entries_lookup; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_config_entries_lookup ON public.config_entries USING btree (config_key, effective_from);





--

-- Name: idx_config_entries_scope; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_config_entries_scope ON public.config_entries USING btree (scope_type, scope_id);





--

-- Name: idx_dashboard_prefs_user_ws; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_dashboard_prefs_user_ws ON public.dashboard_preferences USING btree (user_id);





--

-- Name: idx_document_folders_parent; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_document_folders_parent ON public.document_folders USING btree (parent_folder_id);





--

-- Name: idx_feature_flags_lookup; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_feature_flags_lookup ON public.feature_flags USING btree (flag_key);





--

-- Name: idx_file_acknowledgements_document; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_file_acknowledgements_document ON public.file_acknowledgements USING btree (document_id);





--

-- Name: idx_file_acknowledgements_employee; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_file_acknowledgements_employee ON public.file_acknowledgements USING btree (employee_id);





--

-- Name: idx_infra_job_events_job_created; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_infra_job_events_job_created ON public.infra_job_events USING btree (job_id, created_at, id);





--

-- Name: idx_infra_job_events_organization_created; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_infra_job_events_organization_created ON public.infra_job_events USING btree (created_at);





--

-- Name: idx_infra_job_records_monitor_created; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_infra_job_records_monitor_created ON public.infra_job_records USING btree (created_at DESC, id DESC);





--

-- Name: idx_infra_job_records_organization_created; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_infra_job_records_organization_created ON public.infra_job_records USING btree (created_at, id);





--

-- Name: idx_infra_job_records_parent; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_infra_job_records_parent ON public.infra_job_records USING btree (parent_job_id, created_at);





--

-- Name: idx_infra_job_records_retry; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_infra_job_records_retry ON public.infra_job_records USING btree (state, next_attempt_at) WHERE ((state)::text = 'retryable'::text);





--

-- Name: idx_infra_job_records_state_lease; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_infra_job_records_state_lease ON public.infra_job_records USING btree (state, lease_expires_at);





--

-- Name: idx_lc_employment; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_lc_employment ON public.lifecycle_checklists USING btree (employment_id);





--

-- Name: idx_lc_organization_type; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_lc_organization_type ON public.lifecycle_checklists USING btree (checklist_type);





--

-- Name: idx_lc_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_lc_status ON public.lifecycle_checklists USING btree (status) WHERE ((status)::text = 'in_progress'::text);





--

-- Name: idx_lci_assignee_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_lci_assignee_status ON public.lifecycle_checklist_items USING btree (assignee_user_id, status) WHERE ((status)::text = 'pending'::text);





--

-- Name: idx_lci_checklist; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_lci_checklist ON public.lifecycle_checklist_items USING btree (checklist_id);





--

-- Name: idx_lct_organization_type; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_lct_organization_type ON public.lifecycle_checklist_templates USING btree (checklist_type) WHERE (deleted_at IS NULL);





--

-- Name: idx_lct_unique_default; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX idx_lct_unique_default ON public.lifecycle_checklist_templates USING btree (checklist_type) WHERE ((is_default = true) AND (deleted_at IS NULL));





--

-- Name: idx_lcti_template; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_lcti_template ON public.lifecycle_checklist_template_items USING btree (template_id);





--

-- Name: idx_monitor_alert_events_alert_occurred; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_monitor_alert_events_alert_occurred ON public.monitor_alert_events USING btree (alert_id, occurred_at, id);





--

-- Name: idx_monitor_alerts_last_seen; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_monitor_alerts_last_seen ON public.monitor_alerts USING btree (last_seen_at DESC, id DESC);





--

-- Name: idx_monitor_alerts_status_severity; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_monitor_alerts_status_severity ON public.monitor_alerts USING btree (status, severity, last_seen_at DESC);





--

-- Name: idx_notifications_created_at; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_notifications_created_at ON public.notifications USING btree (created_at DESC);





--

-- Name: idx_notifications_user_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_notifications_user_id ON public.notifications USING btree (user_id);





--

-- Name: idx_outbox_jobs_created_at; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_outbox_jobs_created_at ON public.outbox_jobs USING btree (created_at);





--

-- Name: idx_outbox_jobs_organization_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_outbox_jobs_organization_status ON public.outbox_jobs USING btree (status);





--

-- Name: idx_policy_templates_category; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_policy_templates_category ON public.policy_templates USING btree (category);





--

-- Name: idx_policy_templates_country; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_policy_templates_country ON public.policy_templates USING btree (country_code) WHERE (country_code IS NOT NULL);





--

-- Name: idx_roles_scope_type; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_roles_scope_type ON public.roles USING btree (scope_type);





--

-- Name: idx_sessions_token; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_sessions_token ON public.sessions USING btree (token);





--

-- Name: idx_sessions_user_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_sessions_user_id ON public.sessions USING btree (user_id);





--

-- Name: idx_sf_owner; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_sf_owner ON public.system_files USING btree (owner_type, owner_id) WHERE (is_active = true);





--

-- Name: idx_signature_requests_document; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_signature_requests_document ON public.signature_requests USING btree (document_id);





--

-- Name: idx_signature_requests_signer; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_signature_requests_signer ON public.signature_requests USING btree (signer_id);





--

-- Name: idx_system_audit_logs_action; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_system_audit_logs_action ON public.system_audit_logs USING btree (action);





--

-- Name: idx_system_audit_logs_created_at; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_system_audit_logs_created_at ON public.system_audit_logs USING btree (created_at DESC);





--

-- Name: idx_user_extra_permissions_lookup; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_user_extra_permissions_lookup ON public.user_extra_permissions USING btree (user_id, expires_at);





--

-- Name: idx_user_extra_roles_lookup; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_user_extra_roles_lookup ON public.user_extra_roles USING btree (user_id, expires_at);





--

-- Name: idx_user_roles_user_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_user_roles_user_id ON public.user_roles USING btree (user_id);





--

-- Name: idx_wf_category; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_wf_category ON public.files USING btree (category) WHERE (is_active = true);





--

-- Name: idx_wf_expires; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_wf_expires ON public.files USING btree (expires_at) WHERE ((expires_at IS NOT NULL) AND (is_active = true));





--

-- Name: idx_wf_latest; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_wf_latest ON public.files USING btree (owner_type, owner_id, is_latest) WHERE (is_latest = true);





--

-- Name: idx_wf_owner; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_wf_owner ON public.files USING btree (owner_type, owner_id) WHERE (is_active = true);





--

-- Name: idx_wf_scan_pending; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_wf_scan_pending ON public.files USING btree (scan_status) WHERE ((scan_status)::text = 'pending'::text);





--

-- Name: idx_work_locations_hq; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX idx_work_locations_hq ON public.work_locations USING btree ((true)) WHERE (is_headquarters = true);





--

-- Name: idx_working_conditions_condition_type_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_working_conditions_condition_type_id ON public.working_conditions USING btree (condition_type_id);





--

-- Name: idx_working_conditions_employment_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_working_conditions_employment_id ON public.working_conditions USING btree (employment_id);





--

-- Name: organization_settings_singleton_key; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX organization_settings_singleton_key ON public.organization_settings USING btree ((true));





--

-- Name: outbox_jobs_dedupe_key_uq; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX outbox_jobs_dedupe_key_uq ON public.outbox_jobs USING btree (dedupe_key) WHERE (dedupe_key IS NOT NULL);





--

-- Name: outbox_jobs_type_status_available_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX outbox_jobs_type_status_available_idx ON public.outbox_jobs USING btree (job_type, status, available_at);





--

-- Name: policy_templates_name_version_key; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX policy_templates_name_version_key ON public.policy_templates USING btree (name, version);





--

-- Name: roles_name_key; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX roles_name_key ON public.roles USING btree (name);





--

-- Name: storage_configs_organization_singleton_key; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX storage_configs_organization_singleton_key ON public.storage_configs USING btree ((true));





--

-- Name: uq_monitor_alerts_active_fingerprint; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX uq_monitor_alerts_active_fingerprint ON public.monitor_alerts USING btree (fingerprint) WHERE (resolved_at IS NULL);





--

-- Name: audit_logs_default_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_default_action_idx;





--

-- Name: audit_logs_default_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_default_created_at_idx;





--

-- Name: audit_logs_default_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_default_pkey;





--

-- Name: audit_logs_default_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_default_target_user_id_idx;





--

-- Name: audit_logs_y2026m01_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m01_action_idx;





--

-- Name: audit_logs_y2026m01_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m01_created_at_idx;





--

-- Name: audit_logs_y2026m01_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m01_pkey;





--

-- Name: audit_logs_y2026m01_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m01_target_user_id_idx;





--

-- Name: audit_logs_y2026m02_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m02_action_idx;





--

-- Name: audit_logs_y2026m02_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m02_created_at_idx;





--

-- Name: audit_logs_y2026m02_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m02_pkey;





--

-- Name: audit_logs_y2026m02_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m02_target_user_id_idx;





--

-- Name: audit_logs_y2026m03_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m03_action_idx;





--

-- Name: audit_logs_y2026m03_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m03_created_at_idx;





--

-- Name: audit_logs_y2026m03_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m03_pkey;





--

-- Name: audit_logs_y2026m03_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m03_target_user_id_idx;





--

-- Name: audit_logs_y2026m04_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m04_action_idx;





--

-- Name: audit_logs_y2026m04_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m04_created_at_idx;





--

-- Name: audit_logs_y2026m04_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m04_pkey;





--

-- Name: audit_logs_y2026m04_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m04_target_user_id_idx;





--

-- Name: audit_logs_y2026m05_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m05_action_idx;





--

-- Name: audit_logs_y2026m05_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m05_created_at_idx;





--

-- Name: audit_logs_y2026m05_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m05_pkey;





--

-- Name: audit_logs_y2026m05_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m05_target_user_id_idx;





--

-- Name: audit_logs_y2026m06_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m06_action_idx;





--

-- Name: audit_logs_y2026m06_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m06_created_at_idx;





--

-- Name: audit_logs_y2026m06_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m06_pkey;





--

-- Name: audit_logs_y2026m06_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m06_target_user_id_idx;





--

-- Name: audit_logs_y2026m07_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m07_action_idx;





--

-- Name: audit_logs_y2026m07_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m07_created_at_idx;





--

-- Name: audit_logs_y2026m07_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m07_pkey;





--

-- Name: audit_logs_y2026m07_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m07_target_user_id_idx;





--

-- Name: audit_logs_y2026m08_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m08_action_idx;





--

-- Name: audit_logs_y2026m08_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m08_created_at_idx;





--

-- Name: audit_logs_y2026m08_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m08_pkey;





--

-- Name: audit_logs_y2026m08_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m08_target_user_id_idx;





--

-- Name: audit_logs_y2026m09_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m09_action_idx;





--

-- Name: audit_logs_y2026m09_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m09_created_at_idx;





--

-- Name: audit_logs_y2026m09_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m09_pkey;





--

-- Name: audit_logs_y2026m09_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m09_target_user_id_idx;





--

-- Name: audit_logs_y2026m10_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m10_action_idx;





--

-- Name: audit_logs_y2026m10_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m10_created_at_idx;





--

-- Name: audit_logs_y2026m10_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m10_pkey;





--

-- Name: audit_logs_y2026m10_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m10_target_user_id_idx;





--

-- Name: audit_logs_y2026m11_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m11_action_idx;





--

-- Name: audit_logs_y2026m11_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m11_created_at_idx;





--

-- Name: audit_logs_y2026m11_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m11_pkey;





--

-- Name: audit_logs_y2026m11_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m11_target_user_id_idx;





--

-- Name: audit_logs_y2026m12_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m12_action_idx;





--

-- Name: audit_logs_y2026m12_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_organization_created ATTACH PARTITION public.audit_logs_y2026m12_created_at_idx;





--

-- Name: audit_logs_y2026m12_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m12_pkey;





--

-- Name: audit_logs_y2026m12_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m12_target_user_id_idx;





--

-- Name: compliance_rules trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.compliance_rules FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: dashboard_preferences trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.dashboard_preferences FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: feature_flags trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.feature_flags FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: infra_job_records trg_infra_job_records_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_infra_job_records_updated_at BEFORE UPDATE ON public.infra_job_records FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: monitor_alerts trg_monitor_alerts_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_monitor_alerts_updated_at BEFORE UPDATE ON public.monitor_alerts FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: outbox_jobs trigger_outbox_jobs_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trigger_outbox_jobs_updated_at BEFORE UPDATE ON public.outbox_jobs FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: policy_templates trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.policy_templates FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: roles trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.roles FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: storage_configs trg_updated_at_storage_configs; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at_storage_configs BEFORE UPDATE ON public.storage_configs FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: system_settings trg_system_settings_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_system_settings_updated_at BEFORE UPDATE ON public.system_settings FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: approval_workflow_instances approval_workflow_instances_workflow_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_workflow_instances

    ADD CONSTRAINT approval_workflow_instances_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.approval_workflows(id) ON DELETE SET NULL;





--

-- Name: approval_workflow_steps approval_workflow_steps_workflow_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_workflow_steps

    ADD CONSTRAINT approval_workflow_steps_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.approval_workflows(id) ON DELETE CASCADE;





--

-- Name: approval_workflow_tasks approval_workflow_tasks_instance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.approval_workflow_tasks

    ADD CONSTRAINT approval_workflow_tasks_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.approval_workflow_instances(id) ON DELETE CASCADE;





--

-- Name: custom_field_values custom_field_values_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.custom_field_values

    ADD CONSTRAINT custom_field_values_definition_id_fkey FOREIGN KEY (definition_id) REFERENCES public.custom_field_definitions(id) ON DELETE CASCADE;





--

-- Name: document_folders document_folders_parent_folder_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.document_folders

    ADD CONSTRAINT document_folders_parent_folder_id_fkey FOREIGN KEY (parent_folder_id) REFERENCES public.document_folders(id) ON DELETE CASCADE;





--

-- Name: file_acknowledgements file_acknowledgements_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.file_acknowledgements

    ADD CONSTRAINT file_acknowledgements_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.files(id) ON DELETE CASCADE;





--

-- Name: files fk_organization_files_folder_id; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.files

    ADD CONSTRAINT fk_organization_files_folder_id FOREIGN KEY (folder_id) REFERENCES public.document_folders(id) ON DELETE SET NULL;





--

-- Name: files organization_files_parent_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.files

    ADD CONSTRAINT organization_files_parent_file_id_fkey FOREIGN KEY (parent_file_id) REFERENCES public.files(id);





--

-- Name: infra_job_events infra_job_events_job_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.infra_job_events

    ADD CONSTRAINT infra_job_events_job_id_fkey FOREIGN KEY (job_id) REFERENCES public.infra_job_records(id) ON DELETE RESTRICT;





--

-- Name: lifecycle_checklist_items lifecycle_checklist_items_checklist_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.lifecycle_checklist_items

    ADD CONSTRAINT lifecycle_checklist_items_checklist_id_fkey FOREIGN KEY (checklist_id) REFERENCES public.lifecycle_checklists(id) ON DELETE CASCADE;





--

-- Name: lifecycle_checklist_items lifecycle_checklist_items_template_item_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.lifecycle_checklist_items

    ADD CONSTRAINT lifecycle_checklist_items_template_item_id_fkey FOREIGN KEY (template_item_id) REFERENCES public.lifecycle_checklist_template_items(id);





--

-- Name: lifecycle_checklist_template_items lifecycle_checklist_template_items_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.lifecycle_checklist_template_items

    ADD CONSTRAINT lifecycle_checklist_template_items_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.lifecycle_checklist_templates(id) ON DELETE CASCADE;





--

-- Name: lifecycle_checklists lifecycle_checklists_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.lifecycle_checklists

    ADD CONSTRAINT lifecycle_checklists_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.lifecycle_checklist_templates(id);





--

-- Name: monitor_alert_events monitor_alert_events_alert_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.monitor_alert_events

    ADD CONSTRAINT monitor_alert_events_alert_id_fkey FOREIGN KEY (alert_id) REFERENCES public.monitor_alerts(id) ON DELETE RESTRICT;





--

-- Name: notifications notifications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.notifications

    ADD CONSTRAINT notifications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;





--

-- Name: role_permissions organization_role_permissions_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.role_permissions

    ADD CONSTRAINT organization_role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE CASCADE;





--

-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.sessions

    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;





--

-- Name: signature_requests signature_requests_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.signature_requests

    ADD CONSTRAINT signature_requests_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.files(id) ON DELETE CASCADE;





--

-- Name: system_audit_logs system_audit_logs_actor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.system_audit_logs

    ADD CONSTRAINT system_audit_logs_actor_id_fkey FOREIGN KEY (actor_id) REFERENCES public.users(id) ON DELETE SET NULL;





--

-- Name: system_files system_files_parent_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.system_files

    ADD CONSTRAINT system_files_parent_file_id_fkey FOREIGN KEY (parent_file_id) REFERENCES public.system_files(id);





--

-- Name: user_extra_permissions organization_member_extra_perms_user_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.user_extra_permissions

    ADD CONSTRAINT organization_member_extra_perms_user_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;





--

-- Name: user_extra_roles member_extra_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.user_extra_roles

    ADD CONSTRAINT member_extra_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE CASCADE;





--

-- Name: user_extra_roles member_extra_roles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.user_extra_roles

    ADD CONSTRAINT member_extra_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;





--

-- Name: user_roles organization_members_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.user_roles

    ADD CONSTRAINT organization_members_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id);





--

-- Name: user_roles organization_members_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.user_roles

    ADD CONSTRAINT organization_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;





--

-- Name: users users_avatar_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.users

    ADD CONSTRAINT users_avatar_file_id_fkey FOREIGN KEY (avatar_file_id) REFERENCES public.system_files(id);





--

-- Name: working_conditions working_conditions_condition_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.working_conditions

    ADD CONSTRAINT working_conditions_condition_type_id_fkey FOREIGN KEY (condition_type_id) REFERENCES public.condition_type_definitions(id) ON DELETE SET NULL;





--

-- Name: COLUMN compliance_rules.allow_manager_override; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.compliance_rules.allow_manager_override IS 'When true, managers can override soft compliance warnings by providing a reason.';





--

-- Name: COLUMN compliance_rules.id; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.compliance_rules.id IS 'Primary key.';





--

-- Name: COLUMN compliance_rules.is_enabled; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.compliance_rules.is_enabled IS 'When false, all compliance checks are skipped for this organization.';





--

-- Name: COLUMN compliance_rules.max_monthly_ot_hours; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.compliance_rules.max_monthly_ot_hours IS 'Maximum overtime hours allowed per calendar month. Default 40 per Vietnamese Labor Code.';





--

-- Name: COLUMN compliance_rules.max_weekly_hours; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.compliance_rules.max_weekly_hours IS 'Maximum total working hours allowed per calendar week (Mon-Sun). Default 48.';





--

-- Name: COLUMN compliance_rules.min_rest_period_hours; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.compliance_rules.min_rest_period_hours IS 'Minimum rest hours required between two consecutive shifts. Default 12 per Vietnamese Labor Code.';





--

-- Name: COLUMN condition_type_definitions.affects_rules; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.condition_type_definitions.affects_rules IS 'JSON map of PolicyRules fields this type affects, e.g. {"authorized_early_leave_minutes":true,"is_early_leave_paid":true}';





--

-- Name: COLUMN condition_type_definitions.category; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.condition_type_definitions.category IS 'Grouping: working_hours, leave, general';





--

-- Name: COLUMN condition_type_definitions.code; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.condition_type_definitions.code IS 'Immutable uppercase machine key (e.g., EARLY_LEAVE, MATERNITY)';





--

-- Name: COLUMN condition_type_definitions.is_system; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.condition_type_definitions.is_system IS 'System types cannot be deleted by users';





--

-- Name: COLUMN condition_type_definitions.name_i18n; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.condition_type_definitions.name_i18n IS 'I18n names, e.g. {"en":"Early Leave","vi":"Về sớm"}';





--

-- Name: COLUMN working_conditions.condition_type_id; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.working_conditions.condition_type_id IS 'FK to condition_type_definitions. Nullable for backward compatibility with existing string-based type_ field.';





--

-- Name: EXTENSION btree_gist; Type: COMMENT; Schema: -; Owner: -

--



COMMENT ON EXTENSION btree_gist IS 'support for indexing common datatypes in GiST';





--

-- Name: EXTENSION pg_trgm; Type: COMMENT; Schema: -; Owner: -

--



COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';





--

-- Name: TABLE compliance_rules; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.compliance_rules IS 'Organization-wide compliance rule thresholds for shift conflict detection and rest-period validation.';





--

-- Name: TABLE infra_job_events; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.infra_job_events IS 'Append-only, PII-safe lifecycle transitions for asynchronous jobs.';





--

-- Name: TABLE infra_job_records; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.infra_job_records IS 'Authoritative operational lifecycle for asynchronous jobs. Raw payloads and PII are intentionally excluded.';





--

-- Name: TABLE monitor_alert_events; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.monitor_alert_events IS 'Append-only, PII-safe lifecycle events for persisted monitor alerts.';





--

-- Name: TABLE monitor_alerts; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.monitor_alerts IS 'Persisted, PII-safe infrastructure monitor alerts. Raw evaluator payloads are intentionally excluded.';





--

-- Name: TABLE organization_settings; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.organization_settings IS 'Singleton organization settings; its UUID is internal and never selected by clients.';





--

INSERT INTO public.schema_migrations(version) VALUES ('fresh_foundation_v1');

COMMIT;
