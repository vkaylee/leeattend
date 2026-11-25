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


-- Fresh install phase: attendance_schedule

-- Name: assignment_rules; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.assignment_rules (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

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





--

-- Name: attendance_adjustment_requests; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_adjustment_requests (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    time_entry_id uuid,

    requested_punch_time timestamp with time zone CONSTRAINT attendance_adjustment_requests_requested_punch_in_time_not_null NOT NULL,

    reason text,

    status public.adjustment_status_type DEFAULT 'pending'::public.adjustment_status_type NOT NULL,

    requested_by_user_id uuid,

    approved_by_user_id uuid,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    employment_id uuid NOT NULL,

    attachment_file_ids uuid[],
    adjustment_type text NOT NULL DEFAULT 'attendance_correction',
    details jsonb NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT attendance_adjustment_requests_type_check CHECK (adjustment_type IN ('break_worked', 'missing_punch', 'attendance_correction')),
    CONSTRAINT attendance_adjustment_requests_details_check CHECK (jsonb_typeof(details) = 'object')

);

CREATE INDEX attendance_adjustment_requests_type_idx ON public.attendance_adjustment_requests (adjustment_type, status, created_at DESC);





--

-- Name: attendance_closure_days; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_closure_days (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    closure_id uuid NOT NULL,

    period_id uuid NOT NULL,

    employer_id uuid NOT NULL,

    employment_id uuid NOT NULL,

    business_date date NOT NULL,

    schedule_revision_id uuid NOT NULL,

    schedule_state character varying(16) NOT NULL,

    schedule_snapshot jsonb NOT NULL,

    attendance_snapshot jsonb NOT NULL,

    calculation_snapshot jsonb NOT NULL,

    time_entry_ids uuid[] DEFAULT '{}'::uuid[] NOT NULL,

    raw_attendance_log_ids uuid[] DEFAULT '{}'::uuid[] NOT NULL,
    time_entry_snapshot jsonb NOT NULL DEFAULT '[]'::jsonb,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT attendance_closure_days_values_check CHECK ((((schedule_state)::text = ANY ((ARRAY['published'::character varying, 'explicit_off'::character varying])::text[])) AND (schedule_revision_id IS NOT NULL) AND (jsonb_typeof(schedule_snapshot) = 'object'::text) AND (jsonb_typeof(attendance_snapshot) = 'object'::text) AND (jsonb_typeof(calculation_snapshot) = 'object'::text) AND (jsonb_typeof(time_entry_snapshot) = 'array'::text)))

);





--

-- Name: attendance_closures; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_closures (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    period_id uuid NOT NULL,

    employer_id uuid NOT NULL,

    closure_number integer NOT NULL,

    status character varying(16) DEFAULT 'closed'::character varying NOT NULL,

    closed_by uuid,

    closed_actor_type character varying(16) DEFAULT 'human'::character varying NOT NULL,

    closed_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    snapshot_hash character(64) NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT attendance_closures_values_check CHECK (((closure_number >= 1) AND ((status)::text = 'closed'::text) AND ((closed_actor_type)::text = ANY ((ARRAY['human'::character varying, 'system'::character varying])::text[])) AND ((snapshot_hash)::text ~ '^[0-9a-f]{64}$'::text)))

);





--

-- Name: attendance_daily_summaries; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_daily_summaries (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

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

    employment_id uuid NOT NULL,

    standard_minutes integer DEFAULT 0 NOT NULL,

    early_ot_minutes integer DEFAULT 0 NOT NULL,

    late_ot_minutes integer DEFAULT 0 NOT NULL,

    break_ot_minutes integer DEFAULT 0 NOT NULL,

    is_weekend boolean DEFAULT false NOT NULL,

    late_minutes integer DEFAULT 0 NOT NULL,

    early_departure_minutes integer DEFAULT 0 NOT NULL,

    work_policy_version_id uuid,
    shift_version_id uuid,

    scheduled_break_minutes integer NOT NULL DEFAULT 0,
    actual_break_minutes integer NOT NULL DEFAULT 0,
    deducted_break_minutes integer NOT NULL DEFAULT 0,
    net_work_minutes integer NOT NULL DEFAULT 0,
    net_paid_minutes integer NOT NULL DEFAULT 0,
    break_provisional boolean NOT NULL DEFAULT false,
    break_exception_code text,

    CONSTRAINT attendance_daily_summaries_minutes_nonnegative CHECK (
        actual_work_minutes >= 0 AND standard_minutes >= 0 AND system_detected_ot_minutes >= 0
        AND early_ot_minutes >= 0 AND late_ot_minutes >= 0 AND break_ot_minutes >= 0
        AND scheduled_break_minutes >= 0 AND actual_break_minutes >= 0
        AND deducted_break_minutes >= 0 AND net_work_minutes >= 0 AND net_paid_minutes >= 0
    ),
    CONSTRAINT attendance_daily_summaries_break_exception_check CHECK (
        (break_provisional AND break_exception_code IS NOT NULL)
        OR (NOT break_provisional)
    )

);





--

-- Name: attendance_import_jobs; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_import_jobs (

    id uuid NOT NULL,

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





--

-- Name: attendance_log_errors; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_log_errors (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    agent_id text,

    raw_content jsonb,

    error_message text NOT NULL,

    source text NOT NULL,

    created_at timestamp with time zone DEFAULT now(),

    fingerprint text

);





--

-- Name: attendance_periods; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_periods (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employer_id uuid NOT NULL,

    period_year integer NOT NULL,

    period_month integer NOT NULL,

    status character varying(16) DEFAULT 'open'::character varying NOT NULL,

    version bigint DEFAULT 1 NOT NULL,

    current_closure_id uuid,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT attendance_periods_values_check CHECK ((((period_year >= 2000) AND (period_year <= 2100)) AND ((period_month >= 1) AND (period_month <= 12)) AND ((status)::text = ANY ((ARRAY['open'::character varying, 'closed'::character varying, 'reopened'::character varying])::text[])) AND (version >= 1)))

);





--

-- Name: attendance_policies; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_policies (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name character varying(255) NOT NULL,

    is_default boolean DEFAULT false NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: attendance_policy_assignments; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: attendance_policy_versions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_policy_versions (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    policy_id uuid NOT NULL,

    rules jsonb DEFAULT '{}'::jsonb NOT NULL,

    valid_from timestamp with time zone NOT NULL,

    valid_to timestamp with time zone,

    created_by uuid,

    created_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: attendance_source_pull_states; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_source_pull_states (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    source_id uuid NOT NULL,

    cursor text,

    status character varying(20) DEFAULT 'idle'::character varying NOT NULL,

    lease_owner character varying(100),

    lease_token uuid,

    lease_expires_at timestamp with time zone,

    last_success_at timestamp with time zone,

    last_failure_at timestamp with time zone,

    last_error text,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT attendance_source_pull_states_status_check CHECK (((status)::text = ANY (ARRAY[('idle'::character varying)::text, ('running'::character varying)::text, ('failed'::character varying)::text])))

);





--

-- Name: attendance_sources; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_sources (

    id uuid NOT NULL,

    provider_source_id text NOT NULL,

    is_enabled boolean DEFAULT true NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL

);





--

-- Name: attendance_staging_logs; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL

)

PARTITION BY RANGE (created_at);





--

-- Name: attendance_staging_logs_default; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_default (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m01; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m01 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m02; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m02 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m03; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m03 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m04; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m04 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m05; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m05 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m06; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m06 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m07; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m07 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m08; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m08 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m09; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m09 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m10; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m10 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m11; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m11 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: attendance_staging_logs_y2026m12; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.attendance_staging_logs_y2026m12 (

    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,

    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,

    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,

    raw_data jsonb,

    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,

    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,

    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL

);





--

-- Name: employee_shifts; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.employee_shifts (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    start_date date NOT NULL,

    end_date date,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    employment_id uuid NOT NULL,

    template_id uuid NOT NULL

);





--

-- Name: employee_working_patterns; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.employee_working_patterns (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    pattern_id uuid NOT NULL,

    start_date date NOT NULL,

    end_date date,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    anchor_date date,

    employment_id uuid NOT NULL,

    CONSTRAINT employee_working_patterns_check CHECK (((end_date IS NULL) OR (end_date >= start_date)))

);





--

-- Name: external_attendance_events; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.external_attendance_events (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    external_event_id character varying(255) NOT NULL,

    occurred_at timestamp with time zone NOT NULL,

    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,

    received_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    source_id text NOT NULL

);





--

-- Name: holidays; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.holidays (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

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





--

-- Name: payroll_periods; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.payroll_periods (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employer_id uuid NOT NULL,

    period_year integer NOT NULL,

    period_month integer NOT NULL,

    status character varying(16) DEFAULT 'open'::character varying NOT NULL,

    version bigint DEFAULT 1 NOT NULL,

    locked_at timestamp with time zone,

    locked_by uuid,

    reopened_at timestamp with time zone,

    reopened_by uuid,

    reopen_reason text,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT payroll_periods_month_check CHECK (((period_month >= 1) AND (period_month <= 12))),

    CONSTRAINT payroll_periods_status_check CHECK (((status)::text = ANY ((ARRAY['open'::character varying, 'locked'::character varying])::text[]))),

    CONSTRAINT payroll_periods_year_check CHECK (((period_year >= 2000) AND (period_year <= 2100)))

);





--

-- Name: raw_attendance_logs; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.raw_attendance_logs (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    punch_time timestamp with time zone NOT NULL,

    source public.log_source_type NOT NULL,

    raw_data jsonb,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    punch integer DEFAULT '-1'::integer NOT NULL,

    status integer DEFAULT '-1'::integer NOT NULL,

    employment_id uuid NOT NULL,

    source_timestamp text,

    source_timezone text,

    timestamp_kind text,

    conversion_status text,

    conversion_error text,

    received_at timestamp with time zone

);





--

-- Name: schedule_day_revisions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.schedule_day_revisions (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employment_id uuid NOT NULL,

    employer_id uuid NOT NULL,

    business_date date NOT NULL,

    timezone character varying(64) NOT NULL,

    revision_number bigint NOT NULL,

    state character varying(24) NOT NULL,

    previous_revision_id uuid,

    publication_batch_id uuid,

    source character varying(24) NOT NULL,

    source_snapshot jsonb DEFAULT '{}'::jsonb NOT NULL,

    resolver_version character varying(32) NOT NULL,

    reason text NOT NULL,

    published_by uuid,

    published_actor_type character varying(16) DEFAULT 'human'::character varying NOT NULL,

    published_actor_name character varying(64),

    published_at timestamp with time zone,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT schedule_day_revisions_actor_check CHECK (((published_actor_type)::text = ANY ((ARRAY['human'::character varying, 'system'::character varying])::text[]))),

    CONSTRAINT schedule_day_revisions_batch_check CHECK (((((state)::text = 'legacy_unverified'::text) AND (publication_batch_id IS NULL) AND ((source_snapshot ->> 'provenance'::text) = 'legacy_unverified'::text)) OR (((state)::text <> 'legacy_unverified'::text) AND (publication_batch_id IS NOT NULL)))),

    CONSTRAINT schedule_day_revisions_publication_check CHECK (((((state)::text = ANY (ARRAY[('published'::character varying)::text, ('explicit_off'::character varying)::text, ('superseded'::character varying)::text])) AND (published_at IS NOT NULL)) OR (((state)::text = ANY (ARRAY[('draft'::character varying)::text, ('legacy_unverified'::character varying)::text])) AND (published_at IS NULL)))),

    CONSTRAINT schedule_day_revisions_revision_number_check CHECK ((revision_number >= 1)),

    CONSTRAINT schedule_day_revisions_snapshot_check CHECK ((jsonb_typeof(source_snapshot) = 'object'::text)),

    CONSTRAINT schedule_day_revisions_source_check CHECK (((source)::text = ANY ((ARRAY['roster'::character varying, 'pattern'::character varying, 'assignment_rule'::character varying, 'fixed'::character varying, 'manual'::character varying, 'auto'::character varying, 'legacy_unverified'::character varying])::text[]))),

    CONSTRAINT schedule_day_revisions_state_check CHECK (((state)::text = ANY (ARRAY[('draft'::character varying)::text, ('published'::character varying)::text, ('superseded'::character varying)::text, ('explicit_off'::character varying)::text, ('legacy_unverified'::character varying)::text])))

);





--

-- Name: schedule_legacy_mutation_evidence; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.schedule_legacy_mutation_evidence (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employment_id uuid,

    business_date date,

    operation character varying(32) NOT NULL,

    source character varying(32) DEFAULT 'database_trigger'::character varying NOT NULL,

    details jsonb DEFAULT '{}'::jsonb NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT schedule_legacy_mutation_evidence_details_check CHECK ((jsonb_typeof(details) = 'object'::text)),

    CONSTRAINT schedule_legacy_mutation_evidence_operation_check CHECK (((operation)::text = ANY ((ARRAY['insert'::character varying, 'update'::character varying, 'delete'::character varying, 'guard_blocked'::character varying])::text[])))

);





--

-- Name: schedule_office_policies; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.schedule_office_policies (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employer_id uuid NOT NULL,

    approval_instance_id uuid NOT NULL,

    effective_from date NOT NULL,

    effective_to date,

    status character varying(16) DEFAULT 'active'::character varying NOT NULL,

    created_by uuid,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    revoked_at timestamp with time zone,

    CONSTRAINT schedule_office_policies_range_check CHECK (((effective_to IS NULL) OR (effective_to >= effective_from))),

    CONSTRAINT schedule_office_policies_revoked_check CHECK (((((status)::text = 'active'::text) AND (revoked_at IS NULL)) OR (((status)::text = 'revoked'::text) AND (revoked_at IS NOT NULL)))),

    CONSTRAINT schedule_office_policies_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'revoked'::character varying])::text[])))

);





--

-- Name: schedule_previews; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.schedule_previews (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    requested_by uuid,

    actor_type character varying(16) NOT NULL,

    actor_name character varying(64),

    operation character varying(16) NOT NULL,

    request_hash character(64) NOT NULL,

    result_hash character(64) NOT NULL,

    scope_snapshot jsonb NOT NULL,

    result_snapshot jsonb NOT NULL,

    resolver_version character varying(32) NOT NULL,

    expires_at timestamp with time zone NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT schedule_previews_actor_check CHECK (((((actor_type)::text = 'human'::text) AND (requested_by IS NOT NULL) AND (actor_name IS NULL)) OR (((actor_type)::text = 'system'::text) AND (requested_by IS NULL) AND ((actor_name)::text = 'schedule-publisher'::text) AND ((operation)::text = 'publish'::text)))),

    CONSTRAINT schedule_previews_expiry_check CHECK (((expires_at > created_at) AND (expires_at <= (created_at + '00:15:00'::interval)))),

    CONSTRAINT schedule_previews_hash_check CHECK ((((request_hash)::text ~ '^[0-9a-f]{64}$'::text) AND ((result_hash)::text ~ '^[0-9a-f]{64}$'::text))),

    CONSTRAINT schedule_previews_operation_check CHECK (((operation)::text = ANY ((ARRAY['publish'::character varying, 'replan'::character varying])::text[]))),

    CONSTRAINT schedule_previews_snapshot_check CHECK (((jsonb_typeof(scope_snapshot) = 'object'::text) AND (jsonb_typeof(result_snapshot) = 'object'::text)))

);





--

-- Name: schedule_publication_batches; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.schedule_publication_batches (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    idempotency_key character varying(128) NOT NULL,

    request_hash character(64) NOT NULL,

    mode character varying(16) DEFAULT 'office'::character varying NOT NULL,

    status character varying(16) NOT NULL,

    requested_by uuid,

    operation character varying(16) NOT NULL,

    response_snapshot jsonb NOT NULL,

    actor_type character varying(16) DEFAULT 'human'::character varying NOT NULL,

    actor_name character varying(64),

    approval_instance_id uuid,

    reason text NOT NULL,

    timezone character varying(64) NOT NULL,

    range_start date NOT NULL,

    range_end date NOT NULL,

    published_at timestamp with time zone,

    affected_days integer DEFAULT 0 NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT schedule_publication_batches_operation_check CHECK (((operation)::text = ANY ((ARRAY['publish'::character varying, 'replan'::character varying])::text[]))),

    CONSTRAINT schedule_publication_batches_request_hash_check CHECK (((request_hash)::text ~ '^[0-9a-f]{64}$'::text)),

    CONSTRAINT schedule_publication_batches_response_check CHECK ((jsonb_typeof(response_snapshot) = 'object'::text)),

    CONSTRAINT schedule_publication_batches_values_check CHECK ((((mode)::text = ANY ((ARRAY['office'::character varying, 'rotating'::character varying])::text[])) AND ((status)::text = 'published'::text) AND ((actor_type)::text = ANY ((ARRAY['human'::character varying, 'system'::character varying])::text[])) AND (range_start <= range_end) AND ((char_length(TRIM(BOTH FROM reason)) >= 1) AND (char_length(TRIM(BOTH FROM reason)) <= 2000)) AND (affected_days >= 0) AND ((((actor_type)::text = 'human'::text) AND (requested_by IS NOT NULL) AND (actor_name IS NULL)) OR (((actor_type)::text = 'system'::text) AND (requested_by IS NULL) AND ((actor_name)::text = 'schedule-publisher'::text))) AND ((((mode)::text = 'office'::text) AND (approval_instance_id IS NOT NULL)) OR (((mode)::text = 'rotating'::text) AND (approval_instance_id IS NULL)))))

);





--

-- Name: schedule_shadow_mismatches; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.schedule_shadow_mismatches (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employer_id uuid NOT NULL,

    employment_id uuid NOT NULL,

    business_date date NOT NULL,

    mismatch_kind character varying(32) NOT NULL,

    legacy_snapshot jsonb NOT NULL,

    resolver_snapshot jsonb NOT NULL,

    resolver_version character varying(32) NOT NULL,

    status character varying(16) DEFAULT 'open'::character varying NOT NULL,

    first_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    last_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    resolved_at timestamp with time zone,

    resolution_note text,

    CONSTRAINT schedule_shadow_mismatches_dates_check CHECK ((last_seen_at >= first_seen_at)),

    CONSTRAINT schedule_shadow_mismatches_kind_check CHECK (((mismatch_kind)::text = ANY ((ARRAY['intervals'::character varying, 'overnight'::character varying, 'explicit_off'::character varying, 'missing'::character varying, 'effective_version'::character varying, 'timezone'::character varying, 'source'::character varying, 'other'::character varying])::text[]))),

    CONSTRAINT schedule_shadow_mismatches_resolution_check CHECK (((((status)::text = 'open'::text) AND (resolved_at IS NULL)) OR (((status)::text = ANY ((ARRAY['resolved'::character varying, 'waived'::character varying])::text[])) AND (resolved_at IS NOT NULL)))),

    CONSTRAINT schedule_shadow_mismatches_snapshot_check CHECK (((jsonb_typeof(legacy_snapshot) = 'object'::text) AND (jsonb_typeof(resolver_snapshot) = 'object'::text))),

    CONSTRAINT schedule_shadow_mismatches_status_check CHECK (((status)::text = ANY ((ARRAY['open'::character varying, 'resolved'::character varying, 'waived'::character varying])::text[])))

);





--

-- Name: shift_audit_log; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.shift_audit_log (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    template_id uuid NOT NULL,

    version_id uuid NOT NULL,

    action character varying(20) DEFAULT 'version_created'::character varying NOT NULL,

    change_type character varying(20) NOT NULL,

    changed_by uuid NOT NULL,

    change_reason text,

    old_values jsonb,

    new_values jsonb,

    created_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: shift_default_intervals; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.shift_default_intervals (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    sort_order integer NOT NULL,

    interval_name character varying(100),

    start_time time without time zone NOT NULL,

    end_time time without time zone NOT NULL,

    break_duration_minutes integer DEFAULT 0 NOT NULL,

    break_start time without time zone,

    break_end time without time zone,

    break_punch_mode text NOT NULL DEFAULT 'none',

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    version_id uuid NOT NULL,

    start_day_offset integer DEFAULT 0 NOT NULL,

    end_day_offset integer DEFAULT 0 NOT NULL,

    start_utc_offset_minutes integer,

    end_utc_offset_minutes integer,

    CONSTRAINT shift_default_intervals_day_offsets_check CHECK (((start_day_offset >= 0) AND (end_day_offset >= start_day_offset))),

    CONSTRAINT shift_default_intervals_break_punch_mode_check CHECK (break_punch_mode IN ('none', 'required')),

    CONSTRAINT shift_default_intervals_sort_order_check CHECK ((sort_order >= 0)),

    CONSTRAINT shift_default_intervals_utc_offsets_check CHECK ((((start_utc_offset_minutes IS NULL) OR (((start_utc_offset_minutes >= '-840'::integer) AND (start_utc_offset_minutes <= 840)) AND ((start_utc_offset_minutes % 15) = 0))) AND ((end_utc_offset_minutes IS NULL) OR (((end_utc_offset_minutes >= '-840'::integer) AND (end_utc_offset_minutes <= 840)) AND ((end_utc_offset_minutes % 15) = 0)))))

);





--

-- Name: shift_rosters; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.shift_rosters (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    date date NOT NULL,

    status character varying(20) DEFAULT 'confirmed'::character varying,

    notes text,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    employment_id uuid NOT NULL,

    template_id uuid,

    -- NULL means legacy, ambiguous, or not-yet-materialized roster policy.
    shift_version_id uuid

);





--

-- Name: shift_swap_requests; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.shift_swap_requests (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    requester_id uuid NOT NULL,

    target_id uuid NOT NULL,

    swap_date date NOT NULL,

    status character varying(20) DEFAULT 'pending_admin'::character varying NOT NULL,

    reason text,

    requested_by_user_id uuid,

    approved_by_user_id uuid,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    requester_template_id uuid,

    target_template_id uuid,

    requester_revision_id uuid,

    requester_revision_number bigint,

    target_revision_id uuid,

    target_revision_number bigint,

    CONSTRAINT shift_swap_requests_revision_numbers_check CHECK ((((requester_revision_id IS NULL) AND (requester_revision_number IS NULL)) OR ((requester_revision_id IS NOT NULL) AND (requester_revision_number >= 1)))),

    CONSTRAINT shift_swap_requests_target_revision_numbers_check CHECK ((((target_revision_id IS NULL) AND (target_revision_number IS NULL)) OR ((target_revision_id IS NOT NULL) AND (target_revision_number >= 1))))

);





--

-- Name: shift_templates; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.shift_templates (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name character varying(100) NOT NULL,

    current_version_id uuid,

    created_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: shift_versions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.shift_versions (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    template_id uuid NOT NULL,

    version_number integer NOT NULL,

    name character varying(100) NOT NULL,

    report_symbol character varying(10),

    location_id uuid,

    employer_id uuid,

    effective_from date NOT NULL,

    effective_to date,

    change_type character varying(20) DEFAULT 'create'::character varying NOT NULL,

    change_reason text,

    superseded_by uuid,

    created_by uuid NOT NULL,

    created_at timestamp with time zone DEFAULT now() NOT NULL

);





--

-- Name: shifts; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.shifts (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name character varying(100) NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,

    is_active boolean DEFAULT true NOT NULL,

    parent_shift_id uuid,

    report_symbol character varying(10),

    location_id uuid,

    employer_id uuid

);





--

-- Name: time_entries; Type: TABLE; Schema: public; Owner: -

--



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

    generation_metadata jsonb DEFAULT '{}'::jsonb,

    employment_id uuid NOT NULL,

    segment_id uuid,

    template_id uuid,

    -- NULL preserves historical entries without a materialized policy snapshot.
    shift_version_id uuid,

    breaks_snapshot jsonb

);





--

-- Name: work_segments; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.work_segments (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employment_id uuid NOT NULL,

    date date NOT NULL,

    interval_id uuid,

    sort_order integer DEFAULT 0 NOT NULL,

    scheduled_start time without time zone NOT NULL,

    scheduled_end time without time zone NOT NULL,

    segment_type public.segment_type DEFAULT 'work'::public.segment_type NOT NULL,

    segment_status public.segment_status_type DEFAULT 'scheduled'::public.segment_status_type NOT NULL,

    compensation_rate numeric(3,2) DEFAULT 1.00 NOT NULL,

    gap_compensation_before public.gap_compensation_type DEFAULT 'unpaid'::public.gap_compensation_type NOT NULL,

    gap_compensation_rate_before numeric(3,2) DEFAULT 0.00 NOT NULL,

    break_config_override jsonb,

    ot_config_override jsonb,

    source character varying(30) DEFAULT 'manual'::character varying NOT NULL,

    source_detail jsonb DEFAULT '{}'::jsonb NOT NULL,

    notes text,

    generation_metadata jsonb DEFAULT '{}'::jsonb,

    created_at timestamp with time zone DEFAULT now() NOT NULL,

    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    template_id uuid NOT NULL,

    schedule_revision_id uuid,

    selected_shift_version_id uuid,

    scheduled_start_at timestamp with time zone,

    scheduled_end_at timestamp with time zone,

    breaks_snapshot jsonb DEFAULT '[]'::jsonb NOT NULL,

    start_day_offset integer,

    end_day_offset integer,

    shift_group_key character varying(128),

    start_utc_offset_minutes integer,

    end_utc_offset_minutes integer,

    CONSTRAINT work_segments_breaks_snapshot_check CHECK ((jsonb_typeof(breaks_snapshot) = 'array'::text)),

    CONSTRAINT work_segments_compensation_rate_check CHECK (((compensation_rate >= 0.00) AND (compensation_rate <= 1.00))),

    CONSTRAINT work_segments_day_offsets_check CHECK (((start_day_offset IS NULL) OR ((start_day_offset >= 0) AND (end_day_offset >= start_day_offset)))),

    CONSTRAINT work_segments_gap_rate_check CHECK (((gap_compensation_rate_before >= 0.00) AND (gap_compensation_rate_before <= 1.00))),

    CONSTRAINT work_segments_schedule_values_check CHECK (((schedule_revision_id IS NULL) OR ((selected_shift_version_id IS NOT NULL) AND (scheduled_start_at IS NOT NULL) AND (scheduled_end_at IS NOT NULL) AND (scheduled_end_at > scheduled_start_at)))),

    CONSTRAINT work_segments_sort_order_check CHECK ((sort_order >= 0)),

    CONSTRAINT work_segments_source_check CHECK (((source)::text = ANY (ARRAY['roster'::text, 'pattern'::text, 'rule'::text, 'assignment_rule'::text, 'fixed'::text, 'manual'::text, 'auto'::text]))),

    CONSTRAINT work_segments_utc_offsets_check CHECK ((((start_utc_offset_minutes IS NULL) OR (((start_utc_offset_minutes >= '-840'::integer) AND (start_utc_offset_minutes <= 840)) AND ((start_utc_offset_minutes % 15) = 0))) AND ((end_utc_offset_minutes IS NULL) OR (((end_utc_offset_minutes >= '-840'::integer) AND (end_utc_offset_minutes <= 840)) AND ((end_utc_offset_minutes % 15) = 0)))))

);





--

-- Name: working_pattern_details; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.working_pattern_details (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    pattern_id uuid NOT NULL,

    schedule_day integer CONSTRAINT working_pattern_details_day_of_week_not_null NOT NULL,

    template_id uuid,

    CONSTRAINT working_pattern_details_schedule_day_check CHECK ((schedule_day >= 0))

);





--

-- Name: working_patterns; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.working_patterns (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

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





--

-- Name: attendance_staging_logs_default; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_default DEFAULT;





--

-- Name: attendance_staging_logs_y2026m01; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m01 FOR VALUES FROM ('2026-01-01 07:00:00+07') TO ('2026-02-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m02; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m02 FOR VALUES FROM ('2026-02-01 07:00:00+07') TO ('2026-03-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m03; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m03 FOR VALUES FROM ('2026-03-01 07:00:00+07') TO ('2026-04-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m04; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m04 FOR VALUES FROM ('2026-04-01 07:00:00+07') TO ('2026-05-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m05; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m05 FOR VALUES FROM ('2026-05-01 07:00:00+07') TO ('2026-06-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m06; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m06 FOR VALUES FROM ('2026-06-01 07:00:00+07') TO ('2026-07-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m07; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m07 FOR VALUES FROM ('2026-07-01 07:00:00+07') TO ('2026-08-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m08; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m08 FOR VALUES FROM ('2026-08-01 07:00:00+07') TO ('2026-09-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m09; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m09 FOR VALUES FROM ('2026-09-01 07:00:00+07') TO ('2026-10-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m10; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m10 FOR VALUES FROM ('2026-10-01 07:00:00+07') TO ('2026-11-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m11; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m11 FOR VALUES FROM ('2026-11-01 07:00:00+07') TO ('2026-12-01 07:00:00+07');





--

-- Name: attendance_staging_logs_y2026m12; Type: TABLE ATTACH; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m12 FOR VALUES FROM ('2026-12-01 07:00:00+07') TO ('2027-01-01 07:00:00+07');





--

-- Name: assignment_rules assignment_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.assignment_rules

    ADD CONSTRAINT assignment_rules_pkey PRIMARY KEY (id);





--

-- Name: attendance_adjustment_requests attendance_adjustment_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_adjustment_requests

    ADD CONSTRAINT attendance_adjustment_requests_pkey PRIMARY KEY (id);





--

-- Name: attendance_closure_days attendance_closure_days_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closure_days

    ADD CONSTRAINT attendance_closure_days_key UNIQUE (closure_id, employment_id, business_date);





--

-- Name: attendance_closure_days attendance_closure_days_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closure_days

    ADD CONSTRAINT attendance_closure_days_pkey PRIMARY KEY (id);





--

-- Name: attendance_closures attendance_closures_period_number_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closures

    ADD CONSTRAINT attendance_closures_period_number_key UNIQUE (period_id, closure_number);





--

-- Name: attendance_closures attendance_closures_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closures

    ADD CONSTRAINT attendance_closures_pkey PRIMARY KEY (id);





--

-- Name: attendance_daily_summaries attendance_daily_summaries_employment_id_date_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_daily_summaries

    ADD CONSTRAINT attendance_daily_summaries_employment_id_date_key UNIQUE (employment_id, date);





--

-- Name: attendance_daily_summaries attendance_daily_summaries_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_daily_summaries

    ADD CONSTRAINT attendance_daily_summaries_pkey PRIMARY KEY (id);





--

-- Name: attendance_import_jobs attendance_import_jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_import_jobs

    ADD CONSTRAINT attendance_import_jobs_pkey PRIMARY KEY (id);





--

-- Name: attendance_log_errors attendance_log_errors_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_log_errors

    ADD CONSTRAINT attendance_log_errors_pkey PRIMARY KEY (id);





--

-- Name: attendance_periods attendance_periods_period_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_periods

    ADD CONSTRAINT attendance_periods_period_key UNIQUE (employer_id, period_year, period_month);





--

-- Name: attendance_periods attendance_periods_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_periods

    ADD CONSTRAINT attendance_periods_pkey PRIMARY KEY (id);





--

-- Name: attendance_policies attendance_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_policies

    ADD CONSTRAINT attendance_policies_pkey PRIMARY KEY (id);





--

-- Name: attendance_policy_assignments attendance_policy_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_policy_assignments

    ADD CONSTRAINT attendance_policy_assignments_pkey PRIMARY KEY (id);





--

-- Name: attendance_policy_assignments no_overlapping_policy_assignments; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_policy_assignments

    ADD CONSTRAINT no_overlapping_policy_assignments EXCLUDE USING gist (target_id WITH =, target_type WITH =, tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamp with time zone)) WITH &&);





--

-- Name: attendance_policy_versions attendance_policy_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_policy_versions

    ADD CONSTRAINT attendance_policy_versions_pkey PRIMARY KEY (id);





--

-- Name: attendance_policy_versions no_overlapping_policy_versions; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_policy_versions

    ADD CONSTRAINT no_overlapping_policy_versions EXCLUDE USING gist (policy_id WITH =, tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamp with time zone)) WITH &&);





--

-- Name: attendance_source_pull_states attendance_source_pull_states_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_source_pull_states

    ADD CONSTRAINT attendance_source_pull_states_pkey PRIMARY KEY (id);





--

-- Name: attendance_source_pull_states attendance_source_pull_states_unique; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_source_pull_states

    ADD CONSTRAINT attendance_source_pull_states_unique UNIQUE (source_id);





--

-- Name: attendance_sources attendance_sources_organization_provider_unique; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_sources

    ADD CONSTRAINT attendance_sources_organization_provider_unique UNIQUE (provider_source_id);





--

-- Name: attendance_sources attendance_sources_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_sources

    ADD CONSTRAINT attendance_sources_pkey PRIMARY KEY (id);





--

-- Name: attendance_staging_logs attendance_staging_logs_pkey1; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs

    ADD CONSTRAINT attendance_staging_logs_pkey1 PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_default attendance_staging_logs_default_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_default

    ADD CONSTRAINT attendance_staging_logs_default_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m01 attendance_staging_logs_y2026m01_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m01

    ADD CONSTRAINT attendance_staging_logs_y2026m01_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m02 attendance_staging_logs_y2026m02_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m02

    ADD CONSTRAINT attendance_staging_logs_y2026m02_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m03 attendance_staging_logs_y2026m03_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m03

    ADD CONSTRAINT attendance_staging_logs_y2026m03_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m04 attendance_staging_logs_y2026m04_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m04

    ADD CONSTRAINT attendance_staging_logs_y2026m04_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m05 attendance_staging_logs_y2026m05_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m05

    ADD CONSTRAINT attendance_staging_logs_y2026m05_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m06 attendance_staging_logs_y2026m06_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m06

    ADD CONSTRAINT attendance_staging_logs_y2026m06_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m07 attendance_staging_logs_y2026m07_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m07

    ADD CONSTRAINT attendance_staging_logs_y2026m07_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m08 attendance_staging_logs_y2026m08_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m08

    ADD CONSTRAINT attendance_staging_logs_y2026m08_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m09 attendance_staging_logs_y2026m09_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m09

    ADD CONSTRAINT attendance_staging_logs_y2026m09_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m10 attendance_staging_logs_y2026m10_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m10

    ADD CONSTRAINT attendance_staging_logs_y2026m10_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m11 attendance_staging_logs_y2026m11_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m11

    ADD CONSTRAINT attendance_staging_logs_y2026m11_pkey PRIMARY KEY (id, created_at);





--

-- Name: attendance_staging_logs_y2026m12 attendance_staging_logs_y2026m12_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_staging_logs_y2026m12

    ADD CONSTRAINT attendance_staging_logs_y2026m12_pkey PRIMARY KEY (id, created_at);





--

-- Name: employee_shifts employee_shifts_new_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.employee_shifts

    ADD CONSTRAINT employee_shifts_new_pkey PRIMARY KEY (id);





--

-- Name: employee_working_patterns employee_working_patterns_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.employee_working_patterns

    ADD CONSTRAINT employee_working_patterns_pkey PRIMARY KEY (id);





--

-- Name: external_attendance_events external_attendance_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.external_attendance_events

    ADD CONSTRAINT external_attendance_events_pkey PRIMARY KEY (id);





--

-- Name: external_attendance_events external_attendance_events_source_identity_unique; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.external_attendance_events

    ADD CONSTRAINT external_attendance_events_source_identity_unique UNIQUE (source_id, external_event_id);





--

-- Name: holidays holidays_organization_date_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.holidays

    ADD CONSTRAINT holidays_organization_date_key UNIQUE (date);





--

-- Name: holidays holidays_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.holidays

    ADD CONSTRAINT holidays_pkey PRIMARY KEY (id);





--

-- Name: payroll_periods payroll_periods_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_periods

    ADD CONSTRAINT payroll_periods_pkey PRIMARY KEY (id);





--

-- Name: payroll_periods payroll_periods_unique_period; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_periods

    ADD CONSTRAINT payroll_periods_unique_period UNIQUE (employer_id, period_year, period_month);





--

-- Name: raw_attendance_logs raw_attendance_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.raw_attendance_logs

    ADD CONSTRAINT raw_attendance_logs_pkey PRIMARY KEY (id);





--

-- Name: raw_attendance_logs unique_raw_log; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.raw_attendance_logs

    ADD CONSTRAINT unique_raw_log UNIQUE (employment_id, punch_time, punch, status);





--

-- Name: schedule_day_revisions schedule_day_revisions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_day_revisions

    ADD CONSTRAINT schedule_day_revisions_pkey PRIMARY KEY (id);





--

-- Name: schedule_day_revisions schedule_day_revisions_revision_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_day_revisions

    ADD CONSTRAINT schedule_day_revisions_revision_key UNIQUE (employment_id, business_date, revision_number);





--

-- Name: schedule_legacy_mutation_evidence schedule_legacy_mutation_evidence_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_legacy_mutation_evidence

    ADD CONSTRAINT schedule_legacy_mutation_evidence_pkey PRIMARY KEY (id);





--

-- Name: schedule_office_policies schedule_office_policies_approval_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_office_policies

    ADD CONSTRAINT schedule_office_policies_approval_key UNIQUE (approval_instance_id);





--

-- Name: schedule_office_policies schedule_office_policies_effective_no_overlap; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_office_policies

    ADD CONSTRAINT schedule_office_policies_effective_no_overlap EXCLUDE USING gist (employer_id WITH =, daterange(effective_from, COALESCE((effective_to + 1), 'infinity'::date), '[)'::text) WITH &&) WHERE (((status)::text = 'active'::text));





--

-- Name: schedule_office_policies schedule_office_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_office_policies

    ADD CONSTRAINT schedule_office_policies_pkey PRIMARY KEY (id);





--

-- Name: schedule_previews schedule_previews_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_previews

    ADD CONSTRAINT schedule_previews_pkey PRIMARY KEY (id);





--

-- Name: schedule_publication_batches schedule_publication_batches_idempotency_key_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_publication_batches

    ADD CONSTRAINT schedule_publication_batches_idempotency_key_key UNIQUE (idempotency_key);





--

-- Name: schedule_publication_batches schedule_publication_batches_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_publication_batches

    ADD CONSTRAINT schedule_publication_batches_pkey PRIMARY KEY (id);





--

-- Name: schedule_shadow_mismatches schedule_shadow_mismatches_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_shadow_mismatches

    ADD CONSTRAINT schedule_shadow_mismatches_pkey PRIMARY KEY (id);





--

-- Name: shift_audit_log shift_audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_audit_log

    ADD CONSTRAINT shift_audit_log_pkey PRIMARY KEY (id);





--

-- Name: shift_default_intervals shift_default_intervals_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_default_intervals

    ADD CONSTRAINT shift_default_intervals_pkey PRIMARY KEY (id);





--

-- Name: shift_default_intervals shift_default_intervals_version_id_sort_order_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_default_intervals

    ADD CONSTRAINT shift_default_intervals_version_id_sort_order_key UNIQUE (version_id, sort_order);





--

-- Name: shift_rosters shift_rosters_employment_id_date_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_rosters

    ADD CONSTRAINT shift_rosters_employment_id_date_key UNIQUE (employment_id, date);





--

-- Name: shift_rosters shift_rosters_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_rosters

    ADD CONSTRAINT shift_rosters_pkey PRIMARY KEY (id);





--

-- Name: shift_swap_requests shift_swap_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_swap_requests

    ADD CONSTRAINT shift_swap_requests_pkey PRIMARY KEY (id);





--

-- Name: shift_templates shift_templates_organization_name_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_templates

    ADD CONSTRAINT shift_templates_organization_name_key UNIQUE (name);





--

-- Name: shift_templates shift_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_templates

    ADD CONSTRAINT shift_templates_pkey PRIMARY KEY (id);





--

-- Name: shift_versions shift_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_versions

    ADD CONSTRAINT shift_versions_pkey PRIMARY KEY (id);





--

-- Name: shifts shifts_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shifts

    ADD CONSTRAINT shifts_pkey PRIMARY KEY (id);





--

-- Name: time_entries time_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.time_entries

    ADD CONSTRAINT time_entries_pkey PRIMARY KEY (id);





--

-- Name: work_segments work_segments_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_segments

    ADD CONSTRAINT work_segments_pkey PRIMARY KEY (id);





--

-- Name: working_pattern_details working_pattern_details_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.working_pattern_details

    ADD CONSTRAINT working_pattern_details_pkey PRIMARY KEY (id);





--

-- Name: working_patterns working_patterns_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.working_patterns

    ADD CONSTRAINT working_patterns_pkey PRIMARY KEY (id);





--

-- Name: attendance_closure_days_employment_date_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_closure_days_employment_date_idx ON public.attendance_closure_days USING btree (employment_id, business_date);





--

-- Name: attendance_closure_days_revision_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_closure_days_revision_idx ON public.attendance_closure_days USING btree (schedule_revision_id);





--

-- Name: attendance_closures_employer_closed_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_closures_employer_closed_at_idx ON public.attendance_closures USING btree (employer_id, closed_at DESC);





--

-- Name: attendance_periods_employer_period_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_periods_employer_period_idx ON public.attendance_periods USING btree (employer_id, period_year, period_month);





--

-- Name: attendance_policies_organization_default_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX attendance_policies_organization_default_idx ON public.attendance_policies USING btree ((true)) WHERE (is_default = true);





--

-- Name: attendance_staging_logs_default_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_default_fingerprint_idx ON public.attendance_staging_logs_default USING btree (fingerprint);





--

-- Name: attendance_staging_logs_default_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_default_status_idx ON public.attendance_staging_logs_default USING btree (status);





--

-- Name: attendance_staging_logs_y2026m01_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m01_fingerprint_idx ON public.attendance_staging_logs_y2026m01 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m01_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m01_status_idx ON public.attendance_staging_logs_y2026m01 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m02_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m02_fingerprint_idx ON public.attendance_staging_logs_y2026m02 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m02_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m02_status_idx ON public.attendance_staging_logs_y2026m02 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m03_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m03_fingerprint_idx ON public.attendance_staging_logs_y2026m03 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m03_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m03_status_idx ON public.attendance_staging_logs_y2026m03 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m04_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m04_fingerprint_idx ON public.attendance_staging_logs_y2026m04 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m04_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m04_status_idx ON public.attendance_staging_logs_y2026m04 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m05_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m05_fingerprint_idx ON public.attendance_staging_logs_y2026m05 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m05_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m05_status_idx ON public.attendance_staging_logs_y2026m05 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m06_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m06_fingerprint_idx ON public.attendance_staging_logs_y2026m06 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m06_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m06_status_idx ON public.attendance_staging_logs_y2026m06 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m07_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m07_fingerprint_idx ON public.attendance_staging_logs_y2026m07 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m07_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m07_status_idx ON public.attendance_staging_logs_y2026m07 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m08_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m08_fingerprint_idx ON public.attendance_staging_logs_y2026m08 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m08_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m08_status_idx ON public.attendance_staging_logs_y2026m08 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m09_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m09_fingerprint_idx ON public.attendance_staging_logs_y2026m09 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m09_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m09_status_idx ON public.attendance_staging_logs_y2026m09 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m10_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m10_fingerprint_idx ON public.attendance_staging_logs_y2026m10 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m10_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m10_status_idx ON public.attendance_staging_logs_y2026m10 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m11_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m11_fingerprint_idx ON public.attendance_staging_logs_y2026m11 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m11_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m11_status_idx ON public.attendance_staging_logs_y2026m11 USING btree (status);





--

-- Name: attendance_staging_logs_y2026m12_fingerprint_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m12_fingerprint_idx ON public.attendance_staging_logs_y2026m12 USING btree (fingerprint);





--

-- Name: attendance_staging_logs_y2026m12_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX attendance_staging_logs_y2026m12_status_idx ON public.attendance_staging_logs_y2026m12 USING btree (status);





--

-- Name: idx_assignment_rules_pattern_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_assignment_rules_pattern_id ON public.assignment_rules USING btree (pattern_id);





--

-- Name: idx_assignment_rules_priority; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_assignment_rules_priority ON public.assignment_rules USING btree (priority DESC);





--

-- Name: idx_attendance_adj_time_entry_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_adj_time_entry_id ON public.attendance_adjustment_requests USING btree (time_entry_id);





--

-- Name: idx_attendance_adjustment_requests_employment_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_adjustment_requests_employment_id ON public.attendance_adjustment_requests USING btree (employment_id);





--

-- Name: idx_attendance_adjustment_requests_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_adjustment_requests_status ON public.attendance_adjustment_requests USING btree (status);





--

-- Name: idx_attendance_daily_summaries_employment_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_daily_summaries_employment_id ON public.attendance_daily_summaries USING btree (employment_id);





--

-- Name: idx_attendance_import_jobs_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_import_jobs_status ON public.attendance_import_jobs USING btree (status);





--

-- Name: idx_attendance_policy_assignments_target; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_policy_assignments_target ON public.attendance_policy_assignments USING btree (target_type, target_id);





--

-- Name: idx_attendance_policy_versions_rules; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_policy_versions_rules ON public.attendance_policy_versions USING gin (rules);





--

-- Name: idx_attendance_source_pull_states_lease; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_source_pull_states_lease ON public.attendance_source_pull_states USING btree (status, lease_expires_at);





--

-- Name: idx_attendance_sources_organization_enabled; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_attendance_sources_organization_enabled ON public.attendance_sources USING btree (is_enabled);





--

-- Name: idx_daily_summaries_organization_date; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_daily_summaries_organization_date ON public.attendance_daily_summaries USING btree (date);





--

-- Name: idx_daily_summaries_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_daily_summaries_status ON public.attendance_daily_summaries USING btree (ot_status);





--

-- Name: idx_employee_shifts_dates; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_employee_shifts_dates ON public.employee_shifts USING btree (start_date, end_date);





--

-- Name: idx_employee_shifts_employment_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_employee_shifts_employment_id ON public.employee_shifts USING btree (employment_id);





--

-- Name: idx_employee_working_patterns_employment_date; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_employee_working_patterns_employment_date ON public.employee_working_patterns USING btree (employment_id, start_date);





--

-- Name: idx_employee_working_patterns_pattern_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_employee_working_patterns_pattern_id ON public.employee_working_patterns USING btree (pattern_id);





--

-- Name: idx_errors_fingerprint; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX idx_errors_fingerprint ON public.attendance_log_errors USING btree (fingerprint) WHERE (fingerprint IS NOT NULL);





--

-- Name: idx_external_attendance_events_cleanup; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_external_attendance_events_cleanup ON public.external_attendance_events USING btree (received_at);





--

-- Name: idx_external_attendance_events_source_time; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_external_attendance_events_source_time ON public.external_attendance_events USING btree (source_id, occurred_at);





--

-- Name: idx_holidays_lookup; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_holidays_lookup ON public.holidays USING btree (date);





--

-- Name: idx_holidays_organization_location_date; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_holidays_organization_location_date ON public.holidays USING btree (location_id, date);





--

-- Name: idx_raw_attendance_logs_employment_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_raw_attendance_logs_employment_id ON public.raw_attendance_logs USING btree (employment_id);





--

-- Name: idx_raw_logs_punch_time; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_raw_logs_punch_time ON public.raw_attendance_logs USING btree (punch_time);





--

-- Name: idx_shift_audit_log_template; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_audit_log_template ON public.shift_audit_log USING btree (template_id);





--

-- Name: idx_shift_audit_log_version; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_audit_log_version ON public.shift_audit_log USING btree (version_id);





--

-- Name: idx_shift_rosters_employment_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_rosters_employment_id ON public.shift_rosters USING btree (employment_id);

CREATE INDEX idx_shift_rosters_version_lookup ON public.shift_rosters USING btree (employment_id, date, shift_version_id);





--

-- Name: idx_shift_swap_date; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_swap_date ON public.shift_swap_requests USING btree (swap_date);





--

-- Name: idx_shift_swap_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_swap_status ON public.shift_swap_requests USING btree (status);





--

-- Name: idx_shift_swap_ws_requester; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_swap_ws_requester ON public.shift_swap_requests USING btree (requester_id);





--

-- Name: idx_shift_swap_ws_target; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_swap_ws_target ON public.shift_swap_requests USING btree (target_id);





--

-- Name: idx_shift_versions_template; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_versions_template ON public.shift_versions USING btree (template_id);





--

-- Name: idx_shift_versions_template_effective; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shift_versions_template_effective ON public.shift_versions USING btree (template_id, effective_from DESC, effective_to);





--

-- Name: idx_shifts_active; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shifts_active ON public.shifts USING btree (is_active);





--

-- Name: idx_shifts_employer_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shifts_employer_id ON public.shifts USING btree (employer_id);





--

-- Name: idx_shifts_location_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shifts_location_id ON public.shifts USING btree (location_id);





--

-- Name: idx_shifts_parent_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_shifts_parent_id ON public.shifts USING btree (parent_shift_id);





--

-- Name: idx_staging_logs_fingerprint; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_staging_logs_fingerprint ON ONLY public.attendance_staging_logs USING btree (fingerprint);





--

-- Name: idx_staging_logs_organization_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_staging_logs_organization_status ON ONLY public.attendance_staging_logs USING btree (status);





--

-- Name: idx_time_entries_employment_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_time_entries_employment_id ON public.time_entries USING btree (employment_id);





--

-- Name: idx_time_entries_employment_punch_in; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_time_entries_employment_punch_in ON public.time_entries USING btree (employment_id, punch_in_time);





--

-- Name: idx_time_entries_punch_in_time; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_time_entries_punch_in_time ON public.time_entries USING btree (punch_in_time);





--

-- Name: idx_uq_shift_swap_pair_date; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX idx_uq_shift_swap_pair_date ON public.shift_swap_requests USING btree (requester_id, target_id, swap_date) WHERE ((status)::text <> ALL (ARRAY[('cancelled'::character varying)::text, ('rejected'::character varying)::text]));





--

-- Name: idx_work_segments_date_range; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_work_segments_date_range ON public.work_segments USING btree (employment_id, date, sort_order);





--

-- Name: idx_work_segments_employment_date; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_work_segments_employment_date ON public.work_segments USING btree (employment_id, date);





--

-- Name: idx_work_segments_employment_date_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_work_segments_employment_date_status ON public.work_segments USING btree (employment_id, date, segment_status);





--

-- Name: idx_work_segments_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_work_segments_status ON public.work_segments USING btree (segment_status);





--

-- Name: payroll_periods_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payroll_periods_status_idx ON public.payroll_periods USING btree (status, period_year, period_month);





--

-- Name: schedule_day_revisions_current_uq; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX schedule_day_revisions_current_uq ON public.schedule_day_revisions USING btree (employment_id, business_date) WHERE ((state)::text = ANY (ARRAY[('published'::character varying)::text, ('explicit_off'::character varying)::text]));





--

-- Name: schedule_day_revisions_employer_date_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_day_revisions_employer_date_idx ON public.schedule_day_revisions USING btree (employer_id, business_date);





--

-- Name: schedule_day_revisions_employment_date_revision_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_day_revisions_employment_date_revision_idx ON public.schedule_day_revisions USING btree (employment_id, business_date, revision_number DESC);





--

-- Name: schedule_day_revisions_legacy_uq; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX schedule_day_revisions_legacy_uq ON public.schedule_day_revisions USING btree (employment_id, business_date) WHERE ((state)::text = 'legacy_unverified'::text);





--

-- Name: schedule_legacy_mutation_evidence_scope_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_legacy_mutation_evidence_scope_idx ON public.schedule_legacy_mutation_evidence USING btree (employment_id, business_date, created_at DESC);





--

-- Name: schedule_office_policies_approval_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_office_policies_approval_idx ON public.schedule_office_policies USING btree (approval_instance_id, status);





--

-- Name: schedule_office_policies_employer_effective_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_office_policies_employer_effective_idx ON public.schedule_office_policies USING btree (employer_id, effective_from, effective_to, status);





--

-- Name: schedule_previews_expires_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_previews_expires_at_idx ON public.schedule_previews USING btree (expires_at);





--

-- Name: schedule_previews_requested_by_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_previews_requested_by_created_at_idx ON public.schedule_previews USING btree (requested_by, created_at DESC);





--

-- Name: schedule_publication_batches_created_at_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_publication_batches_created_at_idx ON public.schedule_publication_batches USING btree (created_at DESC, id DESC);





--

-- Name: schedule_publication_batches_range_mode_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_publication_batches_range_mode_idx ON public.schedule_publication_batches USING btree (range_start, range_end, mode);





--

-- Name: schedule_shadow_mismatches_employer_date_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_shadow_mismatches_employer_date_idx ON public.schedule_shadow_mismatches USING btree (employer_id, business_date, status);





--

-- Name: schedule_shadow_mismatches_employment_date_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX schedule_shadow_mismatches_employment_date_idx ON public.schedule_shadow_mismatches USING btree (employment_id, business_date, last_seen_at DESC);





--

-- Name: schedule_shadow_mismatches_open_key; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX schedule_shadow_mismatches_open_key ON public.schedule_shadow_mismatches USING btree (employer_id, employment_id, business_date, mismatch_kind) WHERE ((status)::text = 'open'::text);





--

-- Name: shift_swap_requests_official_revisions_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX shift_swap_requests_official_revisions_idx ON public.shift_swap_requests USING btree (requester_revision_id, target_revision_id) WHERE ((requester_revision_id IS NOT NULL) OR (target_revision_id IS NOT NULL));





--

-- Name: uq_holidays_organization_date; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX uq_holidays_organization_date ON public.holidays USING btree (date, name) WHERE (deleted_at IS NULL);





--

-- Name: uq_working_patterns_name; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX uq_working_patterns_name ON public.working_patterns USING btree (lower(name)) WHERE (deleted_at IS NULL);





--

-- Name: work_segments_legacy_interval_uq; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX work_segments_legacy_interval_uq ON public.work_segments USING btree (employment_id, date, interval_id) WHERE ((schedule_revision_id IS NULL) AND (interval_id IS NOT NULL));





--

-- Name: work_segments_official_interval_uq; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX work_segments_official_interval_uq ON public.work_segments USING btree (schedule_revision_id, interval_id) WHERE ((schedule_revision_id IS NOT NULL) AND (interval_id IS NOT NULL));





--

-- Name: work_segments_schedule_revision_sort_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX work_segments_schedule_revision_sort_idx ON public.work_segments USING btree (schedule_revision_id, sort_order, id);





--

-- Name: attendance_staging_logs_default_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_default_fingerprint_idx;





--

-- Name: attendance_staging_logs_default_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_default_pkey;





--

-- Name: attendance_staging_logs_default_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_default_status_idx;





--

-- Name: attendance_staging_logs_y2026m01_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m01_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m01_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m01_pkey;





--

-- Name: attendance_staging_logs_y2026m01_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m01_status_idx;





--

-- Name: attendance_staging_logs_y2026m02_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m02_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m02_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m02_pkey;





--

-- Name: attendance_staging_logs_y2026m02_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m02_status_idx;





--

-- Name: attendance_staging_logs_y2026m03_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m03_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m03_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m03_pkey;





--

-- Name: attendance_staging_logs_y2026m03_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m03_status_idx;





--

-- Name: attendance_staging_logs_y2026m04_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m04_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m04_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m04_pkey;





--

-- Name: attendance_staging_logs_y2026m04_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m04_status_idx;





--

-- Name: attendance_staging_logs_y2026m05_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m05_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m05_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m05_pkey;





--

-- Name: attendance_staging_logs_y2026m05_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m05_status_idx;





--

-- Name: attendance_staging_logs_y2026m06_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m06_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m06_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m06_pkey;





--

-- Name: attendance_staging_logs_y2026m06_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m06_status_idx;





--

-- Name: attendance_staging_logs_y2026m07_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m07_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m07_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m07_pkey;





--

-- Name: attendance_staging_logs_y2026m07_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m07_status_idx;





--

-- Name: attendance_staging_logs_y2026m08_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m08_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m08_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m08_pkey;





--

-- Name: attendance_staging_logs_y2026m08_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m08_status_idx;





--

-- Name: attendance_staging_logs_y2026m09_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m09_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m09_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m09_pkey;





--

-- Name: attendance_staging_logs_y2026m09_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m09_status_idx;





--

-- Name: attendance_staging_logs_y2026m10_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m10_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m10_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m10_pkey;





--

-- Name: attendance_staging_logs_y2026m10_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m10_status_idx;





--

-- Name: attendance_staging_logs_y2026m11_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m11_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m11_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m11_pkey;





--

-- Name: attendance_staging_logs_y2026m11_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m11_status_idx;





--

-- Name: attendance_staging_logs_y2026m12_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m12_fingerprint_idx;





--

-- Name: attendance_staging_logs_y2026m12_pkey; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m12_pkey;





--

-- Name: attendance_staging_logs_y2026m12_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -

--



ALTER INDEX public.idx_staging_logs_organization_status ATTACH PARTITION public.attendance_staging_logs_y2026m12_status_idx;





--

-- Name: assignment_rules trg_guard_legacy_assignment_rules; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_guard_legacy_assignment_rules BEFORE INSERT OR DELETE OR UPDATE ON public.assignment_rules FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();





--

-- Name: assignment_rules trg_roster_cache_invalidation_assignment_rules; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_assignment_rules AFTER INSERT OR DELETE OR UPDATE ON public.assignment_rules FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');





--

-- Name: assignment_rules trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.assignment_rules FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: attendance_adjustment_requests trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_adjustment_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: attendance_closure_days trg_attendance_closure_days_immutable; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_attendance_closure_days_immutable BEFORE DELETE OR UPDATE ON public.attendance_closure_days FOR EACH ROW EXECUTE FUNCTION public.guard_attendance_closure_immutable();





--

-- Name: attendance_closures trg_attendance_closures_immutable; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_attendance_closures_immutable BEFORE DELETE OR UPDATE ON public.attendance_closures FOR EACH ROW EXECUTE FUNCTION public.guard_attendance_closure_immutable();





--

-- Name: attendance_daily_summaries trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_daily_summaries FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: attendance_import_jobs trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_import_jobs FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: attendance_periods trg_attendance_periods_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_attendance_periods_updated_at BEFORE UPDATE ON public.attendance_periods FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: attendance_policies trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_policies FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: attendance_policy_assignments trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: attendance_policy_assignments trg_validate_target_fk; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_validate_target_fk BEFORE INSERT OR UPDATE OF target_type, target_id ON public.attendance_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_target_fk();





--

-- Name: employee_shifts trg_guard_legacy_employee_shifts; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_guard_legacy_employee_shifts BEFORE INSERT OR DELETE OR UPDATE ON public.employee_shifts FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();





--

-- Name: employee_shifts trg_roster_cache_invalidation_employee_shifts; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_employee_shifts AFTER INSERT OR DELETE OR UPDATE ON public.employee_shifts FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('employment');





--

-- Name: employee_shifts trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_shifts FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: employee_working_patterns trg_guard_legacy_employee_working_patterns; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_guard_legacy_employee_working_patterns BEFORE INSERT OR DELETE OR UPDATE ON public.employee_working_patterns FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();





--

-- Name: employee_working_patterns trg_roster_cache_invalidation_employee_working_patterns; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_employee_working_patterns AFTER INSERT OR DELETE OR UPDATE ON public.employee_working_patterns FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');





--

-- Name: employee_working_patterns trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_working_patterns FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: holidays trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.holidays FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: payroll_periods trg_payroll_periods_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_payroll_periods_updated_at BEFORE UPDATE ON public.payroll_periods FOR EACH ROW EXECUTE FUNCTION public.fn_payroll_periods_updated_at();





--

-- Name: schedule_day_revisions trg_schedule_day_revision_guard; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_schedule_day_revision_guard BEFORE DELETE OR UPDATE ON public.schedule_day_revisions FOR EACH ROW EXECUTE FUNCTION public.guard_schedule_day_revision();





--

-- Name: schedule_publication_batches trg_schedule_publication_batches_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_schedule_publication_batches_updated_at BEFORE UPDATE ON public.schedule_publication_batches FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: shift_default_intervals trg_roster_cache_invalidation_shift_default_intervals; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_shift_default_intervals AFTER INSERT OR DELETE OR UPDATE ON public.shift_default_intervals FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('shift_interval');





--

-- Name: shift_default_intervals trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_default_intervals FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: shift_rosters trg_guard_legacy_shift_rosters; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_guard_legacy_shift_rosters BEFORE INSERT OR DELETE OR UPDATE ON public.shift_rosters FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();





--

-- Name: shift_rosters trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_rosters FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: shift_swap_requests trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_swap_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: shift_templates trg_guard_legacy_shift_templates; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_guard_legacy_shift_templates BEFORE INSERT OR DELETE OR UPDATE ON public.shift_templates FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();





--

-- Name: shift_templates trg_roster_cache_invalidation_shift_templates; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_shift_templates AFTER INSERT OR DELETE OR UPDATE ON public.shift_templates FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');





--

-- Name: shift_versions trg_guard_legacy_shift_versions; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_guard_legacy_shift_versions BEFORE INSERT OR DELETE OR UPDATE ON public.shift_versions FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();





--

-- Name: shift_versions trg_roster_cache_invalidation_shift_versions; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_shift_versions AFTER INSERT OR DELETE OR UPDATE ON public.shift_versions FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');





--

-- Name: shifts trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shifts FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: work_segments trg_roster_cache_invalidation_work_segments; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_work_segments AFTER INSERT OR DELETE OR UPDATE ON public.work_segments FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');





--

-- Name: work_segments trg_schedule_work_segment_guard; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_schedule_work_segment_guard BEFORE INSERT OR DELETE OR UPDATE ON public.work_segments FOR EACH ROW EXECUTE FUNCTION public.guard_schedule_work_segment();





--

-- Name: work_segments trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.work_segments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: working_pattern_details trg_guard_legacy_working_pattern_details; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_guard_legacy_working_pattern_details BEFORE INSERT OR DELETE OR UPDATE ON public.working_pattern_details FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();





--

-- Name: working_pattern_details trg_roster_cache_invalidation_working_pattern_details; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_working_pattern_details AFTER INSERT OR DELETE OR UPDATE ON public.working_pattern_details FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('working_pattern');





--

-- Name: working_patterns trg_guard_legacy_working_patterns; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_guard_legacy_working_patterns BEFORE INSERT OR DELETE OR UPDATE ON public.working_patterns FOR EACH ROW EXECUTE FUNCTION public.guard_legacy_schedule_mutation();





--

-- Name: working_patterns trg_roster_cache_invalidation_working_patterns; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_roster_cache_invalidation_working_patterns AFTER INSERT OR DELETE OR UPDATE ON public.working_patterns FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');





--

-- Name: working_patterns trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.working_patterns FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: assignment_rules assignment_rules_pattern_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.assignment_rules

    ADD CONSTRAINT assignment_rules_pattern_id_fkey FOREIGN KEY (pattern_id) REFERENCES public.working_patterns(id) ON DELETE CASCADE;





--

-- Name: attendance_adjustment_requests attendance_adjustment_requests_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_adjustment_requests

    ADD CONSTRAINT attendance_adjustment_requests_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: attendance_adjustment_requests attendance_adjustment_requests_time_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_adjustment_requests

    ADD CONSTRAINT attendance_adjustment_requests_time_entry_id_fkey FOREIGN KEY (time_entry_id) REFERENCES public.time_entries(id) ON DELETE SET NULL;





--

-- Name: attendance_closure_days attendance_closure_days_closure_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closure_days

    ADD CONSTRAINT attendance_closure_days_closure_fkey FOREIGN KEY (closure_id) REFERENCES public.attendance_closures(id) ON DELETE RESTRICT;





--

-- Name: attendance_closure_days attendance_closure_days_employer_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closure_days

    ADD CONSTRAINT attendance_closure_days_employer_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;





--

-- Name: attendance_closure_days attendance_closure_days_employment_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closure_days

    ADD CONSTRAINT attendance_closure_days_employment_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: attendance_closure_days attendance_closure_days_period_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closure_days

    ADD CONSTRAINT attendance_closure_days_period_fkey FOREIGN KEY (period_id) REFERENCES public.attendance_periods(id) ON DELETE RESTRICT;





--

-- Name: attendance_closure_days attendance_closure_days_revision_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closure_days

    ADD CONSTRAINT attendance_closure_days_revision_fkey FOREIGN KEY (schedule_revision_id) REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT;





--

-- Name: attendance_closures attendance_closures_closed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closures

    ADD CONSTRAINT attendance_closures_closed_by_fkey FOREIGN KEY (closed_by) REFERENCES public.users(id) ON DELETE SET NULL;





--

-- Name: attendance_closures attendance_closures_employer_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closures

    ADD CONSTRAINT attendance_closures_employer_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;





--

-- Name: attendance_closures attendance_closures_period_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_closures

    ADD CONSTRAINT attendance_closures_period_fkey FOREIGN KEY (period_id) REFERENCES public.attendance_periods(id) ON DELETE RESTRICT;





--

-- Name: attendance_daily_summaries attendance_daily_summaries_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_daily_summaries

    ADD CONSTRAINT attendance_daily_summaries_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: attendance_periods attendance_periods_current_closure_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_periods

    ADD CONSTRAINT attendance_periods_current_closure_fkey FOREIGN KEY (current_closure_id) REFERENCES public.attendance_closures(id) ON DELETE RESTRICT;





--

-- Name: attendance_periods attendance_periods_employer_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_periods

    ADD CONSTRAINT attendance_periods_employer_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;





--

-- Name: attendance_policy_assignments attendance_policy_assignments_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_policy_assignments

    ADD CONSTRAINT attendance_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.attendance_policies(id) ON DELETE CASCADE;





--

-- Name: attendance_policy_versions attendance_policy_versions_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.attendance_policy_versions

    ADD CONSTRAINT attendance_policy_versions_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.attendance_policies(id) ON DELETE CASCADE;





--

-- Name: departments departments_default_working_pattern_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.departments

    ADD CONSTRAINT departments_default_working_pattern_id_fkey FOREIGN KEY (default_working_pattern_id) REFERENCES public.working_patterns(id) ON DELETE SET NULL;





--

-- Name: employee_shifts employee_shifts_new_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.employee_shifts

    ADD CONSTRAINT employee_shifts_new_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;





--

-- Name: employee_shifts employee_shifts_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.employee_shifts

    ADD CONSTRAINT employee_shifts_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.shift_templates(id) ON DELETE CASCADE;





--

-- Name: employee_working_patterns employee_working_patterns_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.employee_working_patterns

    ADD CONSTRAINT employee_working_patterns_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;





--

-- Name: employee_working_patterns employee_working_patterns_pattern_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.employee_working_patterns

    ADD CONSTRAINT employee_working_patterns_pattern_id_fkey FOREIGN KEY (pattern_id) REFERENCES public.working_patterns(id) ON DELETE CASCADE;





--

-- Name: holidays holidays_leave_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.holidays

    ADD CONSTRAINT holidays_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE SET NULL;





--

-- Name: holidays holidays_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.holidays

    ADD CONSTRAINT holidays_location_id_fkey FOREIGN KEY (location_id) REFERENCES public.work_locations(id) ON DELETE SET NULL;





--

-- Name: overtime_request_employees overtime_request_employees_matched_summary_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.overtime_request_employees

    ADD CONSTRAINT overtime_request_employees_matched_summary_id_fkey FOREIGN KEY (matched_summary_id) REFERENCES public.attendance_daily_summaries(id);





--

-- Name: payroll_periods payroll_periods_employer_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_periods

    ADD CONSTRAINT payroll_periods_employer_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id);





--

-- Name: payroll_periods payroll_periods_locked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_periods

    ADD CONSTRAINT payroll_periods_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);





--

-- Name: payroll_periods payroll_periods_reopened_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_periods

    ADD CONSTRAINT payroll_periods_reopened_by_fkey FOREIGN KEY (reopened_by) REFERENCES public.users(id);





--

-- Name: raw_attendance_logs raw_attendance_logs_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.raw_attendance_logs

    ADD CONSTRAINT raw_attendance_logs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: schedule_day_revisions schedule_day_revisions_batch_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_day_revisions

    ADD CONSTRAINT schedule_day_revisions_batch_fkey FOREIGN KEY (publication_batch_id) REFERENCES public.schedule_publication_batches(id) ON DELETE RESTRICT;





--

-- Name: schedule_day_revisions schedule_day_revisions_employer_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_day_revisions

    ADD CONSTRAINT schedule_day_revisions_employer_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;





--

-- Name: schedule_day_revisions schedule_day_revisions_employment_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_day_revisions

    ADD CONSTRAINT schedule_day_revisions_employment_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: schedule_day_revisions schedule_day_revisions_previous_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_day_revisions

    ADD CONSTRAINT schedule_day_revisions_previous_fkey FOREIGN KEY (previous_revision_id) REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT;





--

-- Name: schedule_day_revisions schedule_day_revisions_published_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_day_revisions

    ADD CONSTRAINT schedule_day_revisions_published_by_fkey FOREIGN KEY (published_by) REFERENCES public.users(id) ON DELETE SET NULL;





--

-- Name: schedule_legacy_mutation_evidence schedule_legacy_mutation_evidence_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_legacy_mutation_evidence

    ADD CONSTRAINT schedule_legacy_mutation_evidence_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: schedule_office_policies schedule_office_policies_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_office_policies

    ADD CONSTRAINT schedule_office_policies_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE SET NULL;





--

-- Name: schedule_office_policies schedule_office_policies_employer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_office_policies

    ADD CONSTRAINT schedule_office_policies_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;





--

-- Name: schedule_previews schedule_previews_requested_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_previews

    ADD CONSTRAINT schedule_previews_requested_by_fkey FOREIGN KEY (requested_by) REFERENCES public.users(id) ON DELETE RESTRICT;





--

-- Name: schedule_publication_batches schedule_publication_batches_requested_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_publication_batches

    ADD CONSTRAINT schedule_publication_batches_requested_by_fkey FOREIGN KEY (requested_by) REFERENCES public.users(id) ON DELETE RESTRICT;





--

-- Name: schedule_shadow_mismatches schedule_shadow_mismatches_employer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_shadow_mismatches

    ADD CONSTRAINT schedule_shadow_mismatches_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;





--

-- Name: schedule_shadow_mismatches schedule_shadow_mismatches_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.schedule_shadow_mismatches

    ADD CONSTRAINT schedule_shadow_mismatches_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: shift_default_intervals shift_default_intervals_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_default_intervals

    ADD CONSTRAINT shift_default_intervals_version_id_fkey FOREIGN KEY (version_id) REFERENCES public.shift_versions(id) ON DELETE CASCADE;





--

-- Name: shift_rosters shift_rosters_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_rosters

    ADD CONSTRAINT shift_rosters_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: shift_rosters shift_rosters_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_rosters

    ADD CONSTRAINT shift_rosters_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.shift_templates(id) ON DELETE SET NULL;



-- Name: shift_rosters shift_rosters_shift_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.shift_rosters

    ADD CONSTRAINT shift_rosters_shift_version_id_fkey FOREIGN KEY (shift_version_id) REFERENCES public.shift_versions(id) ON DELETE RESTRICT;





--

-- Name: shift_swap_requests shift_swap_requests_requester_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_swap_requests

    ADD CONSTRAINT shift_swap_requests_requester_id_fkey FOREIGN KEY (requester_id) REFERENCES public.employees(id) ON DELETE CASCADE;





--

-- Name: shift_swap_requests shift_swap_requests_requester_revision_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_swap_requests

    ADD CONSTRAINT shift_swap_requests_requester_revision_fkey FOREIGN KEY (requester_revision_id) REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT;





--

-- Name: shift_swap_requests shift_swap_requests_requester_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_swap_requests

    ADD CONSTRAINT shift_swap_requests_requester_template_id_fkey FOREIGN KEY (requester_template_id) REFERENCES public.shift_templates(id) ON DELETE SET NULL;





--

-- Name: shift_swap_requests shift_swap_requests_target_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_swap_requests

    ADD CONSTRAINT shift_swap_requests_target_id_fkey FOREIGN KEY (target_id) REFERENCES public.employees(id) ON DELETE CASCADE;





--

-- Name: shift_swap_requests shift_swap_requests_target_revision_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_swap_requests

    ADD CONSTRAINT shift_swap_requests_target_revision_fkey FOREIGN KEY (target_revision_id) REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT;





--

-- Name: shift_swap_requests shift_swap_requests_target_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_swap_requests

    ADD CONSTRAINT shift_swap_requests_target_template_id_fkey FOREIGN KEY (target_template_id) REFERENCES public.shift_templates(id) ON DELETE SET NULL;





--

-- Name: shift_templates shift_templates_current_version_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_templates

    ADD CONSTRAINT shift_templates_current_version_fkey FOREIGN KEY (current_version_id) REFERENCES public.shift_versions(id) ON DELETE SET NULL;





--

-- Name: shift_versions shift_versions_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.shift_versions

    ADD CONSTRAINT shift_versions_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.shift_templates(id);





--

-- Name: time_entries time_entries_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.time_entries

    ADD CONSTRAINT time_entries_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: time_entries time_entries_segment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.time_entries

    ADD CONSTRAINT time_entries_segment_id_fkey FOREIGN KEY (segment_id) REFERENCES public.work_segments(id) ON DELETE SET NULL;





--

-- Name: time_entries time_entries_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.time_entries

    ADD CONSTRAINT time_entries_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.shift_templates(id) ON DELETE SET NULL;



-- Name: time_entries time_entries_shift_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

ALTER TABLE ONLY public.time_entries

    ADD CONSTRAINT time_entries_shift_version_id_fkey FOREIGN KEY (shift_version_id) REFERENCES public.shift_versions(id) ON DELETE RESTRICT;





--

-- Name: work_segments work_segments_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_segments

    ADD CONSTRAINT work_segments_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;





--

-- Name: work_segments work_segments_interval_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_segments

    ADD CONSTRAINT work_segments_interval_id_fkey FOREIGN KEY (interval_id) REFERENCES public.shift_default_intervals(id) ON DELETE SET NULL;





--

-- Name: work_segments work_segments_schedule_revision_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_segments

    ADD CONSTRAINT work_segments_schedule_revision_fkey FOREIGN KEY (schedule_revision_id) REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT;





--

-- Name: work_segments work_segments_selected_shift_version_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_segments

    ADD CONSTRAINT work_segments_selected_shift_version_fkey FOREIGN KEY (selected_shift_version_id) REFERENCES public.shift_versions(id) ON DELETE RESTRICT;





--

-- Name: work_segments work_segments_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_segments

    ADD CONSTRAINT work_segments_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.shift_templates(id) ON DELETE RESTRICT;





--

-- Name: working_pattern_details working_pattern_details_pattern_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.working_pattern_details

    ADD CONSTRAINT working_pattern_details_pattern_id_fkey FOREIGN KEY (pattern_id) REFERENCES public.working_patterns(id) ON DELETE CASCADE;





--

-- Name: working_pattern_details working_pattern_details_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.working_pattern_details

    ADD CONSTRAINT working_pattern_details_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.shift_templates(id) ON DELETE SET NULL;





--

-- PostgreSQL database dump complete

--

-- Name: COLUMN employee_working_patterns.anchor_date; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.employee_working_patterns.anchor_date IS 'Reference start date for rotation patterns. Determines phase offset per assignment.';





--

-- Name: COLUMN shift_audit_log.new_values; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.shift_audit_log.new_values IS 'Full JSON snapshot of the new version including intervals.';





--

-- Name: COLUMN shift_audit_log.old_values; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.shift_audit_log.old_values IS 'Full JSON snapshot of the superseded version including intervals. NULL on initial creation.';





--

-- Name: COLUMN shift_versions.change_type; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.shift_versions.change_type IS 'create=initial version, revise=scheduled change with new effective_from, correct=error fix keeping same effective_from';





--

-- Name: COLUMN shift_versions.effective_to; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.shift_versions.effective_to IS 'NULL for current version. Set to (new.effective_from - 1 day) when superseded by a revision, or (new.effective_from) when superseded by a correction.';





--

-- Name: COLUMN shift_versions.superseded_by; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.shift_versions.superseded_by IS 'Points to the version that replaces this one. NULL if this is the current version.';





--

-- Name: COLUMN time_entries.segment_id; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.time_entries.segment_id IS 'Links the actual punch session to its scheduled work segment. NULL for backward compat (entries before Work Segments migration).';





--

-- Name: COLUMN work_segments.compensation_rate; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.work_segments.compensation_rate IS '1.00 = 100% pay for this segment. For standby typically 0.30-0.50';





--

-- Name: COLUMN work_segments.gap_compensation_before; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.work_segments.gap_compensation_before IS 'How the gap BEFORE this segment is treated: unpaid (no pay), standby (partial pay), break (auto-deduct like a long break)';





--

-- Name: COLUMN work_segments.segment_type; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.work_segments.segment_type IS 'work=regular shift, standby=paid waiting, on_call=unpaid availability, training=education, travel=business trip';





--

-- Name: COLUMN work_segments.source; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.work_segments.source IS 'roster | pattern | rule | fixed | manual | auto';





--

-- Name: COLUMN working_pattern_details.schedule_day; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON COLUMN public.working_pattern_details.schedule_day IS 'For weekly patterns: day_of_week (0=Sun..6=Sat). For rotation patterns: cycle_day (0..N-1)';





--

-- Name: TABLE employee_shifts; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.employee_shifts IS 'DEPRECATED: This table has been replaced by employee_working_patterns (for pattern assignment) and work_segments (for scheduled work instances). Do not use for new features. Kept only for backward compatibility during migration period.';





--

-- Name: TABLE shift_audit_log; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.shift_audit_log IS 'Append-only tamper-proof audit trail for all shift changes. Populated by application code, not triggers.';





--

-- Name: TABLE shift_default_intervals; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.shift_default_intervals IS 'Optional time blocks within a shift template. When empty, the shift start_time/end_time is used as a single virtual interval.';





--

-- Name: TABLE shift_templates; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.shift_templates IS 'Stable logical identity for a shift concept. Never changes across revisions or corrections.';





--

-- Name: TABLE shift_versions; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.shift_versions IS 'Immutable SCD Type-2 rows. One row per change. Never UPDATE, only INSERT.';





--

-- Name: TABLE work_segments; Type: COMMENT; Schema: public; Owner: -

--



COMMENT ON TABLE public.work_segments IS 'Work segments are the atomic unit of scheduled work. One work day can have N segments. Replaces shift_rosters as the scheduling entity.';





--

CREATE TABLE public.time_entry_breaks (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    time_entry_id uuid NOT NULL,
    shift_interval_id uuid,
    break_start_at timestamp with time zone NOT NULL,
    break_end_at timestamp with time zone,
    source text NOT NULL DEFAULT 'punch',
    close_reason text,
    start_idempotency_key text,
    end_idempotency_key text,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT time_entry_breaks_pkey PRIMARY KEY (id),
    CONSTRAINT time_entry_breaks_entry_fkey FOREIGN KEY (time_entry_id) REFERENCES public.time_entries(id) ON DELETE RESTRICT,
    CONSTRAINT time_entry_breaks_interval_fkey FOREIGN KEY (shift_interval_id) REFERENCES public.shift_default_intervals(id) ON DELETE SET NULL,
    CONSTRAINT time_entry_breaks_source_check CHECK (source IN ('punch', 'system_implied', 'manual')),
    CONSTRAINT time_entry_breaks_close_reason_check CHECK (close_reason IS NULL OR close_reason IN ('punch', 'work_out', 'manual_correction')),
    CONSTRAINT time_entry_breaks_order_check CHECK (break_end_at IS NULL OR break_end_at >= break_start_at)
);
CREATE INDEX time_entry_breaks_time_entry_idx ON public.time_entry_breaks (time_entry_id, break_start_at);
CREATE UNIQUE INDEX time_entry_breaks_one_open_idx ON public.time_entry_breaks (time_entry_id) WHERE break_end_at IS NULL;

CREATE TABLE public.attendance_action_idempotency (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_id uuid NOT NULL,
    idempotency_key text NOT NULL,
    action text NOT NULL,
    request_hash character(64) NOT NULL,
    response_snapshot jsonb NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attendance_action_idempotency_pkey PRIMARY KEY (id),
    CONSTRAINT attendance_action_idempotency_employment_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_action_idempotency_key_key UNIQUE (employment_id, idempotency_key),
    CONSTRAINT attendance_action_idempotency_action_check CHECK (action IN ('work_in', 'work_out', 'break_start', 'break_end')),
    CONSTRAINT attendance_action_idempotency_hash_check CHECK (request_hash::text ~ '^[0-9a-f]{64}$'),
    CONSTRAINT attendance_action_idempotency_response_check CHECK (jsonb_typeof(response_snapshot) = 'object')
);
CREATE INDEX attendance_action_idempotency_created_idx ON public.attendance_action_idempotency (created_at);

CREATE TABLE public.schedule_day_overrides (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_id uuid NOT NULL,
    business_date date NOT NULL,
    work_segment_id uuid NOT NULL,
    override_type text NOT NULL,
    start_at timestamp with time zone NOT NULL,
    end_at timestamp with time zone NOT NULL,
    minutes integer NOT NULL,
    reason text NOT NULL,
    status text NOT NULL DEFAULT 'approved',
    created_by uuid NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT schedule_day_overrides_pkey PRIMARY KEY (id),
    CONSTRAINT schedule_day_overrides_employment_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_overrides_segment_fkey FOREIGN KEY (work_segment_id) REFERENCES public.work_segments(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_overrides_creator_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_overrides_type_check CHECK (override_type = 'break_worked'),
    CONSTRAINT schedule_day_overrides_status_check CHECK (status IN ('pending', 'approved', 'rejected')),
    CONSTRAINT schedule_day_overrides_time_check CHECK (end_at > start_at AND minutes > 0),
    CONSTRAINT schedule_day_overrides_reason_check CHECK (length(btrim(reason)) BETWEEN 1 AND 2000)
);
CREATE INDEX schedule_day_overrides_scope_idx ON public.schedule_day_overrides (employment_id, business_date, work_segment_id);

CREATE TABLE public.attendance_adjustment_evidence (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    adjustment_request_id uuid NOT NULL,
    employment_id uuid NOT NULL,
    business_date date NOT NULL,
    attendance_period_id uuid,
    closure_id uuid,
    before_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    proposed_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    reason text NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attendance_adjustment_evidence_pkey PRIMARY KEY (id),
    CONSTRAINT attendance_adjustment_evidence_request_fkey FOREIGN KEY (adjustment_request_id) REFERENCES public.attendance_adjustment_requests(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_employment_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_period_fkey FOREIGN KEY (attendance_period_id) REFERENCES public.attendance_periods(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_closure_fkey FOREIGN KEY (closure_id) REFERENCES public.attendance_closures(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_creator_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_before_check CHECK (jsonb_typeof(before_snapshot) = 'object'),
    CONSTRAINT attendance_adjustment_evidence_proposed_check CHECK (jsonb_typeof(proposed_snapshot) = 'object'),
    CONSTRAINT attendance_adjustment_evidence_reason_check CHECK (length(btrim(reason)) BETWEEN 1 AND 2000)
);
CREATE UNIQUE INDEX attendance_adjustment_evidence_request_idx ON public.attendance_adjustment_evidence (adjustment_request_id);
CREATE INDEX attendance_adjustment_evidence_scope_idx ON public.attendance_adjustment_evidence (employment_id, business_date, created_at DESC);

CREATE TABLE public.attendance_punch_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_id uuid NOT NULL,
    time_entry_id uuid NOT NULL,
    event_type text NOT NULL,
    occurred_at timestamp with time zone NOT NULL,
    idempotency_key text NOT NULL,
    source text NOT NULL DEFAULT 'ess',
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attendance_punch_events_pkey PRIMARY KEY (id),
    CONSTRAINT attendance_punch_events_employment_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_punch_events_time_entry_fkey FOREIGN KEY (time_entry_id) REFERENCES public.time_entries(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_punch_events_type_check CHECK (event_type IN ('work_in', 'break_start', 'break_end', 'work_out')),
    CONSTRAINT attendance_punch_events_source_check CHECK (source IN ('ess', 'manual')),
    CONSTRAINT attendance_punch_events_key_check CHECK (length(btrim(idempotency_key)) BETWEEN 1 AND 128),
    CONSTRAINT attendance_punch_events_entry_key_key UNIQUE (employment_id, idempotency_key)
);

CREATE INDEX attendance_punch_events_entry_time_idx ON public.attendance_punch_events (time_entry_id, occurred_at, id);
CREATE INDEX attendance_punch_events_employment_time_idx ON public.attendance_punch_events (employment_id, occurred_at, id);

CREATE OR REPLACE FUNCTION public.guard_attendance_punch_events_immutable() RETURNS trigger
LANGUAGE plpgsql SET search_path TO 'public'
AS $$
BEGIN
    RAISE EXCEPTION 'attendance punch events are immutable';
END;
$$;

CREATE TRIGGER trg_attendance_punch_events_immutable
BEFORE UPDATE OR DELETE ON public.attendance_punch_events
FOR EACH ROW EXECUTE FUNCTION public.guard_attendance_punch_events_immutable();

INSERT INTO public.schema_migrations(version) VALUES ('fresh_attendance_schedule_v1');

COMMIT;
