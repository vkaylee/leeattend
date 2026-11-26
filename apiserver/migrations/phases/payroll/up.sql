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

-- Fresh install phase: payroll
CREATE TABLE public.work_policy_groups (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    description text,
    archived_at timestamp with time zone,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT work_policy_groups_code_check CHECK (length(btrim(code)) BETWEEN 1 AND 64),
    CONSTRAINT work_policy_groups_name_check CHECK (length(btrim(name)) BETWEEN 1 AND 255)
);

CREATE TABLE public.work_policy_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    group_id uuid NOT NULL,
    version_no integer NOT NULL,
    status text DEFAULT 'draft'::text NOT NULL,
    row_version bigint DEFAULT 1 NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    published_by uuid,
    published_at timestamp with time zone,
    publish_idempotency_key text,
    minutes_per_credit integer DEFAULT 480 NOT NULL,
    work_unit_minutes integer DEFAULT 1 NOT NULL,
    rounding_policy text DEFAULT 'none'::text NOT NULL,
    early_arrival_strategy text DEFAULT 'clamp'::text NOT NULL,
    late_arrival_grace_minutes integer DEFAULT 0 NOT NULL,
    early_ot_enabled boolean DEFAULT false NOT NULL,
    early_ot_qualification_grace_minutes integer DEFAULT 0 NOT NULL,
    early_ot_max_minutes integer,
    break_auto_deduct boolean DEFAULT true NOT NULL,
    break_is_overtime boolean DEFAULT false NOT NULL,
    ot_aggregation_mode text DEFAULT 'aggregate'::text NOT NULL,
    working_pattern_id uuid,
    pay_method text NOT NULL,
    monthly_denominator text,
    monthly_standard_days numeric(12,4),
    rate_component_definition_id uuid,
    paid_leave_credit_mode text DEFAULT 'standard_day'::text NOT NULL,
    overtime_policy_version_id uuid NOT NULL,
    CONSTRAINT work_policy_versions_version_no_check CHECK (version_no >= 1),
    CONSTRAINT work_policy_versions_row_version_check CHECK (row_version >= 1),
    CONSTRAINT work_policy_versions_status_check CHECK (status IN ('draft', 'published', 'archived')),
    CONSTRAINT work_policy_versions_effective_range_check CHECK (effective_to IS NULL OR effective_to > effective_from),
    CONSTRAINT work_policy_versions_published_meta_check CHECK (
        (status = 'draft' AND published_by IS NULL AND published_at IS NULL)
        OR (status IN ('published', 'archived') AND published_by IS NOT NULL AND published_at IS NOT NULL)
    ),
    CONSTRAINT work_policy_versions_minutes_check CHECK (minutes_per_credit > 0 AND work_unit_minutes > 0),
    CONSTRAINT work_policy_versions_rounding_check CHECK (rounding_policy IN ('none', 'floor', 'round', 'ceil')),
    CONSTRAINT work_policy_versions_early_arrival_check CHECK (early_arrival_strategy IN ('allow', 'clamp', 'block')),
    CONSTRAINT work_policy_versions_late_grace_check CHECK (late_arrival_grace_minutes >= 0),
    CONSTRAINT work_policy_versions_early_grace_check CHECK (early_ot_qualification_grace_minutes >= 0),
    CONSTRAINT work_policy_versions_early_max_check CHECK (early_ot_max_minutes IS NULL OR early_ot_max_minutes >= 0),
    CONSTRAINT work_policy_versions_ot_aggregation_check CHECK (ot_aggregation_mode IN ('aggregate', 'per_segment')),
    CONSTRAINT work_policy_versions_pay_method_check CHECK (pay_method IN ('monthly', 'hourly', 'shift')),
    CONSTRAINT work_policy_versions_paid_leave_credit_mode_check CHECK (
        paid_leave_credit_mode IN ('standard_day', 'scheduled_minutes', 'unpaid')
    ),
    CONSTRAINT work_policy_versions_pay_basis_check CHECK (
        (pay_method = 'monthly' AND monthly_denominator = 'fixed' AND monthly_standard_days IS NOT NULL AND monthly_standard_days > 0 AND rate_component_definition_id IS NULL)
        OR (pay_method = 'monthly' AND monthly_denominator = 'schedule' AND monthly_standard_days IS NULL AND rate_component_definition_id IS NULL)
        OR (pay_method IN ('hourly', 'shift') AND monthly_denominator IS NULL AND monthly_standard_days IS NULL AND rate_component_definition_id IS NOT NULL)
    )
);

CREATE TABLE public.work_policy_memberships (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_relationship_id uuid NOT NULL,
    group_id uuid NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT work_policy_memberships_effective_range_check CHECK (effective_to IS NULL OR effective_to > effective_from)
);

CREATE TABLE public.payroll_calculation_runs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_id uuid NOT NULL,
    period_year integer NOT NULL,
    period_month integer NOT NULL,
    formula_version_id uuid,
    engine_version text DEFAULT 'legacy-v1'::text NOT NULL,
    input_snapshot jsonb DEFAULT '{}'::jsonb NOT NULL,
    output_snapshot jsonb DEFAULT '{}'::jsonb NOT NULL,
    calculation_trace jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    payslip_id uuid,
    CONSTRAINT payroll_calculation_runs_period_check CHECK ((((period_year >= 2000) AND (period_year <= 2100)) AND ((period_month >= 1) AND (period_month <= 12))))
);

CREATE TABLE public.payroll_formula_rule_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    rule_id uuid NOT NULL,
    version_number integer NOT NULL,
    expression text NOT NULL,
    effective_from date NOT NULL,
    status text DEFAULT 'draft'::text NOT NULL,
    created_by uuid NOT NULL,
    published_by uuid,
    published_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    retired_at timestamp with time zone,
    CONSTRAINT payroll_formula_rule_versions_expression_check CHECK (((length(btrim(expression)) >= 1) AND (length(btrim(expression)) <= 4096))),
    CONSTRAINT payroll_formula_rule_versions_status_check CHECK ((status = ANY (ARRAY['draft'::text, 'published'::text, 'retired'::text]))),
    CONSTRAINT payroll_formula_rule_versions_version_check CHECK ((version_number >= 1))
);

CREATE TABLE public.payroll_formula_rules (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    component_code text NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    retired_at timestamp with time zone,
    CONSTRAINT payroll_formula_rules_code_check CHECK ((code ~ '^[a-z][a-z0-9_.]{1,99}$'::text)),
    CONSTRAINT payroll_formula_rules_component_check CHECK ((component_code ~ '^[A-Z][A-Z0-9_.]{1,99}$'::text))
);

CREATE TABLE public.payroll_incentive_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_incentive_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['organization'::text, 'department'::text, 'employee'::text, 'user'::text])))
);

CREATE TABLE public.payroll_incentive_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
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

CREATE TABLE public.payroll_input_batches (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    period_year integer NOT NULL,
    period_month integer NOT NULL,
    source text NOT NULL,
    idempotency_key text NOT NULL,
    request_hash text NOT NULL,
    status text DEFAULT 'submitted'::text NOT NULL,
    submitted_by uuid NOT NULL,
    approved_by uuid,
    locked_by uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_input_batches_period_check CHECK ((((period_year >= 2000) AND (period_year <= 2100)) AND ((period_month >= 1) AND (period_month <= 12)))),
    CONSTRAINT payroll_input_batches_source_check CHECK (((length(btrim(source)) >= 1) AND (length(btrim(source)) <= 100))),
    CONSTRAINT payroll_input_batches_status_check CHECK ((status = ANY (ARRAY['submitted'::text, 'approved'::text, 'locked'::text])))
);

CREATE TABLE public.payroll_input_corrections (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    input_id uuid NOT NULL,
    old_value numeric(20,6) NOT NULL,
    new_value numeric(20,6) NOT NULL,
    old_unit text NOT NULL,
    new_unit text NOT NULL,
    reason text NOT NULL,
    corrected_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_input_corrections_reason_check CHECK (((length(btrim(reason)) >= 1) AND (length(btrim(reason)) <= 500)))
);

CREATE TABLE public.payroll_inputs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    batch_id uuid NOT NULL,
    employment_id uuid NOT NULL,
    metric_definition_id uuid NOT NULL,
    value numeric(20,6) NOT NULL,
    unit text NOT NULL,
    source_reference text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    status text DEFAULT 'accepted'::text NOT NULL,
    validation_message text,
    corrected_at timestamp with time zone,
    corrected_by uuid,
    CONSTRAINT payroll_inputs_status_check CHECK ((status = ANY (ARRAY['accepted'::text, 'warning'::text, 'rejected'::text, 'corrected'::text]))),
    CONSTRAINT payroll_inputs_unit_check CHECK (((length(btrim(unit)) >= 1) AND (length(btrim(unit)) <= 40)))
);

CREATE TABLE public.payroll_metric_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    data_type text DEFAULT 'decimal'::text NOT NULL,
    unit text NOT NULL,
    scope text DEFAULT 'employee'::text NOT NULL,
    source text DEFAULT 'external'::text NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_metric_definitions_code_check CHECK ((code ~ '^[a-z][a-z0-9_.]{1,99}$'::text)),
    CONSTRAINT payroll_metric_definitions_data_type_check CHECK ((data_type = 'decimal'::text)),
    CONSTRAINT payroll_metric_definitions_scope_check CHECK ((scope = 'employee'::text))
);

CREATE TABLE public.payroll_rule_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    rule_definition_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    effective_from date NOT NULL,
    effective_to date,
    override_config jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_rule_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['organization'::text, 'department'::text, 'employee'::text])))
);

CREATE TABLE public.payroll_rule_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
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
    std_days_source character varying(50),
    work_policy_version_id uuid
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
    employment_id uuid NOT NULL
);

ALTER TABLE ONLY public.work_policy_groups
    ADD CONSTRAINT work_policy_groups_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.work_policy_groups
    ADD CONSTRAINT work_policy_groups_code_key UNIQUE (code);

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_group_version_unique UNIQUE (group_id, version_no);

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_published_no_overlap EXCLUDE USING gist (
        group_id WITH =,
        daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)'::text) WITH &&
    ) WHERE (status = 'published'::text);

ALTER TABLE ONLY public.work_policy_memberships
    ADD CONSTRAINT work_policy_memberships_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.work_policy_memberships
    ADD CONSTRAINT work_policy_memberships_effective_no_overlap EXCLUDE USING gist (
        employment_relationship_id WITH =,
        daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)'::text) WITH &&
    );

ALTER TABLE ONLY public.payroll_calculation_runs
    ADD CONSTRAINT payroll_calculation_runs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_formula_rule_versions
    ADD CONSTRAINT payroll_formula_published_effective_no_overlap EXCLUDE USING gist (rule_id WITH =, daterange(effective_from, 'infinity'::date, '[)'::text) WITH &&) WHERE ((status = 'published'::text));

ALTER TABLE ONLY public.payroll_formula_rule_versions
    ADD CONSTRAINT payroll_formula_rule_versions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_formula_rule_versions
    ADD CONSTRAINT payroll_formula_rule_versions_rule_id_version_number_key UNIQUE (rule_id, version_number);

ALTER TABLE ONLY public.payroll_formula_rules
    ADD CONSTRAINT payroll_formula_rules_code_key UNIQUE (code);

ALTER TABLE ONLY public.payroll_formula_rules
    ADD CONSTRAINT payroll_formula_rules_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_incentive_assignments
    ADD CONSTRAINT payroll_incentive_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_incentive_policies
    ADD CONSTRAINT payroll_incentive_policies_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_incentive_tiers
    ADD CONSTRAINT payroll_incentive_tiers_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_input_batches
    ADD CONSTRAINT payroll_input_batches_idempotency_key_key UNIQUE (idempotency_key);

ALTER TABLE ONLY public.payroll_input_batches
    ADD CONSTRAINT payroll_input_batches_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_input_corrections
    ADD CONSTRAINT payroll_input_corrections_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_inputs
    ADD CONSTRAINT payroll_inputs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_inputs
    ADD CONSTRAINT payroll_inputs_unique_metric UNIQUE (batch_id, employment_id, metric_definition_id);

ALTER TABLE ONLY public.payroll_metric_definitions
    ADD CONSTRAINT payroll_metric_definitions_code_key UNIQUE (code);

ALTER TABLE ONLY public.payroll_metric_definitions
    ADD CONSTRAINT payroll_metric_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_organization_rule_definition_id_a_excl EXCLUDE USING gist (rule_definition_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&);

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.payroll_rule_definitions
    ADD CONSTRAINT payroll_rule_definitions_organization_code_key UNIQUE (code);

ALTER TABLE ONLY public.payroll_rule_definitions
    ADD CONSTRAINT payroll_rule_definitions_pkey PRIMARY KEY (id);

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

CREATE INDEX idx_incentive_tiers_policy_id ON public.payroll_incentive_tiers USING btree (policy_id);

CREATE INDEX idx_incentive_tiers_policy_priority ON public.payroll_incentive_tiers USING btree (policy_id, priority);

CREATE INDEX idx_incentive_tiers_rule_config ON public.payroll_incentive_tiers USING gin (rule_config);

CREATE INDEX idx_payroll_incentive_assignments_assignee ON public.payroll_incentive_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_payroll_incentive_assignments_lookup ON public.payroll_incentive_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_payroll_incentive_policies_deleted_at ON public.payroll_incentive_policies USING btree (deleted_at) WHERE (deleted_at IS NULL);

CREATE INDEX idx_payroll_incentive_policies_temporal ON public.payroll_incentive_policies USING btree (effective_from, effective_to);

CREATE INDEX idx_payroll_rule_assignments_assignee ON public.payroll_rule_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_payroll_rule_assignments_rule ON public.payroll_rule_assignments USING btree (rule_definition_id);

CREATE INDEX idx_payroll_rule_assignments_scope ON public.payroll_rule_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_payroll_rule_assignments_temporal ON public.payroll_rule_assignments USING btree (effective_from, effective_to);

CREATE INDEX idx_payroll_rule_defs_category ON public.payroll_rule_definitions USING btree (category);

CREATE INDEX idx_payroll_rule_defs_rule_config ON public.payroll_rule_definitions USING gin (rule_config);

CREATE INDEX idx_payroll_rule_tiers_definition ON public.payroll_rule_tiers USING btree (rule_definition_id);

CREATE INDEX idx_payslip_incentives_payslip_id ON public.payslip_incentives USING btree (payslip_id);

CREATE INDEX idx_payslip_rule_evaluations_payslip_id ON public.payslip_rule_evaluations USING btree (payslip_id);

CREATE INDEX idx_payslip_segments_department_id ON public.payslip_segments USING btree (department_id);

CREATE INDEX idx_payslip_segments_payslip_id ON public.payslip_segments USING btree (payslip_id);

CREATE INDEX idx_payslips_employment_id ON public.payslips USING btree (employment_id);

CREATE INDEX idx_payslips_period ON public.payslips USING btree (period_start, period_end);

CREATE INDEX idx_payslips_status ON public.payslips USING btree (status);

CREATE INDEX work_policy_versions_effective_idx ON public.work_policy_versions USING btree (group_id, effective_from DESC, version_no DESC, id DESC) WHERE (status = 'published'::text);

CREATE INDEX work_policy_versions_group_status_idx ON public.work_policy_versions USING btree (group_id, status, version_no DESC);

CREATE UNIQUE INDEX work_policy_versions_publish_idempotency_idx ON public.work_policy_versions USING btree (group_id, publish_idempotency_key) WHERE (publish_idempotency_key IS NOT NULL);

CREATE INDEX work_policy_memberships_rel_effective_idx ON public.work_policy_memberships USING btree (employment_relationship_id, effective_from DESC, id DESC);

CREATE INDEX work_policy_memberships_group_effective_idx ON public.work_policy_memberships USING btree (group_id, effective_from DESC, id DESC);

CREATE INDEX attendance_daily_summaries_policy_provenance_idx ON public.attendance_daily_summaries USING btree (employment_id, date, work_policy_version_id, shift_version_id);

CREATE INDEX payroll_calculation_runs_employment_period_idx ON public.payroll_calculation_runs USING btree (employment_id, period_year, period_month, created_at DESC);

CREATE UNIQUE INDEX payroll_calculation_runs_formula_payslip_unique ON public.payroll_calculation_runs USING btree (payslip_id, formula_version_id) WHERE ((payslip_id IS NOT NULL) AND (formula_version_id IS NOT NULL));

CREATE INDEX payroll_calculation_runs_payslip_idx ON public.payroll_calculation_runs USING btree (payslip_id, created_at DESC);

CREATE INDEX payroll_formula_versions_effective_idx ON public.payroll_formula_rule_versions USING btree (rule_id, effective_from DESC, version_number DESC);

CREATE INDEX payroll_input_batches_period_status_idx ON public.payroll_input_batches USING btree (period_year, period_month, status);

CREATE INDEX payroll_input_corrections_input_idx ON public.payroll_input_corrections USING btree (input_id, created_at DESC);

CREATE INDEX payroll_inputs_batch_idx ON public.payroll_inputs USING btree (batch_id);

CREATE INDEX payroll_inputs_employment_metric_idx ON public.payroll_inputs USING btree (employment_id, metric_definition_id, created_at DESC);

CREATE INDEX payslip_segments_work_policy_version_idx ON public.payslip_segments USING btree (work_policy_version_id) WHERE (work_policy_version_id IS NOT NULL);

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.payroll_incentive_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.payroll_rule_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.payroll_rule_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.payslips FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.work_policy_groups FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.work_policy_versions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE OR REPLACE FUNCTION public.fn_guard_work_policy_version_immutability() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status IN ('published', 'archived') THEN
            RAISE EXCEPTION 'published or archived work policy version cannot be deleted'
                USING ERRCODE = '23514';
        END IF;
        RETURN OLD;
    END IF;

    IF TG_OP = 'INSERT' OR (TG_OP = 'UPDATE' AND OLD.status = 'draft' AND NEW.status = 'published') THEN
        IF NEW.status = 'published' AND NOT EXISTS (
            SELECT 1
            FROM public.overtime_policy_versions ov
            WHERE ov.id = NEW.overtime_policy_version_id
              AND ov.status = 'published'
        ) THEN
            RAISE EXCEPTION 'published work policy version requires a published overtime policy version'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    IF TG_OP = 'INSERT' THEN
        RETURN NEW;
    END IF;

    IF OLD.status = 'archived' THEN
        RAISE EXCEPTION 'archived work policy version is immutable'
            USING ERRCODE = '23514';
    END IF;

    IF OLD.status = 'published' THEN
        IF NEW.status NOT IN ('published', 'archived') THEN
            RAISE EXCEPTION 'published work policy version cannot revert to draft'
                USING ERRCODE = '23514';
        END IF;

        IF NEW.group_id IS DISTINCT FROM OLD.group_id
           OR NEW.version_no IS DISTINCT FROM OLD.version_no
           OR NEW.effective_from IS DISTINCT FROM OLD.effective_from
           OR NEW.created_by IS DISTINCT FROM OLD.created_by
           OR NEW.created_at IS DISTINCT FROM OLD.created_at
           OR NEW.published_by IS DISTINCT FROM OLD.published_by
           OR NEW.published_at IS DISTINCT FROM OLD.published_at
           OR NEW.publish_idempotency_key IS DISTINCT FROM OLD.publish_idempotency_key
           OR NEW.minutes_per_credit IS DISTINCT FROM OLD.minutes_per_credit
           OR NEW.work_unit_minutes IS DISTINCT FROM OLD.work_unit_minutes
           OR NEW.rounding_policy IS DISTINCT FROM OLD.rounding_policy
           OR NEW.early_arrival_strategy IS DISTINCT FROM OLD.early_arrival_strategy
           OR NEW.late_arrival_grace_minutes IS DISTINCT FROM OLD.late_arrival_grace_minutes
           OR NEW.early_ot_enabled IS DISTINCT FROM OLD.early_ot_enabled
           OR NEW.early_ot_qualification_grace_minutes IS DISTINCT FROM OLD.early_ot_qualification_grace_minutes
           OR NEW.early_ot_max_minutes IS DISTINCT FROM OLD.early_ot_max_minutes
           OR NEW.break_auto_deduct IS DISTINCT FROM OLD.break_auto_deduct
           OR NEW.break_is_overtime IS DISTINCT FROM OLD.break_is_overtime
           OR NEW.ot_aggregation_mode IS DISTINCT FROM OLD.ot_aggregation_mode
           OR NEW.working_pattern_id IS DISTINCT FROM OLD.working_pattern_id
           OR NEW.pay_method IS DISTINCT FROM OLD.pay_method
           OR NEW.monthly_denominator IS DISTINCT FROM OLD.monthly_denominator
           OR NEW.monthly_standard_days IS DISTINCT FROM OLD.monthly_standard_days
           OR NEW.rate_component_definition_id IS DISTINCT FROM OLD.rate_component_definition_id
           OR NEW.paid_leave_credit_mode IS DISTINCT FROM OLD.paid_leave_credit_mode
           OR NEW.overtime_policy_version_id IS DISTINCT FROM OLD.overtime_policy_version_id THEN
            RAISE EXCEPTION 'published work policy version rules are immutable'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_guard_work_policy_version_immutability
    BEFORE INSERT OR UPDATE OR DELETE ON public.work_policy_versions
    FOR EACH ROW EXECUTE FUNCTION public.fn_guard_work_policy_version_immutability();

CREATE OR REPLACE FUNCTION public.fn_validate_work_policy_membership_rel_bounds() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_rel_start date;
    v_rel_end date;
    v_rel_deleted_at timestamp with time zone;
BEGIN
    SELECT start_date, end_date, deleted_at
    INTO v_rel_start, v_rel_end, v_rel_deleted_at
    FROM public.employment_relationships
    WHERE id = NEW.employment_relationship_id;

    IF NOT FOUND THEN
        RETURN NEW;
    END IF;

    IF v_rel_deleted_at IS NOT NULL THEN
        RAISE EXCEPTION 'work policy membership requires non-deleted employment relationship'
            USING ERRCODE = '23514';
    END IF;

    IF NEW.effective_from < v_rel_start
       OR (v_rel_end IS NOT NULL AND (NEW.effective_to IS NULL OR NEW.effective_to > (v_rel_end + 1))) THEN
        RAISE EXCEPTION 'work policy membership dates must fit within parent employment relationship bounds'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION public.fn_validate_relationship_policy_bounds_guard() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.deleted_at IS NULL
       AND (NEW.start_date IS DISTINCT FROM OLD.start_date OR NEW.end_date IS DISTINCT FROM OLD.end_date)
       AND EXISTS (
           SELECT 1
           FROM public.work_policy_memberships m
           WHERE m.employment_relationship_id = NEW.id
             AND (
                 m.effective_from < NEW.start_date
                 OR (NEW.end_date IS NOT NULL AND (m.effective_to IS NULL OR m.effective_to > (NEW.end_date + 1)))
             )
       ) THEN
        RAISE EXCEPTION 'employment relationship bounds cannot exclude existing work policy memberships'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_work_policy_memberships_rel_bounds
    BEFORE INSERT OR UPDATE OF employment_relationship_id, effective_from, effective_to ON public.work_policy_memberships
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_work_policy_membership_rel_bounds();

CREATE TRIGGER trg_employment_relationships_policy_bounds_guard
    BEFORE UPDATE OF start_date, end_date, deleted_at ON public.employment_relationships
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_relationship_policy_bounds_guard();

ALTER TABLE ONLY public.work_policy_groups
    ADD CONSTRAINT work_policy_groups_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_group_id_fkey FOREIGN KEY (group_id) REFERENCES public.work_policy_groups(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_published_by_fkey FOREIGN KEY (published_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_working_pattern_id_fkey FOREIGN KEY (working_pattern_id) REFERENCES public.working_patterns(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_rate_component_definition_id_fkey FOREIGN KEY (rate_component_definition_id) REFERENCES public.salary_component_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.work_policy_versions
    ADD CONSTRAINT work_policy_versions_overtime_policy_version_id_fkey FOREIGN KEY (overtime_policy_version_id) REFERENCES public.overtime_policy_versions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.work_policy_memberships
    ADD CONSTRAINT work_policy_memberships_relationship_id_fkey FOREIGN KEY (employment_relationship_id) REFERENCES public.employment_relationships(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.work_policy_memberships
    ADD CONSTRAINT work_policy_memberships_group_id_fkey FOREIGN KEY (group_id) REFERENCES public.work_policy_groups(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.work_policy_memberships
    ADD CONSTRAINT work_policy_memberships_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_work_policy_version_id_fkey FOREIGN KEY (work_policy_version_id) REFERENCES public.work_policy_versions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.payroll_calculation_runs
    ADD CONSTRAINT payroll_calculation_runs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employment_segments(id);

ALTER TABLE ONLY public.payroll_calculation_runs
    ADD CONSTRAINT payroll_calculation_runs_formula_version_id_fkey FOREIGN KEY (formula_version_id) REFERENCES public.payroll_formula_rule_versions(id);

ALTER TABLE ONLY public.payroll_calculation_runs
    ADD CONSTRAINT payroll_calculation_runs_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id);

ALTER TABLE ONLY public.payroll_formula_rule_versions
    ADD CONSTRAINT payroll_formula_rule_versions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.payroll_formula_rule_versions
    ADD CONSTRAINT payroll_formula_rule_versions_published_by_fkey FOREIGN KEY (published_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.payroll_formula_rule_versions
    ADD CONSTRAINT payroll_formula_rule_versions_rule_id_fkey FOREIGN KEY (rule_id) REFERENCES public.payroll_formula_rules(id);

ALTER TABLE ONLY public.payroll_formula_rules
    ADD CONSTRAINT payroll_formula_rules_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.payroll_incentive_assignments
    ADD CONSTRAINT payroll_incentive_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_incentive_tiers
    ADD CONSTRAINT payroll_incentive_tiers_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payroll_input_batches
    ADD CONSTRAINT payroll_input_batches_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.payroll_input_batches
    ADD CONSTRAINT payroll_input_batches_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.payroll_input_batches
    ADD CONSTRAINT payroll_input_batches_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.payroll_input_corrections
    ADD CONSTRAINT payroll_input_corrections_corrected_by_fkey FOREIGN KEY (corrected_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.payroll_input_corrections
    ADD CONSTRAINT payroll_input_corrections_input_id_fkey FOREIGN KEY (input_id) REFERENCES public.payroll_inputs(id);

ALTER TABLE ONLY public.payroll_inputs
    ADD CONSTRAINT payroll_inputs_batch_id_fkey FOREIGN KEY (batch_id) REFERENCES public.payroll_input_batches(id);

ALTER TABLE ONLY public.payroll_inputs
    ADD CONSTRAINT payroll_inputs_corrected_by_fkey FOREIGN KEY (corrected_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.payroll_inputs
    ADD CONSTRAINT payroll_inputs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employment_segments(id);

ALTER TABLE ONLY public.payroll_inputs
    ADD CONSTRAINT payroll_inputs_metric_definition_id_fkey FOREIGN KEY (metric_definition_id) REFERENCES public.payroll_metric_definitions(id);

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
    ADD CONSTRAINT payslip_segments_work_policy_version_id_fkey FOREIGN KEY (work_policy_version_id) REFERENCES public.work_policy_versions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.payslip_segments
    ADD CONSTRAINT payslip_segments_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.payslips
    ADD CONSTRAINT payslips_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employment_segments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_linked_rule_id_fkey FOREIGN KEY (linked_rule_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE SET NULL;

INSERT INTO public.schema_migrations(version) VALUES ('fresh_payroll_v1');

COMMIT;
