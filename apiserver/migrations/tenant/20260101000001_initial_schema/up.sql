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

CREATE FUNCTION public.update_employee_profiles_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();

RETURN NEW;

END;

$$;

CREATE TABLE public.assignment_rules (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    pattern_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    conditions jsonb NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public.attendance_adjustment_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    time_entry_id uuid,
    requested_punch_in_time timestamp with time zone NOT NULL,
    requested_punch_out_time timestamp with time zone,
    reason text,
    status public.adjustment_status_type DEFAULT 'pending'::public.adjustment_status_type NOT NULL,
    requested_by_user_id uuid,
    approved_by_user_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    is_break_overtime boolean DEFAULT false NOT NULL
);

CREATE TABLE public.attendance_daily_summaries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    employee_id uuid NOT NULL,
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
    holiday_name character varying(255)
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
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    device_id text,
    raw_user_id text NOT NULL,
    punch_time timestamp with time zone NOT NULL,
    raw_data jsonb,
    fingerprint text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now()
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

CREATE TABLE public.department_default_shifts (
    department_id uuid NOT NULL,
    shift_id uuid NOT NULL
);

CREATE TABLE public.department_payroll_configs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    department_id uuid NOT NULL,
    std_working_days numeric(5,2) NOT NULL,
    effective_date date NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    hours_per_day numeric(5,2) NOT NULL
);

CREATE TABLE public.departments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    manager_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    default_working_pattern_id uuid,
    std_working_days numeric(5,2)
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

CREATE TABLE public.employee_payroll_configs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    std_working_days numeric(5,2) NOT NULL,
    effective_date date NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    hours_per_day numeric(5,2) NOT NULL
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
    national_id character varying(50),
    bank_name character varying(100),
    bank_account_number character varying(50),
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
    CONSTRAINT employee_working_patterns_check CHECK (((end_date IS NULL) OR (end_date >= start_date)))
);

CREATE TABLE public.employees (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    user_id uuid,
    employee_id_number character varying(50) NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    tags TEXT[] DEFAULT '{}',
    full_name character varying(255) NOT NULL,
    recruitment_source_id uuid,
    CONSTRAINT chk_employee_full_name CHECK ((length((full_name)::text) > 0))
);

CREATE TABLE public.employment_periods (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    hire_date date NOT NULL,
    termination_date date,
    is_rehire boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
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
    is_credit_on_work boolean DEFAULT true NOT NULL
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
    employee_id uuid NOT NULL,
    year integer NOT NULL,
    entitlement integer DEFAULT 12 NOT NULL,
    carried_over integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    accrued_amount numeric(5,2) DEFAULT 0.00 NOT NULL,
    leave_type_id uuid NOT NULL
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
    leave_type_id uuid NOT NULL
);

CREATE TABLE public.leave_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
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
    CONSTRAINT chk_start_before_end CHECK ((start_date <= end_date))
);

CREATE TABLE public.leave_types (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name text NOT NULL,
    is_paid boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    policy_config jsonb DEFAULT '{}'::jsonb NOT NULL
);

CREATE TABLE public.member_extra_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    assigned_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    expires_at timestamp with time zone NOT NULL,
    role_id uuid NOT NULL
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

CREATE TABLE public.payroll_incentive_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    base_amount double precision NOT NULL,
    payout_type text DEFAULT 'fixed'::text NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.payroll_incentive_tiers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_id uuid NOT NULL,
    priority integer NOT NULL,
    max_late_early_count integer,
    max_unpaid_leave_days double precision,
    max_sick_leave_days double precision,
    max_total_leave_days double precision,
    payout_amount double precision NOT NULL
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

CREATE TABLE public.payroll_settings (
    workspace_id uuid NOT NULL,
    std_working_day_mode text DEFAULT 'fixed'::text NOT NULL,
    fixed_std_days numeric(5,2) DEFAULT 26.0 NOT NULL,
    working_days_per_week integer DEFAULT 6 NOT NULL,
    hours_per_day numeric(4,2) DEFAULT 8.0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    effective_date date DEFAULT '2026-01-01'::date NOT NULL,
    max_ot_hours_per_day numeric(4,1) DEFAULT 4.0,
    max_ot_hours_per_month numeric(5,1) DEFAULT 40.0
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
    employee_id uuid NOT NULL,
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
    workspace_id uuid NOT NULL
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
    employee_id uuid NOT NULL,
    punch_time timestamp with time zone NOT NULL,
    source public.log_source_type NOT NULL,
    device_id character varying(50),
    raw_data jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    punch integer DEFAULT '-1'::integer NOT NULL,
    status integer DEFAULT '-1'::integer NOT NULL
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
    is_base_for_ot boolean DEFAULT false NOT NULL,
    is_taxable boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    component_type text DEFAULT 'allowance'::text NOT NULL,
    is_insurance_base boolean DEFAULT false NOT NULL,
    linked_rule_id uuid
);

CREATE TABLE public.shift_rosters (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    shift_id uuid,
    date date NOT NULL,
    status character varying(20) DEFAULT 'confirmed'::character varying,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
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
    report_symbol character varying(10)
);

CREATE TABLE public.time_entries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
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
    workspace_id uuid NOT NULL
);

CREATE TABLE public.working_conditions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    type character varying(50) NOT NULL,
    minute_allowance integer NOT NULL,
    is_paid boolean DEFAULT false NOT NULL,
    start_date date NOT NULL,
    end_date date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public.working_pattern_details (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    pattern_id uuid NOT NULL,
    day_of_week integer NOT NULL,
    shift_id uuid,
    CONSTRAINT working_pattern_details_day_of_week_check CHECK (((day_of_week >= 0) AND (day_of_week <= 6)))
);

CREATE TABLE public.working_patterns (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    description text,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.workspace_member_extra_permissions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    permission text NOT NULL,
    assigned_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    expires_at timestamp with time zone NOT NULL,
    assigned_by uuid
);

CREATE TABLE public.workspace_members (
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    role_id uuid NOT NULL
);

CREATE TABLE public.workspace_role_permissions (
    role_id uuid NOT NULL,
    permission_name text NOT NULL
);

CREATE TABLE public.workspace_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid,
    name text NOT NULL,
    description text,
    color text DEFAULT 'slate'::text,
    is_system boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    scope_type character varying(16) DEFAULT 'scoped'::character varying NOT NULL,
    CONSTRAINT workspace_roles_scope_type_check CHECK (((scope_type)::text = ANY ((ARRAY['global'::character varying, 'scoped'::character varying])::text[])))
);

ALTER TABLE ONLY public.assignment_rules
    ADD CONSTRAINT assignment_rules_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_employee_id_date_key UNIQUE (employee_id, date);

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
    ADD CONSTRAINT attendance_staging_logs_pkey PRIMARY KEY (id);





ALTER TABLE ONLY public.dashboard_preferences
    ADD CONSTRAINT dashboard_preferences_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.dashboard_preferences
    ADD CONSTRAINT dashboard_preferences_user_id_workspace_id_key UNIQUE (user_id, workspace_id);

ALTER TABLE ONLY public.department_default_shifts
    ADD CONSTRAINT department_default_shifts_pkey PRIMARY KEY (department_id, shift_id);

ALTER TABLE ONLY public.department_payroll_configs
    ADD CONSTRAINT department_payroll_configs_department_id_effective_date_key UNIQUE (department_id, effective_date);

ALTER TABLE ONLY public.department_payroll_configs
    ADD CONSTRAINT department_payroll_configs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_department_history
    ADD CONSTRAINT employee_department_history_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_emergency_contacts
    ADD CONSTRAINT employee_emergency_contacts_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_payroll_configs
    ADD CONSTRAINT employee_payroll_configs_employee_id_effective_date_key UNIQUE (employee_id, effective_date);

ALTER TABLE ONLY public.employee_payroll_configs
    ADD CONSTRAINT employee_payroll_configs_pkey PRIMARY KEY (id);

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

ALTER TABLE ONLY public.employment_periods
    ADD CONSTRAINT employment_periods_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_workspace_id_name_key UNIQUE (workspace_id, name);

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
    ADD CONSTRAINT leave_balances_employee_leave_type_id_year_key UNIQUE (employee_id, leave_type_id, year);

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

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.attendance_policy_assignments
    ADD CONSTRAINT no_overlapping_policy_assignments EXCLUDE USING gist (target_id WITH =, target_type WITH =, tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamp with time zone)) WITH &&);

ALTER TABLE ONLY public.attendance_policy_versions
    ADD CONSTRAINT no_overlapping_policy_versions EXCLUDE USING gist (policy_id WITH =, tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamp with time zone)) WITH &&);

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_workspace_id_assignee_type_coa_excl EXCLUDE USING gist (workspace_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&);

ALTER TABLE ONLY public.overtime_policy_definitions
    ADD CONSTRAINT overtime_policy_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_rules
    ADD CONSTRAINT overtime_rules_pkey PRIMARY KEY (id);

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

ALTER TABLE ONLY public.payroll_settings
    ADD CONSTRAINT payroll_settings_pkey PRIMARY KEY (workspace_id, effective_date);

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



ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_token_key UNIQUE (token);

ALTER TABLE ONLY public.shift_rosters
    ADD CONSTRAINT shift_rosters_employee_id_date_key UNIQUE (employee_id, date);

ALTER TABLE ONLY public.shift_rosters
    ADD CONSTRAINT shift_rosters_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.shifts
    ADD CONSTRAINT shifts_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.system_roles
    ADD CONSTRAINT system_roles_name_key UNIQUE (name);



ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT unique_raw_log UNIQUE (workspace_id, employee_id, punch_time, punch, status);

ALTER TABLE ONLY public.employee_shifts
    ADD CONSTRAINT uq_employee_shift_start UNIQUE (employee_id, start_date);

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT uq_shift_swap_pair_date UNIQUE (workspace_id, requester_id, target_id, swap_date);







ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.working_pattern_details
    ADD CONSTRAINT working_pattern_details_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.working_patterns
    ADD CONSTRAINT working_patterns_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_permissions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_pkey PRIMARY KEY (workspace_id, user_id);

ALTER TABLE ONLY public.workspace_role_permissions
    ADD CONSTRAINT workspace_role_permissions_pkey PRIMARY KEY (role_id, permission_name);

ALTER TABLE ONLY public.workspace_roles
    ADD CONSTRAINT workspace_roles_pkey PRIMARY KEY (id);



CREATE UNIQUE INDEX attendance_policies_workspace_default_idx ON public.attendance_policies USING btree (workspace_id) WHERE (is_default = true);

CREATE INDEX idx_assignment_rules_pattern_id ON public.assignment_rules USING btree (pattern_id);

CREATE INDEX idx_assignment_rules_priority ON public.assignment_rules USING btree (priority DESC);

CREATE INDEX idx_assignment_rules_workspace_id ON public.assignment_rules USING btree (workspace_id);

CREATE INDEX idx_attendance_adj_time_entry_id ON public.attendance_adjustment_requests USING btree (time_entry_id);

CREATE INDEX idx_attendance_adjustment_requests_employee_id ON public.attendance_adjustment_requests USING btree (employee_id);

CREATE INDEX idx_attendance_adjustment_requests_status ON public.attendance_adjustment_requests USING btree (status);

CREATE INDEX idx_attendance_import_jobs_status ON public.attendance_import_jobs USING btree (status);

CREATE INDEX idx_attendance_import_jobs_workspace_id ON public.attendance_import_jobs USING btree (workspace_id);

CREATE INDEX idx_audit_logs_actor_id ON public.audit_logs USING btree (actor_id);

CREATE INDEX idx_audit_logs_created_at ON public.audit_logs USING btree (created_at);

CREATE INDEX idx_audit_logs_target_user_id ON public.audit_logs USING btree (target_user_id);

CREATE INDEX idx_audit_logs_workspace_id ON public.audit_logs USING btree (workspace_id);

CREATE INDEX idx_authorized_agents_trusted ON public.authorized_agents USING btree (is_trusted);

CREATE INDEX idx_authorized_agents_workspace ON public.authorized_agents USING btree (workspace_id);

CREATE INDEX idx_daily_summaries_status ON public.attendance_daily_summaries USING btree (ot_status);

CREATE INDEX idx_daily_summaries_workspace_date ON public.attendance_daily_summaries USING btree (workspace_id, date);

CREATE INDEX idx_daily_summaries_ws_emp_date ON public.attendance_daily_summaries USING btree (workspace_id, employee_id, date);

CREATE INDEX idx_dashboard_prefs_user_ws ON public.dashboard_preferences USING btree (user_id, workspace_id);

CREATE INDEX idx_departments_manager_id ON public.departments USING btree (manager_id);

CREATE INDEX idx_departments_workspace_id ON public.departments USING btree (workspace_id);

CREATE INDEX idx_dept_payroll_configs_department_id ON public.department_payroll_configs USING btree (department_id);

CREATE INDEX idx_devices_agent ON public.devices USING btree (agent_id);

CREATE INDEX idx_devices_workspace ON public.devices USING btree (workspace_id);

CREATE INDEX idx_emp_dept_hist_department_id ON public.employee_department_history USING btree (department_id);

CREATE INDEX idx_emp_dept_hist_emp_date ON public.employee_department_history USING btree (employee_id, effective_date DESC);

CREATE INDEX idx_emp_dept_hist_employee_id ON public.employee_department_history USING btree (employee_id);

CREATE INDEX idx_emp_payroll_configs_employee_id ON public.employee_payroll_configs USING btree (employee_id);

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

CREATE INDEX idx_employees_workspace_active ON public.employees USING btree (workspace_id, is_active);

CREATE UNIQUE INDEX idx_employees_workspace_employee_id ON public.employees USING btree (workspace_id, employee_id_number);

CREATE INDEX idx_employees_workspace_id ON public.employees USING btree (workspace_id);

CREATE INDEX idx_employment_periods_employee_id ON public.employment_periods USING btree (employee_id);

CREATE INDEX idx_employment_phase_defs_type ON public.employment_phase_definitions USING btree (workspace_id, phase_type);

CREATE INDEX idx_employment_phase_defs_workspace ON public.employment_phase_definitions USING btree (workspace_id);

CREATE UNIQUE INDEX idx_errors_fingerprint ON public.attendance_log_errors USING btree (workspace_id, fingerprint) WHERE (fingerprint IS NOT NULL);

CREATE INDEX idx_extra_perms_lookup ON public.workspace_member_extra_permissions USING btree (workspace_id, user_id, expires_at);

CREATE INDEX idx_feature_flags_lookup ON public.feature_flags USING btree (workspace_id, flag_key);

CREATE INDEX idx_feature_flags_workspace ON public.feature_flags USING btree (workspace_id);

CREATE INDEX idx_holidays_lookup ON public.holidays USING btree (workspace_id, date);

CREATE INDEX idx_holidays_workspace_id ON public.holidays USING btree (workspace_id);

CREATE INDEX idx_incentive_tiers_policy_id ON public.payroll_incentive_tiers USING btree (policy_id);

CREATE INDEX idx_incentive_tiers_policy_priority ON public.payroll_incentive_tiers USING btree (policy_id, priority);

CREATE INDEX idx_leave_accrual_logs_leave_type_id ON public.leave_accrual_logs USING btree (leave_type_id);

CREATE INDEX idx_leave_accrual_logs_policy_assignment_id ON public.leave_accrual_logs USING btree (policy_assignment_id);

CREATE INDEX idx_leave_accrual_logs_workspace ON public.leave_accrual_logs USING btree (workspace_id);

CREATE INDEX idx_leave_balances_emp_leave_type ON public.leave_balances USING btree (employee_id, leave_type_id);

CREATE INDEX idx_leave_balances_employee_id ON public.leave_balances USING btree (employee_id);

CREATE INDEX idx_leave_balances_leave_type_id ON public.leave_balances USING btree (leave_type_id);

CREATE INDEX idx_leave_policy_assignments_policy_id ON public.leave_policy_assignments USING btree (policy_id);

CREATE INDEX idx_leave_policy_assignments_workspace ON public.leave_policy_assignments USING btree (workspace_id);

CREATE INDEX idx_leave_policy_definitions_workspace ON public.leave_policy_definitions USING btree (workspace_id);

CREATE UNIQUE INDEX idx_leave_policy_definitions_ws_name ON public.leave_policy_definitions USING btree (workspace_id, name);

CREATE INDEX idx_leave_requests_approved_by ON public.leave_requests USING btree (approved_by_user_id);

CREATE INDEX idx_leave_requests_date_range ON public.leave_requests USING btree (start_date, end_date);

CREATE INDEX idx_leave_requests_employee_id ON public.leave_requests USING btree (employee_id);

CREATE INDEX idx_leave_requests_leave_type_id ON public.leave_requests USING btree (leave_type_id);

CREATE INDEX idx_leave_requests_requested_by ON public.leave_requests USING btree (requested_by_user_id);

CREATE INDEX idx_leave_requests_status ON public.leave_requests USING btree (status);

CREATE INDEX idx_leave_types_workspace_id ON public.leave_types USING btree (workspace_id);

CREATE INDEX idx_member_extra_roles_lookup ON public.member_extra_roles USING btree (workspace_id, user_id, expires_at);

CREATE INDEX idx_overtime_policy_assignments_policy_id ON public.overtime_policy_assignments USING btree (policy_id);

CREATE INDEX idx_overtime_policy_assignments_scope ON public.overtime_policy_assignments USING btree (workspace_id, assignee_type, assignee_id);

CREATE INDEX idx_overtime_policy_definitions_workspace ON public.overtime_policy_definitions USING btree (workspace_id);

CREATE INDEX idx_overtime_rules_policy_id ON public.overtime_rules USING btree (policy_id);

CREATE INDEX idx_payroll_incentive_policies_workspace ON public.payroll_incentive_policies USING btree (workspace_id);

CREATE INDEX idx_payroll_rule_assignments_rule ON public.payroll_rule_assignments USING btree (rule_definition_id);

CREATE INDEX idx_payroll_rule_assignments_scope ON public.payroll_rule_assignments USING btree (workspace_id, assignee_type, assignee_id);

CREATE INDEX idx_payroll_rule_assignments_temporal ON public.payroll_rule_assignments USING btree (workspace_id, effective_from, effective_to);

CREATE INDEX idx_payroll_rule_assignments_workspace ON public.payroll_rule_assignments USING btree (workspace_id);

CREATE INDEX idx_payroll_rule_defs_category ON public.payroll_rule_definitions USING btree (workspace_id, category);

CREATE INDEX idx_payroll_rule_defs_workspace ON public.payroll_rule_definitions USING btree (workspace_id);

CREATE INDEX idx_payroll_rule_tiers_definition ON public.payroll_rule_tiers USING btree (rule_definition_id);

CREATE INDEX idx_payslip_incentives_payslip_id ON public.payslip_incentives USING btree (payslip_id);

CREATE INDEX idx_payslip_rule_evaluations_payslip_id ON public.payslip_rule_evaluations USING btree (payslip_id);

CREATE INDEX idx_payslip_segments_department_id ON public.payslip_segments USING btree (department_id);

CREATE INDEX idx_payslip_segments_payslip_id ON public.payslip_segments USING btree (payslip_id);

CREATE INDEX idx_payslips_employee_id ON public.payslips USING btree (employee_id);

CREATE INDEX idx_payslips_period ON public.payslips USING btree (period_start, period_end);

CREATE INDEX idx_payslips_status ON public.payslips USING btree (status);

CREATE INDEX idx_payslips_workspace_id ON public.payslips USING btree (workspace_id);

CREATE INDEX idx_policy_templates_category ON public.policy_templates USING btree (category);

CREATE INDEX idx_policy_templates_country ON public.policy_templates USING btree (country_code) WHERE (country_code IS NOT NULL);

CREATE UNIQUE INDEX idx_policy_templates_system_unique ON public.policy_templates USING btree (name, version) WHERE (workspace_id IS NULL);

CREATE UNIQUE INDEX idx_policy_templates_workspace_unique ON public.policy_templates USING btree (workspace_id, name) WHERE (workspace_id IS NOT NULL);

CREATE INDEX idx_raw_logs_employee_id ON public.raw_attendance_logs USING btree (employee_id);

CREATE INDEX idx_raw_logs_punch_time ON public.raw_attendance_logs USING btree (punch_time);

CREATE INDEX idx_raw_logs_workspace_id ON public.raw_attendance_logs USING btree (workspace_id);

CREATE INDEX idx_raw_logs_ws_emp_time ON public.raw_attendance_logs USING btree (workspace_id, employee_id, punch_time);

CREATE INDEX idx_recruitment_sources_workspace ON public.recruitment_sources USING btree (workspace_id);

CREATE UNIQUE INDEX idx_salary_base_groups_global_slug ON public.salary_base_groups USING btree (slug) WHERE (workspace_id IS NULL);

CREATE UNIQUE INDEX idx_salary_base_groups_workspace_slug ON public.salary_base_groups USING btree (workspace_id, slug) WHERE (workspace_id IS NOT NULL);

CREATE INDEX idx_salary_comp_defs_linked_rule ON public.salary_component_definitions USING btree (linked_rule_id) WHERE (linked_rule_id IS NOT NULL);

CREATE INDEX idx_salary_comp_defs_workspace ON public.salary_component_definitions USING btree (workspace_id);

CREATE INDEX idx_sessions_token ON public.sessions USING btree (token);

CREATE INDEX idx_sessions_user_id ON public.sessions USING btree (user_id);

CREATE INDEX idx_shift_rosters_employee_date ON public.shift_rosters USING btree (employee_id, date);

CREATE INDEX idx_shift_rosters_shift_id ON public.shift_rosters USING btree (shift_id);

CREATE INDEX idx_shift_swap_date ON public.shift_swap_requests USING btree (swap_date);

CREATE INDEX idx_shift_swap_status ON public.shift_swap_requests USING btree (workspace_id, status);

CREATE INDEX idx_shift_swap_workspace ON public.shift_swap_requests USING btree (workspace_id);

CREATE INDEX idx_shift_swap_ws_requester ON public.shift_swap_requests USING btree (workspace_id, requester_id);

CREATE INDEX idx_shift_swap_ws_target ON public.shift_swap_requests USING btree (workspace_id, target_id);

CREATE INDEX idx_shifts_active ON public.shifts USING btree (is_active);

CREATE INDEX idx_shifts_parent_id ON public.shifts USING btree (parent_shift_id);

CREATE INDEX idx_shifts_workspace_id ON public.shifts USING btree (workspace_id);

CREATE UNIQUE INDEX idx_staging_fingerprint ON public.attendance_staging_logs USING btree (workspace_id, fingerprint);

CREATE INDEX idx_staging_pending ON public.attendance_staging_logs USING btree (workspace_id, status) WHERE (status = 'pending'::text);

CREATE INDEX idx_staging_workspace ON public.attendance_staging_logs USING btree (workspace_id);

CREATE UNIQUE INDEX idx_system_roles_unique_name ON public.workspace_roles USING btree (name) WHERE (workspace_id IS NULL);

CREATE INDEX idx_time_entries_emp_punch ON public.time_entries USING btree (employee_id, punch_in_time DESC);

CREATE INDEX idx_time_entries_employee_id ON public.time_entries USING btree (employee_id);

CREATE INDEX idx_time_entries_punch_in_time ON public.time_entries USING btree (punch_in_time);

CREATE INDEX idx_time_entries_workspace_id ON public.time_entries USING btree (workspace_id);

CREATE INDEX idx_time_entries_ws_emp_punch_time ON public.time_entries USING btree (workspace_id, employee_id, punch_in_time);

CREATE INDEX idx_time_entries_ws_employee ON public.time_entries USING btree (workspace_id, employee_id);

CREATE INDEX idx_user_system_roles_role_id ON public.user_system_roles USING btree (role_id);

CREATE INDEX idx_user_system_roles_user_id ON public.user_system_roles USING btree (user_id);

CREATE INDEX idx_working_conditions_employee_date ON public.working_conditions USING btree (employee_id, start_date);

CREATE INDEX idx_working_pattern_details_shift ON public.working_pattern_details USING btree (shift_id);

CREATE INDEX idx_working_patterns_workspace ON public.working_patterns USING btree (workspace_id);

CREATE INDEX idx_workspace_members_user_id ON public.workspace_members USING btree (user_id);

CREATE INDEX idx_workspace_members_workspace_id ON public.workspace_members USING btree (workspace_id);

CREATE INDEX idx_workspace_roles_scope_type ON public.workspace_roles USING btree (scope_type);

CREATE UNIQUE INDEX idx_workspace_roles_unique_name ON public.workspace_roles USING btree (workspace_id, name) WHERE (workspace_id IS NOT NULL);

CREATE TRIGGER trigger_employee_profiles_updated_at BEFORE UPDATE ON public.employee_profiles FOR EACH ROW EXECUTE FUNCTION public.update_employee_profiles_updated_at();

ALTER TABLE ONLY public.assignment_rules
    ADD CONSTRAINT assignment_rules_pattern_id_fkey FOREIGN KEY (pattern_id) REFERENCES public.working_patterns(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.assignment_rules
    ADD CONSTRAINT assignment_rules_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_approved_by_user_id_fkey FOREIGN KEY (approved_by_user_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_requested_by_user_id_fkey FOREIGN KEY (requested_by_user_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_time_entry_id_fkey FOREIGN KEY (time_entry_id) REFERENCES public.time_entries(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_claim_entered_by_user_id_fkey FOREIGN KEY (claim_entered_by_user_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_policies
    ADD CONSTRAINT attendance_policies_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_policy_assignments
    ADD CONSTRAINT attendance_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.attendance_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_policy_versions
    ADD CONSTRAINT attendance_policy_versions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.attendance_policy_versions
    ADD CONSTRAINT attendance_policy_versions_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.attendance_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.attendance_staging_logs
    ADD CONSTRAINT attendance_staging_logs_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE public.audit_logs
    ADD CONSTRAINT audit_logs_actor_id_fkey FOREIGN KEY (actor_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE public.audit_logs
    ADD CONSTRAINT audit_logs_target_user_id_fkey FOREIGN KEY (target_user_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE public.audit_logs
    ADD CONSTRAINT audit_logs_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.authorized_agents
    ADD CONSTRAINT authorized_agents_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.dashboard_preferences
    ADD CONSTRAINT dashboard_preferences_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.dashboard_preferences
    ADD CONSTRAINT dashboard_preferences_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.department_default_shifts
    ADD CONSTRAINT department_default_shifts_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.department_default_shifts
    ADD CONSTRAINT department_default_shifts_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.department_payroll_configs
    ADD CONSTRAINT department_payroll_configs_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_default_working_pattern_id_fkey FOREIGN KEY (default_working_pattern_id) REFERENCES public.working_patterns(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_manager_id_fkey FOREIGN KEY (manager_id) REFERENCES public.employees(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_department_history
    ADD CONSTRAINT employee_department_history_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_department_history
    ADD CONSTRAINT employee_department_history_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_uploaded_by_fkey FOREIGN KEY (uploaded_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_emergency_contacts
    ADD CONSTRAINT employee_emergency_contacts_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_payroll_configs
    ADD CONSTRAINT employee_payroll_configs_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_evaluated_by_fkey FOREIGN KEY (evaluated_by) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_phase_definition_id_fkey FOREIGN KEY (phase_definition_id) REFERENCES public.employment_phase_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

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

ALTER TABLE ONLY public.employee_working_patterns
    ADD CONSTRAINT employee_working_patterns_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_recruitment_source_id_fkey FOREIGN KEY (recruitment_source_id) REFERENCES public.recruitment_sources(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employment_periods
    ADD CONSTRAINT employment_periods_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.feature_flags
    ADD CONSTRAINT feature_flags_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslips
    ADD CONSTRAINT fk_payslips_workspace FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.holidays
    ADD CONSTRAINT holidays_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.holidays
    ADD CONSTRAINT holidays_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_policy_assignment_id_fkey FOREIGN KEY (policy_assignment_id) REFERENCES public.leave_policy_assignments(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.leave_policy_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.leave_policy_definitions
    ADD CONSTRAINT leave_policy_definitions_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_policy_definitions
    ADD CONSTRAINT leave_policy_definitions_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_approved_by_user_id_fkey FOREIGN KEY (approved_by_user_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_requested_by_user_id_fkey FOREIGN KEY (requested_by_user_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.leave_types
    ADD CONSTRAINT leave_types_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.overtime_policy_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.overtime_policy_definitions
    ADD CONSTRAINT overtime_policy_definitions_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.overtime_rules
    ADD CONSTRAINT overtime_rules_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.overtime_policy_definitions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_incentive_tiers
    ADD CONSTRAINT payroll_incentive_tiers_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_rule_definition_id_fkey FOREIGN KEY (rule_definition_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_rule_definitions
    ADD CONSTRAINT payroll_rule_definitions_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_rule_tiers
    ADD CONSTRAINT payroll_rule_tiers_rule_definition_id_fkey FOREIGN KEY (rule_definition_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_settings
    ADD CONSTRAINT payroll_settings_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslip_incentives
    ADD CONSTRAINT payslip_incentives_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslip_rule_evaluations
    ADD CONSTRAINT payslip_rule_evaluations_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslip_segments
    ADD CONSTRAINT payslip_segments_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id);

ALTER TABLE ONLY public.payslip_segments
    ADD CONSTRAINT payslip_segments_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslips
    ADD CONSTRAINT payslips_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.policy_templates
    ADD CONSTRAINT policy_templates_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT raw_attendance_logs_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT raw_attendance_logs_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.recruitment_sources
    ADD CONSTRAINT recruitment_sources_referrer_employee_id_fkey FOREIGN KEY (referrer_employee_id) REFERENCES public.employees(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.recruitment_sources
    ADD CONSTRAINT recruitment_sources_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.salary_base_groups
    ADD CONSTRAINT salary_base_groups_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.salary_component_base_members
    ADD CONSTRAINT salary_component_base_members_base_group_id_fkey FOREIGN KEY (base_group_id) REFERENCES public.salary_base_groups(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.salary_component_base_members
    ADD CONSTRAINT salary_component_base_members_component_definition_id_fkey FOREIGN KEY (component_definition_id) REFERENCES public.salary_component_definitions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_linked_rule_id_fkey FOREIGN KEY (linked_rule_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.shift_rosters
    ADD CONSTRAINT shift_rosters_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.shift_rosters
    ADD CONSTRAINT shift_rosters_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_approved_by_user_id_fkey FOREIGN KEY (approved_by_user_id) REFERENCES public.users(id);

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requested_by_user_id_fkey FOREIGN KEY (requested_by_user_id) REFERENCES public.users(id);

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requester_id_fkey FOREIGN KEY (requester_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requester_shift_id_fkey FOREIGN KEY (requester_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_target_id_fkey FOREIGN KEY (target_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_target_shift_id_fkey FOREIGN KEY (target_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.shifts
    ADD CONSTRAINT shifts_parent_shift_id_fkey FOREIGN KEY (parent_shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.shifts
    ADD CONSTRAINT shifts_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id);

ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.system_roles(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.working_pattern_details
    ADD CONSTRAINT working_pattern_details_pattern_id_fkey FOREIGN KEY (pattern_id) REFERENCES public.working_patterns(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.working_pattern_details
    ADD CONSTRAINT working_pattern_details_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES public.shifts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.working_patterns
    ADD CONSTRAINT working_patterns_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_permissions_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_permissions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_permissions_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id);

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_role_permissions
    ADD CONSTRAINT workspace_role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_roles
    ADD CONSTRAINT workspace_roles_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

-- ═══════════════════════════════════════════════════════════════════
-- Seed Data: System Roles & Permissions
-- ═══════════════════════════════════════════════════════════════════

-- Seed System Roles with deterministic UUIDs
INSERT INTO workspace_roles (id, workspace_id, name, description, color, is_system, scope_type)
VALUES 
  ('a0000000-0000-0000-0000-000000000001', NULL, 'owner', 'Workspace owner with full access', 'rose', TRUE, 'global'),
  ('a0000000-0000-0000-0000-000000000002', NULL, 'admin', 'Administrator with full access', 'rose', TRUE, 'global'),
  ('a0000000-0000-0000-0000-000000000003', NULL, 'hr_manager', 'Manage people and time', 'indigo', TRUE, 'global'),
  ('a0000000-0000-0000-0000-000000000004', NULL, 'shift_manager', 'Manage shifts', 'amber', TRUE, 'global'),
  ('a0000000-0000-0000-0000-000000000005', NULL, 'team_lead', 'Manage team members', 'blue', TRUE, 'scoped'),
  ('a0000000-0000-0000-0000-000000000006', NULL, 'employee', 'Default employee access', 'slate', TRUE, 'scoped')
ON CONFLICT DO NOTHING;

-- Seed Permissions for System Roles
-- Owner has all core permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000001', 'workspace:update'),
  ('a0000000-0000-0000-0000-000000000001', 'workspace:delete'),
  ('a0000000-0000-0000-0000-000000000001', 'workspace:settings:read'),
  ('a0000000-0000-0000-0000-000000000001', 'workspace:settings:write'),
  ('a0000000-0000-0000-0000-000000000001', 'members:read'),
  ('a0000000-0000-0000-0000-000000000001', 'members:invite'),
  ('a0000000-0000-0000-0000-000000000001', 'members:roles:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'members:remove'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:create'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:update'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:delete'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:rehire'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:upload'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:read'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:create'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:update'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:delete'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:transfer'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:logs:read'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:logs:import'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:adjustments:submit'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:adjustments:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:reprocess'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:staging:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:policies:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:paper:upload'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:requests:submit'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:requests:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:balances:read'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:balances:adjust'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:types:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:policies:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:accrual:run'),
  ('a0000000-0000-0000-0000-000000000001', 'shifts:read'),
  ('a0000000-0000-0000-0000-000000000001', 'shifts:create'),
  ('a0000000-0000-0000-0000-000000000001', 'shifts:update'),
  ('a0000000-0000-0000-0000-000000000001', 'shifts:assign'),
  ('a0000000-0000-0000-0000-000000000001', 'shift_swaps:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'shift_swaps:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'patterns:read'),
  ('a0000000-0000-0000-0000-000000000001', 'patterns:create'),
  ('a0000000-0000-0000-0000-000000000001', 'patterns:assign'),
  ('a0000000-0000-0000-0000-000000000001', 'patterns:delete'),
  ('a0000000-0000-0000-0000-000000000001', 'rosters:read'),
  ('a0000000-0000-0000-0000-000000000001', 'rosters:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'assignment_rules:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:read'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:calculate'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:settings:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:components:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:incentives:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'holidays:read'),
  ('a0000000-0000-0000-0000-000000000001', 'holidays:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'reports:read'),
  ('a0000000-0000-0000-0000-000000000001', 'reports:export'),
  ('a0000000-0000-0000-0000-000000000001', 'dashboard:read'),
  ('a0000000-0000-0000-0000-000000000001', 'dashboard:preferences'),
  ('a0000000-0000-0000-0000-000000000001', 'agents:read'),
  ('a0000000-0000-0000-0000-000000000001', 'agents:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'devices:read'),
  ('a0000000-0000-0000-0000-000000000001', 'devices:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'employment_phases:read'),
  ('a0000000-0000-0000-0000-000000000001', 'employment_phases:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'recruitment:read'),
  ('a0000000-0000-0000-0000-000000000001', 'recruitment:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'feature_flags:read'),
  ('a0000000-0000-0000-0000-000000000001', 'feature_flags:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'infra:monitor'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:profile'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:attendance'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:clock'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:leave'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:payslips'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:shift_swaps')
ON CONFLICT DO NOTHING;

-- Admin has similar permissions to Owner
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000002', 'workspace:settings:write'),
  ('a0000000-0000-0000-0000-000000000002', 'workspace:settings:read'),
  ('a0000000-0000-0000-0000-000000000002', 'members:invite'),
  ('a0000000-0000-0000-0000-000000000002', 'members:roles:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'members:remove'),
  ('a0000000-0000-0000-0000-000000000002', 'members:read'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:create'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:update'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:delete'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:rehire'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:upload'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:read'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:create'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:update'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:delete'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:transfer'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:logs:read'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:logs:import'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:submit'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:reprocess'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:staging:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:policies:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:paper:upload'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:requests:submit'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:requests:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:balances:read'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:balances:adjust'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:types:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:policies:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:accrual:run'),
  ('a0000000-0000-0000-0000-000000000002', 'shifts:read'),
  ('a0000000-0000-0000-0000-000000000002', 'shifts:create'),
  ('a0000000-0000-0000-0000-000000000002', 'shifts:update'),
  ('a0000000-0000-0000-0000-000000000002', 'shifts:assign'),
  ('a0000000-0000-0000-0000-000000000002', 'shift_swaps:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'shift_swaps:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'patterns:read'),
  ('a0000000-0000-0000-0000-000000000002', 'patterns:create'),
  ('a0000000-0000-0000-0000-000000000002', 'patterns:assign'),
  ('a0000000-0000-0000-0000-000000000002', 'patterns:delete'),
  ('a0000000-0000-0000-0000-000000000002', 'rosters:read'),
  ('a0000000-0000-0000-0000-000000000002', 'rosters:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'assignment_rules:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:read'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:calculate'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:settings:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:components:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:incentives:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'holidays:read'),
  ('a0000000-0000-0000-0000-000000000002', 'holidays:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'reports:read'),
  ('a0000000-0000-0000-0000-000000000002', 'reports:export'),
  ('a0000000-0000-0000-0000-000000000002', 'dashboard:read'),
  ('a0000000-0000-0000-0000-000000000002', 'dashboard:preferences'),
  ('a0000000-0000-0000-0000-000000000002', 'agents:read'),
  ('a0000000-0000-0000-0000-000000000002', 'agents:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'devices:read'),
  ('a0000000-0000-0000-0000-000000000002', 'devices:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'employment_phases:read'),
  ('a0000000-0000-0000-0000-000000000002', 'employment_phases:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'recruitment:read'),
  ('a0000000-0000-0000-0000-000000000002', 'recruitment:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'feature_flags:read'),
  ('a0000000-0000-0000-0000-000000000002', 'feature_flags:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'infra:monitor'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:profile'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:attendance'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:clock'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:leave'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:payslips'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:shift_swaps')
ON CONFLICT DO NOTHING;

-- HrManager permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000003', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000003', 'employees:create'),
  ('a0000000-0000-0000-0000-000000000003', 'employees:update'),
  ('a0000000-0000-0000-0000-000000000003', 'attendance:logs:read'),
  ('a0000000-0000-0000-0000-000000000003', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000003', 'attendance:adjustments:approve'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:requests:approve'),
  ('a0000000-0000-0000-0000-000000000003', 'payroll:read'),
  ('a0000000-0000-0000-0000-000000000003', 'departments:read'),
  ('a0000000-0000-0000-0000-000000000003', 'holidays:read'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:requests:submit'),
  ('a0000000-0000-0000-0000-000000000003', 'employment_phases:read'),
  ('a0000000-0000-0000-0000-000000000003', 'employment_phases:manage')
ON CONFLICT DO NOTHING;

-- ShiftManager permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000004', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000004', 'shifts:read'),
  ('a0000000-0000-0000-0000-000000000004', 'shifts:create'),
  ('a0000000-0000-0000-0000-000000000004', 'shifts:update'),
  ('a0000000-0000-0000-0000-000000000004', 'shifts:assign'),
  ('a0000000-0000-0000-0000-000000000004', 'rosters:read'),
  ('a0000000-0000-0000-0000-000000000004', 'rosters:manage'),
  ('a0000000-0000-0000-0000-000000000004', 'attendance:logs:read'),
  ('a0000000-0000-0000-0000-000000000004', 'employment_phases:read')
ON CONFLICT DO NOTHING;

-- TeamLead permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000005', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000005', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:submit'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:approve')
ON CONFLICT DO NOTHING;

-- Employee default permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000006', 'ess:profile'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:attendance'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:clock'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:leave'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:payslips'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:shift_swaps')
ON CONFLICT DO NOTHING;

INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000001', 'audit_logs:read'),
  ('a0000000-0000-0000-0000-000000000001', 'audit_logs:export'),
  ('a0000000-0000-0000-0000-000000000001', 'security:settings:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:bulk_edit'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:force_clock_out'),
  ('a0000000-0000-0000-0000-000000000001', 'documents:medical:read'),
  ('a0000000-0000-0000-0000-000000000001', 'documents:contract:read'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:disburse'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:sensitive:read'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:approve:l1'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:approve:l2')
ON CONFLICT DO NOTHING;

INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000002', 'audit_logs:read'),
  ('a0000000-0000-0000-0000-000000000002', 'audit_logs:export'),
  ('a0000000-0000-0000-0000-000000000002', 'security:settings:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:bulk_edit'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:force_clock_out'),
  ('a0000000-0000-0000-0000-000000000002', 'documents:medical:read'),
  ('a0000000-0000-0000-0000-000000000002', 'documents:contract:read'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:sensitive:read'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:approve:l1'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:approve:l2')
ON CONFLICT DO NOTHING;

INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000003', 'employees:bulk_edit'),
  ('a0000000-0000-0000-0000-000000000003', 'documents:contract:read'),
  ('a0000000-0000-0000-0000-000000000003', 'payroll:approve'),
  ('a0000000-0000-0000-0000-000000000003', 'payroll:sensitive:read'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:approve:l2')
ON CONFLICT DO NOTHING;

INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000005', 'leave:approve:l1')
ON CONFLICT DO NOTHING;

-- System-level platform roles
INSERT INTO system_roles (id, name, description) VALUES
    ('c0000000-0000-0000-0000-000000000001', 'platform_admin', 'Full platform control: create/suspend workspaces, assign system roles, manage infrastructure'),
    ('c0000000-0000-0000-0000-000000000002', 'support',        'Read-only cross-workspace access for debugging customer issues'),
    ('c0000000-0000-0000-0000-000000000003', 'billing_admin',  'Manage billing, subscription plans, and invoicing')
ON CONFLICT DO NOTHING;