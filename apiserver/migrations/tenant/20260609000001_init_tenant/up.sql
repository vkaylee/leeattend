
CREATE EXTENSION IF NOT EXISTS btree_gist WITH SCHEMA public;

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;

CREATE TYPE public.adjustment_status_type AS ENUM (
    'pending',
    'approved',
    'rejected'
);

CREATE TYPE public.assignment_history_type AS ENUM (
    'roster',
    'pattern',
    'department_pattern',
    'fixed',
    'department_fixed'
);

CREATE TYPE public.assignment_status AS ENUM (
    'active',
    'completed',
    'cancelled'
);

CREATE TYPE public.contract_status_enum AS ENUM (
    'draft',
    'active',
    'expired',
    'terminated'
);

CREATE TYPE public.contract_type_enum AS ENUM (
    'probation',
    'training',
    'fixed_term',
    'indefinite'
);

CREATE TYPE public.early_arrival_strategy_type AS ENUM (
    'allow',
    'block',
    'clamp'
);

CREATE TYPE public.import_job_status_type AS ENUM (
    'pending',
    'processing',
    'completed',
    'failed'
);

CREATE TYPE public.leave_status_type AS ENUM (
    'pending',
    'approved',
    'rejected',
    'cancelled',
    'pre_approved'
);

CREATE TYPE public.leave_transaction_type AS ENUM (
    'accrual',
    'usage',
    'adjustment',
    'carry_over'
);

CREATE TYPE public.log_source_type AS ENUM (
    'device',
    'import',
    'manual'
);

CREATE TYPE public.member_role_type AS ENUM (
    'owner',
    'admin',
    'member',
    'hr_manager',
    'shift_manager',
    'team_lead',
    'employee'
);

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

CREATE TYPE public.ot_status_type AS ENUM (
    'none',
    'pending_employee',
    'pending_manager',
    'approved',
    'rejected'
);

CREATE TYPE public.recruitment_source_type AS ENUM (
    'direct',
    'referral',
    'agency'
);

CREATE OR REPLACE FUNCTION public.fn_cascade_delete_department_assignments() RETURNS trigger
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

CREATE OR REPLACE FUNCTION public.fn_cascade_delete_employee_assignments() RETURNS trigger
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

CREATE OR REPLACE FUNCTION public.fn_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();

RETURN NEW;

END;

$$;

CREATE OR REPLACE FUNCTION public.fn_validate_assignee_fk() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- 'workspace' type uses workspace_id (already has real FK), no extra check
    IF NEW.assignee_type = 'workspace' THEN
        RETURN NEW;

END IF;

-- assignee_id must not be null for department/employee
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

CREATE OR REPLACE FUNCTION public.fn_validate_target_fk() RETURNS trigger
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

ELSE
        RAISE EXCEPTION 'Unknown target_type: %', NEW.target_type;

END IF;

RETURN NEW;

END;

$$;

CREATE TABLE public.approval_action_log (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY NOT NULL,
    workspace_id uuid NOT NULL,
    entity_type character varying(50) NOT NULL,
    entity_id uuid NOT NULL,
    step_order integer NOT NULL,
    step_name character varying(100),
    acted_by_user_id uuid NOT NULL,
    action character varying(20) NOT NULL,
    note text,
    acted_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT approval_action_log_action_check CHECK (((action)::text = ANY ((ARRAY['approved'::character varying, 'rejected'::character varying, 'skipped'::character varying, 'returned'::character varying])::text[])))
);

CREATE TABLE public.approval_workflow_instances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    workflow_id uuid,
    entity_type character varying(50) NOT NULL,
    entity_id uuid NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    current_step_order integer DEFAULT 1 NOT NULL,
    steps_snapshot jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

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
    CONSTRAINT approval_workflow_steps_action_required_check CHECK (((action_required)::text = ANY ((ARRAY['approve'::character varying, 'review'::character varying, 'acknowledge'::character varying])::text[]))),
    CONSTRAINT approval_workflow_steps_approver_type_check CHECK (((approver_type)::text = ANY ((ARRAY['department_manager'::character varying, 'role'::character varying, 'permission'::character varying, 'specific_user'::character varying])::text[]))),
    CONSTRAINT approval_workflow_steps_step_order_check CHECK ((step_order >= 1))
);

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

CREATE TABLE public.approval_workflows (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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

CREATE TABLE public.assignment_rules (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    pattern_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    conditions jsonb NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    start_date date,
    end_date date
);

CREATE TABLE public.attendance_adjustment_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    time_entry_id uuid,
    requested_punch_in_time timestamp with time zone NOT NULL,
    requested_punch_out_time timestamp with time zone,
    reason text,
    status public.adjustment_status_type DEFAULT 'pending'::public.adjustment_status_type NOT NULL,
    requested_by_user_id uuid,
    approved_by_user_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    is_break_overtime boolean DEFAULT false NOT NULL,
    employment_id uuid NOT NULL
);

CREATE TABLE public.attendance_daily_summaries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    date date NOT NULL,
    actual_work_minutes integer DEFAULT 0 NOT NULL,
    system_detected_ot_minutes integer DEFAULT 0 NOT NULL,
    claimed_ot_minutes integer DEFAULT 0 NOT NULL,
    approved_ot_minutes integer DEFAULT 0 NOT NULL,
    ot_status public.ot_status_type DEFAULT 'none'::public.ot_status_type NOT NULL,
    ot_reason text,
    manager_note text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    claim_entered_by_user_id uuid,
    claim_entered_at timestamp with time zone,
    paper_form_photo_url text,
    entered_via text DEFAULT 'self_service'::text,
    is_holiday boolean DEFAULT false NOT NULL,
    holiday_name character varying(255),
    employment_id uuid NOT NULL
);

CREATE TABLE public.attendance_import_jobs (
    id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    status public.import_job_status_type DEFAULT 'pending'::public.import_job_status_type NOT NULL,
    total_rows integer DEFAULT 0 NOT NULL,
    processed_rows integer DEFAULT 0 NOT NULL,
    error_log text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    file_path text,
    target_type character varying(50) DEFAULT 'adjustment'::character varying NOT NULL
);

CREATE TABLE public.attendance_log_errors (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    agent_id text,
    raw_content jsonb,
    error_message text NOT NULL,
    source text NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    fingerprint text
);

CREATE TABLE public.attendance_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    is_default boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.attendance_policy_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_id uuid NOT NULL,
    target_type character varying(50) NOT NULL,
    target_id uuid NOT NULL,
    valid_from timestamp with time zone NOT NULL,
    valid_to timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.attendance_policy_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_id uuid NOT NULL,
    rules jsonb DEFAULT '{}'::jsonb NOT NULL,
    valid_from timestamp with time zone NOT NULL,
    valid_to timestamp with time zone,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.attendance_staging_logs (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
)
PARTITION BY RANGE (created_at);

CREATE TABLE public.attendance_staging_logs_default (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m01 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m02 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m03 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m04 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m05 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m06 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m07 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m08 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m09 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m10 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m11 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.attendance_staging_logs_y2026m12 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    device_id text,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

CREATE TABLE public.audit_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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

CREATE TABLE public.audit_logs_default (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m01 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m02 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m03 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m04 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m05 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m06 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m07 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m08 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m09 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m10 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m11 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.audit_logs_y2026m12 (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT audit_logs_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT audit_logs_workspace_id_not_null NOT NULL,
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

CREATE TABLE public.behalf_assignments (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT ot_claim_behalf_assignments_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT ot_claim_behalf_assignments_workspace_id_not_null NOT NULL,
    target_employee_id uuid,
    target_department_id uuid,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT ot_claim_behalf_assignments_created_at_not_null NOT NULL,
    updated_at timestamp with time zone DEFAULT now() CONSTRAINT ot_claim_behalf_assignments_updated_at_not_null NOT NULL,
    assignment_type character varying(50) NOT NULL,
    assignee_user_id uuid NOT NULL,
    CONSTRAINT check_target CHECK ((((target_employee_id IS NOT NULL) AND (target_department_id IS NULL)) OR ((target_employee_id IS NULL) AND (target_department_id IS NOT NULL))))
);

CREATE TABLE public.dashboard_preferences (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    active_view text DEFAULT 'overview'::text NOT NULL,
    visible_widgets jsonb DEFAULT '{}'::jsonb NOT NULL,
    widget_order jsonb DEFAULT '[]'::jsonb NOT NULL,
    collapsed_sections jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.departments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    manager_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    std_working_days numeric(5,2),
    deleted_at timestamp with time zone
);

CREATE TABLE public.devices (
    id uuid NOT NULL,
    agent_id character varying(50) NOT NULL,
    name character varying(100) NOT NULL,
    ip character varying(45) NOT NULL,
    port integer DEFAULT 4370,
    is_enabled boolean DEFAULT true,
    last_status character varying(20) DEFAULT 'unknown'::character varying,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    workspace_id uuid NOT NULL,
    brand text DEFAULT 'zkteco'::text NOT NULL,
    password character varying(50),
    timezone character varying(50) DEFAULT 'UTC'::character varying
);

CREATE TABLE public.employee_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    home_location_id uuid NOT NULL,
    host_location_id uuid NOT NULL,
    start_date date NOT NULL,
    end_date date,
    reason text,
    status public.assignment_status DEFAULT 'active'::public.assignment_status NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    employment_id uuid NOT NULL
);

CREATE TABLE public.employee_contracts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    contract_number character varying(100),
    contract_type public.contract_type_enum NOT NULL,
    status public.contract_status_enum DEFAULT 'active'::public.contract_status_enum NOT NULL,
    start_date date NOT NULL,
    end_date date,
    document_id uuid,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at timestamp with time zone,
    employment_id uuid NOT NULL,
    CONSTRAINT chk_contract_dates CHECK (((end_date IS NULL) OR (start_date <= end_date))),
    CONSTRAINT chk_indefinite_end_date CHECK ((((contract_type = 'indefinite'::public.contract_type_enum) AND (end_date IS NULL)) OR (contract_type <> 'indefinite'::public.contract_type_enum)))
);

CREATE TABLE public.employee_department_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    department_id uuid NOT NULL,
    effective_date date NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public.employee_documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    document_type character varying(50) NOT NULL,
    file_name character varying(255) NOT NULL,
    file_path character varying(500) NOT NULL,
    file_size integer NOT NULL,
    mime_type character varying(100) NOT NULL,
    expires_at date,
    is_active boolean DEFAULT true NOT NULL,
    uploaded_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.employee_emergency_contacts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    contact_name character varying(100) NOT NULL,
    relationship character varying(50) NOT NULL,
    phone character varying(20) NOT NULL,
    is_primary boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.employee_phases (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    employee_id uuid NOT NULL,
    phase_definition_id uuid NOT NULL,
    start_date date NOT NULL,
    end_date date NOT NULL,
    status text DEFAULT 'active'::text NOT NULL,
    override_config jsonb,
    evaluation_notes text,
    evaluated_by uuid,
    completed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT employee_phases_check CHECK ((end_date >= start_date))
);

CREATE TABLE public.employee_profiles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    phone character varying(20),
    address text,
    date_of_birth date,
    gender character varying(10),
    national_id text,
    bank_name character varying(100),
    bank_account_number text,
    bank_branch character varying(100),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.employee_salary_components (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    component_definition_id uuid NOT NULL,
    amount numeric(15,2) DEFAULT 0.0 NOT NULL,
    effective_date date NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public.employee_shifts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    shift_id uuid NOT NULL,
    start_date date NOT NULL,
    end_date date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public.employee_working_patterns (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    employee_id uuid NOT NULL,
    pattern_id uuid NOT NULL,
    start_date date NOT NULL,
    end_date date,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    anchor_date date,
    CONSTRAINT employee_working_patterns_check CHECK (((end_date IS NULL) OR (end_date >= start_date)))
);

COMMENT ON COLUMN public.employee_working_patterns.anchor_date IS 'Reference start date for rotation patterns. Determines phase offset per assignment.';

CREATE TABLE public.employees (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    user_id uuid,
    employee_id_number character varying(50) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    tags text[] DEFAULT '{}'::text[],
    full_name character varying(255) NOT NULL,
    recruitment_source_id uuid,
    deleted_at timestamp with time zone,
    CONSTRAINT chk_employee_full_name CHECK ((length((full_name)::text) > 0))
);

CREATE TABLE public.employers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    tax_code character varying(50),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    legal_address character varying(255),
    deleted_at timestamp with time zone,
    parent_id uuid,
    relationship_type character varying(100)
);

CREATE TABLE public.employment_phase_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    description text,
    phase_type text DEFAULT 'custom'::text NOT NULL,
    rule_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    display_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.employments (
    id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    employee_id uuid NOT NULL,
    employer_id uuid NOT NULL,
    primary_location_id uuid,
    department_id uuid,
    status character varying(50) NOT NULL,
    start_date date NOT NULL,
    end_date date,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    job_title character varying(100),
    employment_type character varying(50) DEFAULT 'full_time'::character varying NOT NULL,
    manager_employment_id uuid
);

CREATE TABLE public.feature_flags (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    flag_key character varying(128) NOT NULL,
    enabled boolean DEFAULT false NOT NULL,
    description text,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.holidays (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    date date NOT NULL,
    holiday_type character varying(50) DEFAULT 'national'::character varying NOT NULL,
    is_paid boolean DEFAULT true NOT NULL,
    ot_multiplier numeric(3,1),
    description text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    leave_type_id uuid,
    work_threshold_percent integer,
    is_credit_on_work boolean DEFAULT true NOT NULL,
    deleted_at timestamp with time zone,
    location_id uuid
);

CREATE TABLE public.leave_accrual_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    employee_id uuid NOT NULL,
    policy_assignment_id uuid,
    accrual_date date NOT NULL,
    amount numeric(5,2) NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    leave_type_id uuid NOT NULL
);

CREATE TABLE public.leave_balances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    year integer NOT NULL,
    entitlement integer DEFAULT 12 NOT NULL,
    carried_over integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    accrued_amount numeric(5,2) DEFAULT 0.00 NOT NULL,
    leave_type_id uuid NOT NULL,
    employment_id uuid NOT NULL
);

CREATE TABLE public.leave_policy_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    policy_id uuid NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    deleted_at timestamp with time zone,
    CONSTRAINT leave_policy_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text])))
);

CREATE TABLE public.leave_policy_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    description text,
    rule_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    leave_type_id uuid NOT NULL,
    deleted_at timestamp with time zone
);

CREATE TABLE public.leave_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    start_date date NOT NULL,
    end_date date NOT NULL,
    reason text,
    status public.leave_status_type DEFAULT 'pending'::public.leave_status_type NOT NULL,
    requested_by_user_id uuid,
    approved_by_user_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    leave_type_id uuid NOT NULL,
    duration_type character varying(20) DEFAULT 'full_day'::character varying NOT NULL,
    start_time time without time zone,
    end_time time without time zone,
    employment_id uuid NOT NULL,
    CONSTRAINT chk_start_before_end CHECK ((start_date <= end_date))
);

CREATE TABLE public.leave_types (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name text NOT NULL,
    is_paid boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    policy_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    deleted_at timestamp with time zone
);

CREATE TABLE public.outbox_jobs (
    id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    job_type character varying(50) NOT NULL,
    payload jsonb NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    error_message text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.overtime_policy_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    policy_id uuid NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT overtime_policy_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text])))
);

CREATE TABLE public.overtime_policy_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public.overtime_request_employees (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    overtime_request_id uuid NOT NULL,
    consent_status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    consent_at timestamp with time zone,
    decline_reason text,
    actual_ot_minutes integer,
    matched_summary_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    employment_id uuid NOT NULL,
    CONSTRAINT overtime_request_employees_consent_status_check CHECK (((consent_status)::text = ANY ((ARRAY['pending'::character varying, 'accepted'::character varying, 'declined'::character varying])::text[])))
);

CREATE TABLE public.overtime_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    requested_by_user_id uuid NOT NULL,
    department_id uuid,
    planned_date date NOT NULL,
    planned_start_time time without time zone,
    planned_end_time time without time zone,
    planned_minutes integer NOT NULL,
    reason text NOT NULL,
    status public.ot_request_status_type DEFAULT 'draft'::public.ot_request_status_type NOT NULL,
    workflow_id uuid,
    current_step integer DEFAULT 0 NOT NULL,
    quota_warning jsonb,
    reconciled_at timestamp with time zone,
    reconciled_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT overtime_requests_planned_minutes_check CHECK ((planned_minutes > 0))
);

CREATE TABLE public.overtime_rules (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_id uuid NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    multiplier numeric(5,2) DEFAULT 1.50 NOT NULL,
    time_start time without time zone,
    time_end time without time zone,
    days_of_week integer DEFAULT 127 NOT NULL,
    is_holiday boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public.payroll_incentive_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    policy_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_incentive_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text])))
);

CREATE TABLE public.payroll_incentive_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    base_amount double precision NOT NULL,
    payout_type text DEFAULT 'fixed'::text NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    effective_from date DEFAULT CURRENT_DATE NOT NULL,
    effective_to date,
    deleted_at timestamp with time zone
);

CREATE TABLE public.payroll_incentive_tiers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_id uuid NOT NULL,
    priority integer NOT NULL,
    payout_amount double precision NOT NULL,
    rule_config jsonb DEFAULT '{}'::jsonb NOT NULL
);

CREATE TABLE public.payroll_rule_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    rule_definition_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    effective_from date NOT NULL,
    effective_to date,
    override_config jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_rule_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text])))
);

CREATE TABLE public.payroll_rule_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    description text,
    code text NOT NULL,
    category text DEFAULT 'deduction'::text NOT NULL,
    applies_to text DEFAULT 'employee'::text NOT NULL,
    calculation_method text DEFAULT 'percentage'::text NOT NULL,
    rule_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    is_mandatory boolean DEFAULT false NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    display_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.payroll_rule_tiers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    rule_definition_id uuid NOT NULL,
    tier_order integer NOT NULL,
    min_amount numeric(15,2) NOT NULL,
    max_amount numeric(15,2),
    rate numeric(8,4) NOT NULL,
    fixed_amount numeric(15,2) DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_rule_tiers_check CHECK (((max_amount IS NULL) OR (max_amount > min_amount)))
);

CREATE TABLE public.payslip_incentives (
    payslip_id uuid NOT NULL,
    policy_name text NOT NULL,
    amount double precision NOT NULL,
    tier_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    standard_amount double precision DEFAULT 0.0 NOT NULL,
    id uuid DEFAULT gen_random_uuid() NOT NULL
);

CREATE TABLE public.payslip_rule_evaluations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    payslip_id uuid NOT NULL,
    rule_code text NOT NULL,
    amount numeric(15,2) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.payslip_segments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    payslip_id uuid NOT NULL,
    start_date date NOT NULL,
    end_date date NOT NULL,
    department_id uuid,
    applied_base_salary numeric(15,2) NOT NULL,
    applied_std_days numeric(5,2) NOT NULL,
    actual_work_days numeric(5,2) DEFAULT 0 NOT NULL,
    paid_work_days numeric(5,2) DEFAULT 0 NOT NULL,
    amount numeric(15,2) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    applied_hours_per_day numeric(5,2) NOT NULL,
    std_days_source character varying(50)
);

CREATE TABLE public.payslips (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    period_start date NOT NULL,
    period_end date NOT NULL,
    base_pay_amount numeric(15,2) NOT NULL,
    overtime_pay_amount numeric(15,2) DEFAULT 0 NOT NULL,
    deductions_amount numeric(15,2) DEFAULT 0 NOT NULL,
    net_pay_amount numeric(15,2) NOT NULL,
    status character varying(20) DEFAULT 'draft'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    standard_units integer DEFAULT 0 NOT NULL,
    overtime_units integer DEFAULT 0 NOT NULL,
    holiday_work_premium_amount numeric(15,2) DEFAULT 0 NOT NULL,
    workspace_id uuid NOT NULL,
    employment_id uuid NOT NULL
);

CREATE TABLE public.policy_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid,
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

CREATE TABLE public.raw_attendance_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    punch_time timestamp with time zone NOT NULL,
    source public.log_source_type NOT NULL,
    device_id character varying(50),
    raw_data jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    punch integer DEFAULT '-1'::integer NOT NULL,
    status integer DEFAULT '-1'::integer NOT NULL,
    employment_id uuid NOT NULL
);

CREATE TABLE public.recruitment_sources (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    source_type public.recruitment_source_type NOT NULL,
    name character varying(255) NOT NULL,
    referrer_employee_id uuid,
    referral_bonus_amount numeric(15,2),
    referral_bonus_paid boolean DEFAULT false NOT NULL,
    agency_contact_name character varying(255),
    agency_contact_email character varying(255),
    agency_fee_amount numeric(15,2),
    agency_fee_paid boolean DEFAULT false NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.salary_base_groups (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid,
    name character varying(255) NOT NULL,
    slug character varying(100) NOT NULL,
    description text,
    is_system boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.salary_component_base_members (
    component_definition_id uuid NOT NULL,
    base_group_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.salary_component_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(100) NOT NULL,
    code character varying(50) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    component_type text DEFAULT 'allowance'::text NOT NULL,
    linked_rule_id uuid
);

CREATE TABLE public.shift_rosters (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    shift_id uuid,
    date date NOT NULL,
    status character varying(20) DEFAULT 'confirmed'::character varying,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    employment_id uuid NOT NULL
);

CREATE TABLE public.shift_swap_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    requester_id uuid NOT NULL,
    target_id uuid NOT NULL,
    swap_date date NOT NULL,
    requester_shift_id uuid,
    target_shift_id uuid,
    status character varying(20) DEFAULT 'pending_admin'::character varying NOT NULL,
    reason text,
    requested_by_user_id uuid,
    approved_by_user_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.shifts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(100) NOT NULL,
    start_time time without time zone NOT NULL,
    end_time time without time zone NOT NULL,
    break_duration_minutes integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    break_is_overtime boolean DEFAULT false NOT NULL,
    early_punch_in_grace_period_minutes integer DEFAULT 0 NOT NULL,
    break_start time without time zone,
    break_end time without time zone,
    is_break_auto_deduct boolean DEFAULT true NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    parent_shift_id uuid,
    allow_early_ot boolean DEFAULT false NOT NULL,
    report_symbol character varying(10),
    max_early_ot_minutes integer,
    location_id uuid,
    employer_id uuid
);

CREATE TABLE public.time_entries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    punch_in_time timestamp with time zone NOT NULL,
    punch_out_time timestamp with time zone,
    duration_minutes integer,
    entry_type character varying(50) DEFAULT 'work'::character varying NOT NULL,
    note text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    is_break_overtime boolean DEFAULT false NOT NULL,
    shift_id uuid,
    generation_metadata jsonb DEFAULT '{}'::jsonb,
    workspace_id uuid NOT NULL,
    employment_id uuid NOT NULL
);

CREATE TABLE public.work_locations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    address text,
    timezone character varying(50) DEFAULT 'UTC'::character varying NOT NULL,
    is_headquarters boolean DEFAULT false NOT NULL,
    status character varying(20) DEFAULT 'active'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.working_conditions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    type character varying(50) NOT NULL,
    minute_allowance integer NOT NULL,
    is_paid boolean DEFAULT false NOT NULL,
    start_date date NOT NULL,
    end_date date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    employment_id uuid NOT NULL
);

CREATE TABLE public.working_pattern_details (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pattern_id uuid NOT NULL,
    schedule_day integer CONSTRAINT working_pattern_details_day_of_week_not_null NOT NULL,
    shift_id uuid,
    CONSTRAINT working_pattern_details_schedule_day_check CHECK ((schedule_day >= 0))
);

COMMENT ON COLUMN public.working_pattern_details.schedule_day IS 'For weekly patterns: day_of_week (0=Sun..6=Sat). For rotation patterns: cycle_day (0..N-1)';

CREATE TABLE public.working_patterns (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    description text,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    pattern_type character varying(10) DEFAULT 'weekly'::character varying NOT NULL,
    cycle_length_days integer,
    deleted_at timestamp with time zone,
    CONSTRAINT chk_rotation_fields CHECK (((((pattern_type)::text = 'weekly'::text) AND (cycle_length_days IS NULL)) OR (((pattern_type)::text = 'rotation'::text) AND (cycle_length_days IS NOT NULL) AND (cycle_length_days > 0))))
);

CREATE TABLE public.workspace_config_entries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    scope_type character varying(20) DEFAULT 'workspace'::character varying NOT NULL,
    scope_id uuid,
    config_key character varying(100) NOT NULL,
    value_int integer,
    value_decimal numeric(12,4),
    value_text text,
    value_bool boolean,
    value_json jsonb,
    effective_from date NOT NULL,
    effective_to date,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT workspace_config_entries_check CHECK ((((((((value_int IS NOT NULL))::integer + ((value_decimal IS NOT NULL))::integer) + ((value_text IS NOT NULL))::integer) + ((value_bool IS NOT NULL))::integer) + ((value_json IS NOT NULL))::integer) = 1)),
    CONSTRAINT workspace_config_entries_scope_type_check CHECK (((scope_type)::text = ANY ((ARRAY['workspace'::character varying, 'employer'::character varying, 'department'::character varying, 'employee'::character varying])::text[])))
);

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_default DEFAULT;

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m01 FOR VALUES FROM ('2026-01-01 00:00:00+00') TO ('2026-02-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m02 FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m03 FOR VALUES FROM ('2026-03-01 00:00:00+00') TO ('2026-04-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m04 FOR VALUES FROM ('2026-04-01 00:00:00+00') TO ('2026-05-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m05 FOR VALUES FROM ('2026-05-01 00:00:00+00') TO ('2026-06-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m06 FOR VALUES FROM ('2026-06-01 00:00:00+00') TO ('2026-07-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m07 FOR VALUES FROM ('2026-07-01 00:00:00+00') TO ('2026-08-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m08 FOR VALUES FROM ('2026-08-01 00:00:00+00') TO ('2026-09-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m09 FOR VALUES FROM ('2026-09-01 00:00:00+00') TO ('2026-10-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m10 FOR VALUES FROM ('2026-10-01 00:00:00+00') TO ('2026-11-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m11 FOR VALUES FROM ('2026-11-01 00:00:00+00') TO ('2026-12-01 00:00:00+00');

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m12 FOR VALUES FROM ('2026-12-01 00:00:00+00') TO ('2027-01-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_default DEFAULT;

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m01 FOR VALUES FROM ('2026-01-01 00:00:00+00') TO ('2026-02-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m02 FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m03 FOR VALUES FROM ('2026-03-01 00:00:00+00') TO ('2026-04-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m04 FOR VALUES FROM ('2026-04-01 00:00:00+00') TO ('2026-05-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m05 FOR VALUES FROM ('2026-05-01 00:00:00+00') TO ('2026-06-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m06 FOR VALUES FROM ('2026-06-01 00:00:00+00') TO ('2026-07-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m07 FOR VALUES FROM ('2026-07-01 00:00:00+00') TO ('2026-08-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m08 FOR VALUES FROM ('2026-08-01 00:00:00+00') TO ('2026-09-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m09 FOR VALUES FROM ('2026-09-01 00:00:00+00') TO ('2026-10-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m10 FOR VALUES FROM ('2026-10-01 00:00:00+00') TO ('2026-11-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m11 FOR VALUES FROM ('2026-11-01 00:00:00+00') TO ('2026-12-01 00:00:00+00');

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m12 FOR VALUES FROM ('2026-12-01 00:00:00+00') TO ('2027-01-01 00:00:00+00');

ALTER TABLE ONLY public.approval_workflow_instances
    ADD CONSTRAINT approval_workflow_instances_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.approval_workflow_steps
    ADD CONSTRAINT approval_workflow_steps_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.approval_workflow_steps
    ADD CONSTRAINT approval_workflow_steps_workflow_id_step_order_key UNIQUE (workflow_id, step_order);

ALTER TABLE ONLY public.approval_workflow_tasks
    ADD CONSTRAINT approval_workflow_tasks_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.approval_workflows
    ADD CONSTRAINT approval_workflows_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.assignment_rules
    ADD CONSTRAINT assignment_rules_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_employment_id_date_key UNIQUE (employment_id, date);

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_import_jobs
    ADD CONSTRAINT attendance_import_jobs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_log_errors
    ADD CONSTRAINT attendance_log_errors_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_policies
    ADD CONSTRAINT attendance_policies_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_policy_assignments
    ADD CONSTRAINT attendance_policy_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_policy_versions
    ADD CONSTRAINT attendance_policy_versions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_staging_logs
    ADD CONSTRAINT attendance_staging_logs_pkey1 PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_default
    ADD CONSTRAINT attendance_staging_logs_default_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m01
    ADD CONSTRAINT attendance_staging_logs_y2026m01_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m02
    ADD CONSTRAINT attendance_staging_logs_y2026m02_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m03
    ADD CONSTRAINT attendance_staging_logs_y2026m03_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m04
    ADD CONSTRAINT attendance_staging_logs_y2026m04_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m05
    ADD CONSTRAINT attendance_staging_logs_y2026m05_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m06
    ADD CONSTRAINT attendance_staging_logs_y2026m06_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m07
    ADD CONSTRAINT attendance_staging_logs_y2026m07_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m08
    ADD CONSTRAINT attendance_staging_logs_y2026m08_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m09
    ADD CONSTRAINT attendance_staging_logs_y2026m09_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m10
    ADD CONSTRAINT attendance_staging_logs_y2026m10_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m11
    ADD CONSTRAINT attendance_staging_logs_y2026m11_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.attendance_staging_logs_y2026m12
    ADD CONSTRAINT attendance_staging_logs_y2026m12_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs
    ADD CONSTRAINT audit_logs_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_default
    ADD CONSTRAINT audit_logs_default_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m01
    ADD CONSTRAINT audit_logs_y2026m01_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m02
    ADD CONSTRAINT audit_logs_y2026m02_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m03
    ADD CONSTRAINT audit_logs_y2026m03_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m04
    ADD CONSTRAINT audit_logs_y2026m04_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m05
    ADD CONSTRAINT audit_logs_y2026m05_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m06
    ADD CONSTRAINT audit_logs_y2026m06_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m07
    ADD CONSTRAINT audit_logs_y2026m07_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m08
    ADD CONSTRAINT audit_logs_y2026m08_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m09
    ADD CONSTRAINT audit_logs_y2026m09_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m10
    ADD CONSTRAINT audit_logs_y2026m10_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m11
    ADD CONSTRAINT audit_logs_y2026m11_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.audit_logs_y2026m12
    ADD CONSTRAINT audit_logs_y2026m12_pkey PRIMARY KEY (id, created_at);

ALTER TABLE ONLY public.dashboard_preferences
    ADD CONSTRAINT dashboard_preferences_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.dashboard_preferences
    ADD CONSTRAINT dashboard_preferences_user_id_workspace_id_key UNIQUE (user_id, workspace_id);

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_department_history
    ADD CONSTRAINT employee_department_history_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_emergency_contacts
    ADD CONSTRAINT employee_emergency_contacts_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_employee_id_key UNIQUE (employee_id);

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_salary_components
    ADD CONSTRAINT employee_salary_components_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_shifts
    ADD CONSTRAINT employee_shifts_new_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_working_patterns
    ADD CONSTRAINT employee_working_patterns_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employers
    ADD CONSTRAINT employers_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_workspace_id_name_key UNIQUE (workspace_id, name);

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.feature_flags
    ADD CONSTRAINT feature_flags_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.feature_flags
    ADD CONSTRAINT feature_flags_workspace_id_flag_key_key UNIQUE (workspace_id, flag_key);

ALTER TABLE ONLY public.holidays
    ADD CONSTRAINT holidays_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.holidays
    ADD CONSTRAINT holidays_workspace_id_date_key UNIQUE (workspace_id, date);

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_employee_leave_type_id_date_key UNIQUE (employee_id, leave_type_id, accrual_date);

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_employment_id_leave_type_id_year_key UNIQUE (employment_id, leave_type_id, year);

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_workspace_id_assignee_type_coales_excl EXCLUDE USING gist (workspace_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&);

ALTER TABLE ONLY public.leave_policy_definitions
    ADD CONSTRAINT leave_policy_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_types
    ADD CONSTRAINT leave_types_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_types
    ADD CONSTRAINT leave_types_workspace_id_code_key UNIQUE (workspace_id, code);

ALTER TABLE ONLY public.attendance_policy_assignments
    ADD CONSTRAINT no_overlapping_policy_assignments EXCLUDE USING gist (target_id WITH =, target_type WITH =, tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamp with time zone)) WITH &&);

ALTER TABLE ONLY public.attendance_policy_versions
    ADD CONSTRAINT no_overlapping_policy_versions EXCLUDE USING gist (policy_id WITH =, tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamp with time zone)) WITH &&);

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT ot_claim_behalf_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.outbox_jobs
    ADD CONSTRAINT outbox_jobs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_workspace_id_assignee_type_coa_excl EXCLUDE USING gist (workspace_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&);

ALTER TABLE ONLY public.overtime_policy_definitions
    ADD CONSTRAINT overtime_policy_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_overtime_request_id_employment_id_ke UNIQUE (overtime_request_id, employment_id);

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_rules
    ADD CONSTRAINT overtime_rules_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_incentive_assignments
    ADD CONSTRAINT payroll_incentive_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_incentive_policies
    ADD CONSTRAINT payroll_incentive_policies_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_incentive_tiers
    ADD CONSTRAINT payroll_incentive_tiers_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_workspace_id_rule_definition_id_a_excl EXCLUDE USING gist (workspace_id WITH =, rule_definition_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&);

ALTER TABLE ONLY public.payroll_rule_definitions
    ADD CONSTRAINT payroll_rule_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_rule_definitions
    ADD CONSTRAINT payroll_rule_definitions_workspace_id_code_key UNIQUE (workspace_id, code);

ALTER TABLE ONLY public.payroll_rule_tiers
    ADD CONSTRAINT payroll_rule_tiers_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_rule_tiers
    ADD CONSTRAINT payroll_rule_tiers_rule_definition_id_tier_order_key UNIQUE (rule_definition_id, tier_order);

ALTER TABLE ONLY public.payslip_incentives
    ADD CONSTRAINT payslip_incentives_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payslip_rule_evaluations
    ADD CONSTRAINT payslip_rule_evaluations_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payslip_segments
    ADD CONSTRAINT payslip_segments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payslips
    ADD CONSTRAINT payslips_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.policy_templates
    ADD CONSTRAINT policy_templates_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT raw_attendance_logs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.recruitment_sources
    ADD CONSTRAINT recruitment_sources_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.salary_base_groups
    ADD CONSTRAINT salary_base_groups_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.salary_component_base_members
    ADD CONSTRAINT salary_component_base_members_pkey PRIMARY KEY (component_definition_id, base_group_id);

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_workspace_id_code_key UNIQUE (workspace_id, code);

ALTER TABLE ONLY public.shift_rosters
    ADD CONSTRAINT shift_rosters_employment_id_date_key UNIQUE (employment_id, date);

ALTER TABLE ONLY public.shift_rosters
    ADD CONSTRAINT shift_rosters_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.shifts
    ADD CONSTRAINT shifts_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT uniq_behalf_assignment UNIQUE (workspace_id, assignee_user_id, target_employee_id, target_department_id, assignment_type);

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT unique_raw_log UNIQUE (workspace_id, employment_id, punch_time, punch, status);

ALTER TABLE ONLY public.employee_shifts
    ADD CONSTRAINT uq_employee_shift_start UNIQUE (employee_id, start_date);

ALTER TABLE ONLY public.work_locations
    ADD CONSTRAINT work_locations_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.working_pattern_details
    ADD CONSTRAINT working_pattern_details_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.working_patterns
    ADD CONSTRAINT working_patterns_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.workspace_config_entries
    ADD CONSTRAINT workspace_config_entries_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.workspace_config_entries
    ADD CONSTRAINT workspace_config_entries_workspace_id_scope_type_scope_id_c_key UNIQUE (workspace_id, scope_type, scope_id, config_key, effective_from);

CREATE UNIQUE INDEX attendance_policies_workspace_default_idx ON public.attendance_policies USING btree (workspace_id) WHERE (is_default = true);

CREATE INDEX idx_staging_logs_fingerprint ON ONLY public.attendance_staging_logs USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_default_fingerprint_idx ON public.attendance_staging_logs_default USING btree (fingerprint);

CREATE INDEX idx_staging_logs_workspace_status ON ONLY public.attendance_staging_logs USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_default_workspace_id_status_idx ON public.attendance_staging_logs_default USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m01_fingerprint_idx ON public.attendance_staging_logs_y2026m01 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m01_workspace_id_status_idx ON public.attendance_staging_logs_y2026m01 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m02_fingerprint_idx ON public.attendance_staging_logs_y2026m02 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m02_workspace_id_status_idx ON public.attendance_staging_logs_y2026m02 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m03_fingerprint_idx ON public.attendance_staging_logs_y2026m03 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m03_workspace_id_status_idx ON public.attendance_staging_logs_y2026m03 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m04_fingerprint_idx ON public.attendance_staging_logs_y2026m04 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m04_workspace_id_status_idx ON public.attendance_staging_logs_y2026m04 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m05_fingerprint_idx ON public.attendance_staging_logs_y2026m05 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m05_workspace_id_status_idx ON public.attendance_staging_logs_y2026m05 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m06_fingerprint_idx ON public.attendance_staging_logs_y2026m06 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m06_workspace_id_status_idx ON public.attendance_staging_logs_y2026m06 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m07_fingerprint_idx ON public.attendance_staging_logs_y2026m07 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m07_workspace_id_status_idx ON public.attendance_staging_logs_y2026m07 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m08_fingerprint_idx ON public.attendance_staging_logs_y2026m08 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m08_workspace_id_status_idx ON public.attendance_staging_logs_y2026m08 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m09_fingerprint_idx ON public.attendance_staging_logs_y2026m09 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m09_workspace_id_status_idx ON public.attendance_staging_logs_y2026m09 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m10_fingerprint_idx ON public.attendance_staging_logs_y2026m10 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m10_workspace_id_status_idx ON public.attendance_staging_logs_y2026m10 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m11_fingerprint_idx ON public.attendance_staging_logs_y2026m11 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m11_workspace_id_status_idx ON public.attendance_staging_logs_y2026m11 USING btree (workspace_id, status);

CREATE INDEX attendance_staging_logs_y2026m12_fingerprint_idx ON public.attendance_staging_logs_y2026m12 USING btree (fingerprint);

CREATE INDEX attendance_staging_logs_y2026m12_workspace_id_status_idx ON public.attendance_staging_logs_y2026m12 USING btree (workspace_id, status);

CREATE INDEX idx_audit_logs_action ON ONLY public.audit_logs USING btree (action);

CREATE INDEX audit_logs_default_action_idx ON public.audit_logs_default USING btree (action);

CREATE INDEX idx_audit_logs_target_user ON ONLY public.audit_logs USING btree (target_user_id);

CREATE INDEX audit_logs_default_target_user_id_idx ON public.audit_logs_default USING btree (target_user_id);

CREATE INDEX idx_audit_logs_workspace_created ON ONLY public.audit_logs USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_default_workspace_id_created_at_idx ON public.audit_logs_default USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m01_action_idx ON public.audit_logs_y2026m01 USING btree (action);

CREATE INDEX audit_logs_y2026m01_target_user_id_idx ON public.audit_logs_y2026m01 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m01_workspace_id_created_at_idx ON public.audit_logs_y2026m01 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m02_action_idx ON public.audit_logs_y2026m02 USING btree (action);

CREATE INDEX audit_logs_y2026m02_target_user_id_idx ON public.audit_logs_y2026m02 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m02_workspace_id_created_at_idx ON public.audit_logs_y2026m02 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m03_action_idx ON public.audit_logs_y2026m03 USING btree (action);

CREATE INDEX audit_logs_y2026m03_target_user_id_idx ON public.audit_logs_y2026m03 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m03_workspace_id_created_at_idx ON public.audit_logs_y2026m03 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m04_action_idx ON public.audit_logs_y2026m04 USING btree (action);

CREATE INDEX audit_logs_y2026m04_target_user_id_idx ON public.audit_logs_y2026m04 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m04_workspace_id_created_at_idx ON public.audit_logs_y2026m04 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m05_action_idx ON public.audit_logs_y2026m05 USING btree (action);

CREATE INDEX audit_logs_y2026m05_target_user_id_idx ON public.audit_logs_y2026m05 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m05_workspace_id_created_at_idx ON public.audit_logs_y2026m05 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m06_action_idx ON public.audit_logs_y2026m06 USING btree (action);

CREATE INDEX audit_logs_y2026m06_target_user_id_idx ON public.audit_logs_y2026m06 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m06_workspace_id_created_at_idx ON public.audit_logs_y2026m06 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m07_action_idx ON public.audit_logs_y2026m07 USING btree (action);

CREATE INDEX audit_logs_y2026m07_target_user_id_idx ON public.audit_logs_y2026m07 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m07_workspace_id_created_at_idx ON public.audit_logs_y2026m07 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m08_action_idx ON public.audit_logs_y2026m08 USING btree (action);

CREATE INDEX audit_logs_y2026m08_target_user_id_idx ON public.audit_logs_y2026m08 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m08_workspace_id_created_at_idx ON public.audit_logs_y2026m08 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m09_action_idx ON public.audit_logs_y2026m09 USING btree (action);

CREATE INDEX audit_logs_y2026m09_target_user_id_idx ON public.audit_logs_y2026m09 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m09_workspace_id_created_at_idx ON public.audit_logs_y2026m09 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m10_action_idx ON public.audit_logs_y2026m10 USING btree (action);

CREATE INDEX audit_logs_y2026m10_target_user_id_idx ON public.audit_logs_y2026m10 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m10_workspace_id_created_at_idx ON public.audit_logs_y2026m10 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m11_action_idx ON public.audit_logs_y2026m11 USING btree (action);

CREATE INDEX audit_logs_y2026m11_target_user_id_idx ON public.audit_logs_y2026m11 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m11_workspace_id_created_at_idx ON public.audit_logs_y2026m11 USING btree (workspace_id, created_at DESC);

CREATE INDEX audit_logs_y2026m12_action_idx ON public.audit_logs_y2026m12 USING btree (action);

CREATE INDEX audit_logs_y2026m12_target_user_id_idx ON public.audit_logs_y2026m12 USING btree (target_user_id);

CREATE INDEX audit_logs_y2026m12_workspace_id_created_at_idx ON public.audit_logs_y2026m12 USING btree (workspace_id, created_at DESC);

CREATE INDEX idx_aal_entity ON public.approval_action_log USING btree (entity_type, entity_id);

CREATE INDEX idx_aal_workspace ON public.approval_action_log USING btree (workspace_id, acted_at);

CREATE INDEX idx_assignment_rules_pattern_id ON public.assignment_rules USING btree (pattern_id);

CREATE INDEX idx_assignment_rules_priority ON public.assignment_rules USING btree (priority DESC);

CREATE INDEX idx_assignment_rules_workspace_id ON public.assignment_rules USING btree (workspace_id);

CREATE INDEX idx_attendance_adj_time_entry_id ON public.attendance_adjustment_requests USING btree (time_entry_id);

CREATE INDEX idx_attendance_adjustment_requests_employment_id ON public.attendance_adjustment_requests USING btree (employment_id);

CREATE INDEX idx_attendance_adjustment_requests_status ON public.attendance_adjustment_requests USING btree (status);

CREATE INDEX idx_attendance_daily_summaries_employment_id ON public.attendance_daily_summaries USING btree (employment_id);

CREATE INDEX idx_attendance_import_jobs_status ON public.attendance_import_jobs USING btree (status);

CREATE INDEX idx_attendance_import_jobs_workspace_id ON public.attendance_import_jobs USING btree (workspace_id);

CREATE INDEX idx_attendance_policy_assignments_target ON public.attendance_policy_assignments USING btree (target_type, target_id);

CREATE INDEX idx_attendance_policy_versions_rules ON public.attendance_policy_versions USING gin (rules);

CREATE UNIQUE INDEX idx_aw_active_unique ON public.approval_workflows USING btree (workspace_id, entity_type) WHERE (is_active = true);

CREATE INDEX idx_aw_workspace ON public.approval_workflows USING btree (workspace_id);

CREATE INDEX idx_awi_entity ON public.approval_workflow_instances USING btree (entity_type, entity_id);

CREATE INDEX idx_awi_workspace ON public.approval_workflow_instances USING btree (workspace_id);

CREATE INDEX idx_aws_workflow ON public.approval_workflow_steps USING btree (workflow_id, step_order);

CREATE INDEX idx_awt_assignee ON public.approval_workflow_tasks USING btree (assignee_type, assignee_target_id);

CREATE INDEX idx_awt_instance ON public.approval_workflow_tasks USING btree (instance_id);

CREATE INDEX idx_awt_status ON public.approval_workflow_tasks USING btree (status);

CREATE INDEX idx_behalf_assignments_assignee ON public.behalf_assignments USING btree (workspace_id, assignee_user_id, assignment_type);

CREATE INDEX idx_daily_summaries_status ON public.attendance_daily_summaries USING btree (ot_status);

CREATE INDEX idx_daily_summaries_workspace_date ON public.attendance_daily_summaries USING btree (workspace_id, date);

CREATE INDEX idx_dashboard_prefs_user_ws ON public.dashboard_preferences USING btree (user_id, workspace_id);

CREATE INDEX idx_departments_manager_id ON public.departments USING btree (manager_id);

CREATE INDEX idx_departments_workspace_id ON public.departments USING btree (workspace_id);

CREATE INDEX idx_devices_agent ON public.devices USING btree (agent_id);

CREATE INDEX idx_devices_workspace ON public.devices USING btree (workspace_id);

CREATE INDEX idx_emp_dept_hist_department_id ON public.employee_department_history USING btree (department_id);

CREATE INDEX idx_emp_dept_hist_emp_date ON public.employee_department_history USING btree (employee_id, effective_date DESC);

CREATE INDEX idx_emp_dept_hist_employee_id ON public.employee_department_history USING btree (employee_id);

CREATE INDEX idx_employee_assignments_employment_id ON public.employee_assignments USING btree (employment_id);

CREATE INDEX idx_employee_contracts_dates ON public.employee_contracts USING btree (start_date, end_date);

CREATE INDEX idx_employee_contracts_employment_id ON public.employee_contracts USING btree (employment_id);

CREATE INDEX idx_employee_contracts_status ON public.employee_contracts USING btree (status);

CREATE INDEX idx_employee_contracts_workspace_id ON public.employee_contracts USING btree (workspace_id);

CREATE INDEX idx_employee_documents_employee_id ON public.employee_documents USING btree (employee_id);

CREATE INDEX idx_employee_documents_expires_at ON public.employee_documents USING btree (expires_at) WHERE ((expires_at IS NOT NULL) AND (is_active = true));

CREATE INDEX idx_employee_documents_type ON public.employee_documents USING btree (document_type);

CREATE INDEX idx_employee_documents_workspace_id ON public.employee_documents USING btree (workspace_id);

CREATE INDEX idx_employee_emergency_contacts_employee_id ON public.employee_emergency_contacts USING btree (employee_id);

CREATE UNIQUE INDEX idx_employee_emergency_contacts_primary ON public.employee_emergency_contacts USING btree (employee_id) WHERE (is_primary = true);

CREATE INDEX idx_employee_phases_active_lookup ON public.employee_phases USING btree (employee_id, status, start_date, end_date) WHERE (status = 'active'::text);

CREATE INDEX idx_employee_phases_definition ON public.employee_phases USING btree (phase_definition_id);

CREATE INDEX idx_employee_phases_employee ON public.employee_phases USING btree (employee_id);

CREATE INDEX idx_employee_phases_status ON public.employee_phases USING btree (workspace_id, status);

CREATE INDEX idx_employee_phases_workspace ON public.employee_phases USING btree (workspace_id);

CREATE INDEX idx_employee_profiles_employee_id ON public.employee_profiles USING btree (employee_id);

CREATE INDEX idx_employee_salary_comp_def_id ON public.employee_salary_components USING btree (component_definition_id);

CREATE INDEX idx_employee_salary_components_effective_date ON public.employee_salary_components USING btree (effective_date);

CREATE INDEX idx_employee_salary_components_employee_id ON public.employee_salary_components USING btree (employee_id);

CREATE INDEX idx_employee_shifts_dates ON public.employee_shifts USING btree (start_date, end_date);

CREATE INDEX idx_employee_shifts_employee_id ON public.employee_shifts USING btree (employee_id);

CREATE INDEX idx_employee_shifts_shift_id ON public.employee_shifts USING btree (shift_id);

CREATE INDEX idx_employee_working_patterns_employee_date ON public.employee_working_patterns USING btree (employee_id, start_date);

CREATE INDEX idx_employee_working_patterns_pattern_id ON public.employee_working_patterns USING btree (pattern_id);

CREATE INDEX idx_employee_working_patterns_workspace ON public.employee_working_patterns USING btree (workspace_id);

CREATE INDEX idx_employees_id_number_search ON public.employees USING gin (employee_id_number public.gin_trgm_ops);

CREATE INDEX idx_employees_name_search ON public.employees USING gin (full_name public.gin_trgm_ops);

CREATE INDEX idx_employees_recruitment_source ON public.employees USING btree (recruitment_source_id);

CREATE INDEX idx_employees_tags ON public.employees USING gin (tags);

CREATE INDEX idx_employees_user_id ON public.employees USING btree (user_id);

CREATE UNIQUE INDEX idx_employees_workspace_employee_id ON public.employees USING btree (workspace_id, employee_id_number) WHERE (deleted_at IS NULL);

CREATE INDEX idx_employees_workspace_id ON public.employees USING btree (workspace_id);

CREATE UNIQUE INDEX idx_employees_workspace_user_id ON public.employees USING btree (workspace_id, user_id) WHERE (user_id IS NOT NULL);

CREATE INDEX idx_employers_parent_id ON public.employers USING btree (parent_id);

CREATE INDEX idx_employers_workspace_id ON public.employers USING btree (workspace_id);

CREATE INDEX idx_employment_phase_defs_rule_config ON public.employment_phase_definitions USING gin (rule_config);

CREATE INDEX idx_employment_phase_defs_type ON public.employment_phase_definitions USING btree (workspace_id, phase_type);

CREATE INDEX idx_employment_phase_defs_workspace ON public.employment_phase_definitions USING btree (workspace_id);

CREATE INDEX idx_employments_date_range ON public.employments USING btree (start_date, end_date);

CREATE INDEX idx_employments_employee_id ON public.employments USING btree (employee_id);

CREATE INDEX idx_employments_employer_id ON public.employments USING btree (employer_id);

CREATE INDEX idx_employments_workspace_id ON public.employments USING btree (workspace_id);

CREATE UNIQUE INDEX idx_errors_fingerprint ON public.attendance_log_errors USING btree (workspace_id, fingerprint) WHERE (fingerprint IS NOT NULL);

CREATE INDEX idx_feature_flags_lookup ON public.feature_flags USING btree (workspace_id, flag_key);

CREATE INDEX idx_feature_flags_workspace ON public.feature_flags USING btree (workspace_id);

CREATE INDEX idx_holidays_lookup ON public.holidays USING btree (workspace_id, date);

CREATE INDEX idx_holidays_workspace_id ON public.holidays USING btree (workspace_id);

CREATE INDEX idx_holidays_workspace_location_date ON public.holidays USING btree (workspace_id, location_id, date);

CREATE INDEX idx_incentive_tiers_policy_id ON public.payroll_incentive_tiers USING btree (policy_id);

CREATE INDEX idx_incentive_tiers_policy_priority ON public.payroll_incentive_tiers USING btree (policy_id, priority);

CREATE INDEX idx_incentive_tiers_rule_config ON public.payroll_incentive_tiers USING gin (rule_config);

CREATE INDEX idx_leave_accrual_logs_leave_type_id ON public.leave_accrual_logs USING btree (leave_type_id);

CREATE INDEX idx_leave_accrual_logs_policy_assignment_id ON public.leave_accrual_logs USING btree (policy_assignment_id);

CREATE INDEX idx_leave_accrual_logs_workspace ON public.leave_accrual_logs USING btree (workspace_id);

CREATE INDEX idx_leave_balances_employment_id ON public.leave_balances USING btree (employment_id);

CREATE INDEX idx_leave_balances_leave_type_id ON public.leave_balances USING btree (leave_type_id);

CREATE INDEX idx_leave_policy_assignments_assignee ON public.leave_policy_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_leave_policy_assignments_policy_id ON public.leave_policy_assignments USING btree (policy_id);

CREATE INDEX idx_leave_policy_assignments_workspace ON public.leave_policy_assignments USING btree (workspace_id);

CREATE INDEX idx_leave_policy_definitions_workspace ON public.leave_policy_definitions USING btree (workspace_id);

CREATE UNIQUE INDEX idx_leave_policy_definitions_ws_name ON public.leave_policy_definitions USING btree (workspace_id, name);

CREATE INDEX idx_leave_policy_defs_rule_config ON public.leave_policy_definitions USING gin (rule_config);

CREATE INDEX idx_leave_requests_approved_by ON public.leave_requests USING btree (approved_by_user_id);

CREATE INDEX idx_leave_requests_date_range ON public.leave_requests USING btree (start_date, end_date);

CREATE INDEX idx_leave_requests_employment_id ON public.leave_requests USING btree (employment_id);

CREATE INDEX idx_leave_requests_leave_type_id ON public.leave_requests USING btree (leave_type_id);

CREATE INDEX idx_leave_requests_requested_by ON public.leave_requests USING btree (requested_by_user_id);

CREATE INDEX idx_leave_requests_status ON public.leave_requests USING btree (status);

CREATE INDEX idx_leave_types_policy_config ON public.leave_types USING gin (policy_config);

CREATE INDEX idx_leave_types_workspace_id ON public.leave_types USING btree (workspace_id);

CREATE INDEX idx_otr_dept ON public.overtime_requests USING btree (department_id);

CREATE INDEX idx_otr_requested_by ON public.overtime_requests USING btree (requested_by_user_id);

CREATE INDEX idx_otr_status ON public.overtime_requests USING btree (workspace_id, status);

CREATE INDEX idx_otr_workspace_date ON public.overtime_requests USING btree (workspace_id, planned_date);

CREATE INDEX idx_otre_consent ON public.overtime_request_employees USING btree (consent_status);

CREATE INDEX idx_otre_request ON public.overtime_request_employees USING btree (overtime_request_id);

CREATE INDEX idx_outbox_jobs_created_at ON public.outbox_jobs USING btree (created_at);

CREATE INDEX idx_outbox_jobs_workspace_status ON public.outbox_jobs USING btree (workspace_id, status);

CREATE INDEX idx_overtime_policy_assignments_assignee ON public.overtime_policy_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_overtime_policy_assignments_policy_id ON public.overtime_policy_assignments USING btree (policy_id);

CREATE INDEX idx_overtime_policy_assignments_scope ON public.overtime_policy_assignments USING btree (workspace_id, assignee_type, assignee_id);

CREATE INDEX idx_overtime_policy_definitions_workspace ON public.overtime_policy_definitions USING btree (workspace_id);

CREATE INDEX idx_overtime_request_employees_employment_id ON public.overtime_request_employees USING btree (employment_id);

CREATE INDEX idx_overtime_rules_policy_id ON public.overtime_rules USING btree (policy_id);

CREATE INDEX idx_payroll_incentive_assignments_assignee ON public.payroll_incentive_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_payroll_incentive_assignments_lookup ON public.payroll_incentive_assignments USING btree (workspace_id, assignee_type, assignee_id);

CREATE INDEX idx_payroll_incentive_policies_deleted_at ON public.payroll_incentive_policies USING btree (workspace_id, deleted_at) WHERE (deleted_at IS NULL);

CREATE INDEX idx_payroll_incentive_policies_temporal ON public.payroll_incentive_policies USING btree (workspace_id, effective_from, effective_to);

CREATE INDEX idx_payroll_incentive_policies_workspace ON public.payroll_incentive_policies USING btree (workspace_id);

CREATE INDEX idx_payroll_rule_assignments_assignee ON public.payroll_rule_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_payroll_rule_assignments_rule ON public.payroll_rule_assignments USING btree (rule_definition_id);

CREATE INDEX idx_payroll_rule_assignments_scope ON public.payroll_rule_assignments USING btree (workspace_id, assignee_type, assignee_id);

CREATE INDEX idx_payroll_rule_assignments_temporal ON public.payroll_rule_assignments USING btree (workspace_id, effective_from, effective_to);

CREATE INDEX idx_payroll_rule_assignments_workspace ON public.payroll_rule_assignments USING btree (workspace_id);

CREATE INDEX idx_payroll_rule_defs_category ON public.payroll_rule_definitions USING btree (workspace_id, category);

CREATE INDEX idx_payroll_rule_defs_rule_config ON public.payroll_rule_definitions USING gin (rule_config);

CREATE INDEX idx_payroll_rule_defs_workspace ON public.payroll_rule_definitions USING btree (workspace_id);

CREATE INDEX idx_payroll_rule_tiers_definition ON public.payroll_rule_tiers USING btree (rule_definition_id);

CREATE INDEX idx_payslip_incentives_payslip_id ON public.payslip_incentives USING btree (payslip_id);

CREATE INDEX idx_payslip_rule_evaluations_payslip_id ON public.payslip_rule_evaluations USING btree (payslip_id);

CREATE INDEX idx_payslip_segments_department_id ON public.payslip_segments USING btree (department_id);

CREATE INDEX idx_payslip_segments_payslip_id ON public.payslip_segments USING btree (payslip_id);

CREATE INDEX idx_payslips_employment_id ON public.payslips USING btree (employment_id);

CREATE INDEX idx_payslips_period ON public.payslips USING btree (period_start, period_end);

CREATE INDEX idx_payslips_status ON public.payslips USING btree (status);

CREATE INDEX idx_payslips_workspace_id ON public.payslips USING btree (workspace_id);

CREATE INDEX idx_policy_templates_category ON public.policy_templates USING btree (category);

CREATE INDEX idx_policy_templates_country ON public.policy_templates USING btree (country_code) WHERE (country_code IS NOT NULL);

CREATE UNIQUE INDEX idx_policy_templates_system_unique ON public.policy_templates USING btree (name, version) WHERE (workspace_id IS NULL);

CREATE UNIQUE INDEX idx_policy_templates_workspace_unique ON public.policy_templates USING btree (workspace_id, name) WHERE (workspace_id IS NOT NULL);

CREATE INDEX idx_raw_attendance_logs_employment_id ON public.raw_attendance_logs USING btree (employment_id);

CREATE INDEX idx_raw_logs_punch_time ON public.raw_attendance_logs USING btree (punch_time);

CREATE INDEX idx_raw_logs_workspace_id ON public.raw_attendance_logs USING btree (workspace_id);

CREATE INDEX idx_recruitment_sources_workspace ON public.recruitment_sources USING btree (workspace_id);

CREATE UNIQUE INDEX idx_salary_base_groups_global_slug ON public.salary_base_groups USING btree (slug) WHERE (workspace_id IS NULL);

CREATE UNIQUE INDEX idx_salary_base_groups_workspace_slug ON public.salary_base_groups USING btree (workspace_id, slug) WHERE (workspace_id IS NOT NULL);

CREATE INDEX idx_salary_comp_defs_linked_rule ON public.salary_component_definitions USING btree (linked_rule_id) WHERE (linked_rule_id IS NOT NULL);

CREATE INDEX idx_salary_comp_defs_workspace ON public.salary_component_definitions USING btree (workspace_id);

CREATE INDEX idx_shift_rosters_employment_id ON public.shift_rosters USING btree (employment_id);

CREATE INDEX idx_shift_rosters_shift_id ON public.shift_rosters USING btree (shift_id);

CREATE INDEX idx_shift_swap_date ON public.shift_swap_requests USING btree (swap_date);

CREATE INDEX idx_shift_swap_status ON public.shift_swap_requests USING btree (workspace_id, status);

CREATE INDEX idx_shift_swap_workspace ON public.shift_swap_requests USING btree (workspace_id);

CREATE INDEX idx_shift_swap_ws_requester ON public.shift_swap_requests USING btree (workspace_id, requester_id);

CREATE INDEX idx_shift_swap_ws_target ON public.shift_swap_requests USING btree (workspace_id, target_id);

CREATE INDEX idx_shifts_active ON public.shifts USING btree (is_active);

CREATE INDEX idx_shifts_employer_id ON public.shifts USING btree (employer_id);

CREATE INDEX idx_shifts_location_id ON public.shifts USING btree (location_id);

CREATE INDEX idx_shifts_parent_id ON public.shifts USING btree (parent_shift_id);

CREATE INDEX idx_shifts_workspace_id ON public.shifts USING btree (workspace_id);

CREATE INDEX idx_time_entries_employment_id ON public.time_entries USING btree (employment_id);

CREATE INDEX idx_time_entries_punch_in_time ON public.time_entries USING btree (punch_in_time);

CREATE INDEX idx_time_entries_workspace_id ON public.time_entries USING btree (workspace_id);

CREATE UNIQUE INDEX idx_uq_shift_swap_pair_date ON public.shift_swap_requests USING btree (workspace_id, requester_id, target_id, swap_date) WHERE ((status)::text <> ALL ((ARRAY['cancelled'::character varying, 'rejected'::character varying])::text[]));

CREATE INDEX idx_wce_active ON public.workspace_config_entries USING btree (workspace_id, config_key) WHERE (effective_to IS NULL);

CREATE INDEX idx_wce_lookup ON public.workspace_config_entries USING btree (workspace_id, config_key, effective_from);

CREATE INDEX idx_wce_scope ON public.workspace_config_entries USING btree (workspace_id, scope_type, scope_id);

CREATE UNIQUE INDEX idx_work_locations_hq ON public.work_locations USING btree (workspace_id) WHERE (is_headquarters = true);

CREATE INDEX idx_working_conditions_employment_id ON public.working_conditions USING btree (employment_id);

CREATE INDEX idx_working_pattern_details_shift ON public.working_pattern_details USING btree (shift_id);

CREATE INDEX idx_working_patterns_workspace ON public.working_patterns USING btree (workspace_id);

CREATE UNIQUE INDEX uq_departments_name ON public.departments USING btree (workspace_id, lower((name)::text)) WHERE (deleted_at IS NULL);

CREATE UNIQUE INDEX uq_holidays_workspace_date ON public.holidays USING btree (workspace_id, date, name) WHERE (deleted_at IS NULL);

CREATE UNIQUE INDEX uq_leave_policy_definitions_name ON public.leave_policy_definitions USING btree (workspace_id, lower(name)) WHERE (deleted_at IS NULL);

CREATE UNIQUE INDEX uq_leave_types_code ON public.leave_types USING btree (workspace_id, lower((code)::text)) WHERE (deleted_at IS NULL);

CREATE UNIQUE INDEX uq_working_patterns_name ON public.working_patterns USING btree (workspace_id, lower(name)) WHERE (deleted_at IS NULL);

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_default_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_default_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_default_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m01_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m01_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m01_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m02_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m02_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m02_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m03_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m03_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m03_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m04_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m04_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m04_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m05_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m05_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m05_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m06_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m06_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m06_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m07_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m07_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m07_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m08_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m08_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m08_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m09_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m09_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m09_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m10_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m10_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m10_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m11_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m11_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m11_workspace_id_status_idx;

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m12_fingerprint_idx;

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m12_pkey;

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m12_workspace_id_status_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_default_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_default_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_default_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_default_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m01_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m01_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m01_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m01_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m02_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m02_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m02_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m02_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m03_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m03_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m03_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m03_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m04_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m04_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m04_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m04_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m05_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m05_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m05_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m05_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m06_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m06_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m06_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m06_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m07_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m07_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m07_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m07_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m08_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m08_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m08_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m08_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m09_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m09_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m09_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m09_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m10_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m10_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m10_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m10_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m11_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m11_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m11_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m11_workspace_id_created_at_idx;

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m12_action_idx;

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m12_pkey;

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m12_target_user_id_idx;

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m12_workspace_id_created_at_idx;

CREATE TRIGGER trg_cascade_delete_dept_assignments BEFORE DELETE ON public.departments FOR EACH ROW EXECUTE FUNCTION public.fn_cascade_delete_department_assignments();

CREATE TRIGGER trg_cascade_delete_emp_assignments BEFORE DELETE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.fn_cascade_delete_employee_assignments();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.assignment_rules FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_adjustment_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_daily_summaries FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_import_jobs FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_policies FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.dashboard_preferences FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.departments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_phases FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_profiles FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_salary_components FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_shifts FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_working_patterns FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employment_phase_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.feature_flags FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.holidays FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_balances FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_policy_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.overtime_policy_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.overtime_rules FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.payroll_rule_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.payslips FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.policy_templates FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.recruitment_sources FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.salary_base_groups FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.salary_component_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_rosters FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_swap_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shifts FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.working_patterns FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.leave_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.overtime_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.payroll_incentive_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.payroll_rule_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

CREATE TRIGGER trg_validate_target_fk BEFORE INSERT OR UPDATE OF target_type, target_id ON public.attendance_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_target_fk();

CREATE TRIGGER trigger_outbox_jobs_updated_at BEFORE UPDATE ON public.outbox_jobs FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

ALTER TABLE ONLY public.approval_workflow_instances
    ADD CONSTRAINT approval_workflow_instances_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.approval_workflows(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.approval_workflow_steps
    ADD CONSTRAINT approval_workflow_steps_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.approval_workflows(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.approval_workflow_tasks
    ADD CONSTRAINT approval_workflow_tasks_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.approval_workflow_instances(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.assignment_rules
    ADD CONSTRAINT assignment_rules_pattern_id_fkey FOREIGN KEY (pattern_id) REFERENCES public.working_patterns(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_time_entry_id_fkey FOREIGN KEY (time_entry_id) REFERENCES public.time_entries(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.attendance_policy_assignments
    ADD CONSTRAINT attendance_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.attendance_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_policy_versions
    ADD CONSTRAINT attendance_policy_versions_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.attendance_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_manager_id_fkey FOREIGN KEY (manager_id) REFERENCES public.employees(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_home_location_id_fkey FOREIGN KEY (home_location_id) REFERENCES public.work_locations(id);

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_host_location_id_fkey FOREIGN KEY (host_location_id) REFERENCES public.work_locations(id);

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.employee_documents(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_department_history
    ADD CONSTRAINT employee_department_history_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_department_history
    ADD CONSTRAINT employee_department_history_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_emergency_contacts
    ADD CONSTRAINT employee_emergency_contacts_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_phase_definition_id_fkey FOREIGN KEY (phase_definition_id) REFERENCES public.employment_phase_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_salary_components
    ADD CONSTRAINT employee_salary_components_component_definition_id_fkey FOREIGN KEY (component_definition_id) REFERENCES public.salary_component_definitions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_salary_components
    ADD CONSTRAINT employee_salary_components_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_shifts
    ADD CONSTRAINT employee_shifts_new_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_shifts
    ADD CONSTRAINT employee_shifts_new_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_working_patterns
    ADD CONSTRAINT employee_working_patterns_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_working_patterns
    ADD CONSTRAINT employee_working_patterns_pattern_id_fkey FOREIGN KEY (pattern_id) REFERENCES public.working_patterns(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_recruitment_source_id_fkey FOREIGN KEY (recruitment_source_id) REFERENCES public.recruitment_sources(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employers
    ADD CONSTRAINT employers_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.employers(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_manager_employment_id_fkey FOREIGN KEY (manager_employment_id) REFERENCES public.employments(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_primary_location_id_fkey FOREIGN KEY (primary_location_id) REFERENCES public.work_locations(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.holidays
    ADD CONSTRAINT holidays_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.holidays
    ADD CONSTRAINT holidays_location_id_fkey FOREIGN KEY (location_id) REFERENCES public.work_locations(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_policy_assignment_id_fkey FOREIGN KEY (policy_assignment_id) REFERENCES public.leave_policy_assignments(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.leave_policy_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_policy_definitions
    ADD CONSTRAINT leave_policy_definitions_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT ot_claim_behalf_assignments_target_department_id_fkey FOREIGN KEY (target_department_id) REFERENCES public.departments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT ot_claim_behalf_assignments_target_employee_id_fkey FOREIGN KEY (target_employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.overtime_policy_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_matched_summary_id_fkey FOREIGN KEY (matched_summary_id) REFERENCES public.attendance_daily_summaries(id);

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id);

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.approval_workflows(id);

ALTER TABLE ONLY public.overtime_rules
    ADD CONSTRAINT overtime_rules_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.overtime_policy_definitions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_incentive_assignments
    ADD CONSTRAINT payroll_incentive_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_incentive_tiers
    ADD CONSTRAINT payroll_incentive_tiers_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_rule_definition_id_fkey FOREIGN KEY (rule_definition_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.payroll_rule_tiers
    ADD CONSTRAINT payroll_rule_tiers_rule_definition_id_fkey FOREIGN KEY (rule_definition_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslip_incentives
    ADD CONSTRAINT payslip_incentives_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslip_rule_evaluations
    ADD CONSTRAINT payslip_rule_evaluations_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslip_segments
    ADD CONSTRAINT payslip_segments_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id);

ALTER TABLE ONLY public.payslip_segments
    ADD CONSTRAINT payslip_segments_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslips
    ADD CONSTRAINT payslips_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT raw_attendance_logs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.recruitment_sources
    ADD CONSTRAINT recruitment_sources_referrer_employee_id_fkey FOREIGN KEY (referrer_employee_id) REFERENCES public.employees(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.salary_component_base_members
    ADD CONSTRAINT salary_component_base_members_base_group_id_fkey FOREIGN KEY (base_group_id) REFERENCES public.salary_base_groups(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.salary_component_base_members
    ADD CONSTRAINT salary_component_base_members_component_definition_id_fkey FOREIGN KEY (component_definition_id) REFERENCES public.salary_component_definitions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_linked_rule_id_fkey FOREIGN KEY (linked_rule_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shift_rosters
    ADD CONSTRAINT shift_rosters_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.shift_rosters
    ADD CONSTRAINT shift_rosters_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requester_id_fkey FOREIGN KEY (requester_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requester_shift_id_fkey FOREIGN KEY (requester_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_target_id_fkey FOREIGN KEY (target_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_target_shift_id_fkey FOREIGN KEY (target_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shifts
    ADD CONSTRAINT shifts_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shifts
    ADD CONSTRAINT shifts_location_id_fkey FOREIGN KEY (location_id) REFERENCES public.work_locations(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shifts
    ADD CONSTRAINT shifts_parent_shift_id_fkey FOREIGN KEY (parent_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id);

ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.working_pattern_details
    ADD CONSTRAINT working_pattern_details_pattern_id_fkey FOREIGN KEY (pattern_id) REFERENCES public.working_patterns(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.working_pattern_details
    ADD CONSTRAINT working_pattern_details_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

-- ==========================================
-- Tenant Seed Data (Inserted during squash initialization)
-- ==========================================

INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system)
VALUES 
  ('b0000000-0000-0000-0000-000000000001', NULL, 'Gross Salary', 'gross', 'Base for all basic, taxable, and insurable income components', TRUE),
  ('b0000000-0000-0000-0000-000000000002', NULL, 'Net Salary', 'net', 'Final take-home pay after all deductions', TRUE),
  ('b0000000-0000-0000-0000-000000000003', NULL, 'Social Insurance Base', 'bhxh', 'Base for calculating social insurance, health insurance, and unemployment insurance contributions', TRUE),
  ('b0000000-0000-0000-0000-000000000004', NULL, 'Taxable Income', 'taxable', 'Base for calculating personal income tax (PIT)', TRUE)
ON CONFLICT (slug) WHERE workspace_id IS NULL DO NOTHING;
