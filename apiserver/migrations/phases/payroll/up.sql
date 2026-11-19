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

-- Name: labor_policy_group_versions; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.labor_policy_group_versions (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    group_id uuid NOT NULL,

    work_credit_policy_id uuid NOT NULL,

    pay_basis_policy_id uuid NOT NULL,

    overtime_policy_id uuid NOT NULL,

    effective_from date NOT NULL,

    effective_to date,

    created_by uuid NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT labor_policy_group_versions_effective_range_check CHECK (((effective_to IS NULL) OR (effective_to > effective_from)))

);





--

-- Name: labor_policy_groups; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.labor_policy_groups (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name text NOT NULL,

    created_by uuid NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT labor_policy_groups_name_check CHECK (((length(btrim(name)) >= 1) AND (length(btrim(name)) <= 255)))

);





--

-- Name: labor_policy_memberships; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.labor_policy_memberships (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    employment_id uuid NOT NULL,

    group_id uuid NOT NULL,

    effective_from date NOT NULL,

    effective_to date,

    created_by uuid NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT labor_policy_memberships_effective_range_check CHECK (((effective_to IS NULL) OR (effective_to > effective_from)))

);





--

-- Name: pay_basis_policies; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.pay_basis_policies (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name text NOT NULL,

    method text NOT NULL,

    monthly_denominator text,

    monthly_standard_days numeric(12,4),

    created_by uuid NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    rate_component_definition_id uuid,

    CONSTRAINT pay_basis_policies_days_check CHECK (((monthly_standard_days IS NULL) OR (monthly_standard_days > (0)::numeric))),

    CONSTRAINT pay_basis_policies_denominator_check CHECK ((((method = 'monthly'::text) AND (monthly_denominator = ANY (ARRAY['fixed'::text, 'schedule'::text]))) OR ((method = ANY (ARRAY['hourly'::text, 'shift'::text])) AND (monthly_denominator IS NULL) AND (monthly_standard_days IS NULL)))),

    CONSTRAINT pay_basis_policies_fixed_days_check CHECK (((method <> 'monthly'::text) OR (monthly_denominator <> 'fixed'::text) OR (monthly_standard_days IS NOT NULL))),

    CONSTRAINT pay_basis_policies_method_check CHECK ((method = ANY (ARRAY['monthly'::text, 'hourly'::text, 'shift'::text]))),

    CONSTRAINT pay_basis_policies_name_check CHECK (((length(btrim(name)) >= 1) AND (length(btrim(name)) <= 255))),

    CONSTRAINT pay_basis_policies_rate_component_check CHECK ((((method = 'monthly'::text) AND (rate_component_definition_id IS NULL)) OR ((method = ANY (ARRAY['hourly'::text, 'shift'::text])) AND (rate_component_definition_id IS NOT NULL)))),

    CONSTRAINT pay_basis_policies_schedule_days_check CHECK (((method <> 'monthly'::text) OR (monthly_denominator <> 'schedule'::text) OR (monthly_standard_days IS NULL)))

);





--

-- Name: payroll_calculation_runs; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_formula_rule_versions; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_formula_rules; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_incentive_assignments; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_incentive_policies; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_incentive_tiers; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.payroll_incentive_tiers (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    policy_id uuid NOT NULL,

    priority integer NOT NULL,

    payout_amount double precision NOT NULL,

    rule_config jsonb DEFAULT '{}'::jsonb NOT NULL

);





--

-- Name: payroll_input_batches; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_input_corrections; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_inputs; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_metric_definitions; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_rule_assignments; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_rule_definitions; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payroll_rule_tiers; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: payslip_incentives; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.payslip_incentives (

    payslip_id uuid NOT NULL,

    policy_name text NOT NULL,

    amount double precision NOT NULL,

    tier_id uuid,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    standard_amount double precision DEFAULT 0.0 NOT NULL,

    id uuid DEFAULT gen_random_uuid() NOT NULL

);





--

-- Name: payslip_rule_evaluations; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.payslip_rule_evaluations (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    payslip_id uuid NOT NULL,

    rule_code text NOT NULL,

    amount numeric(15,2) NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL

);





--

-- Name: payslip_segments; Type: TABLE; Schema: public; Owner: -

--



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

    labor_policy_group_version_id uuid

);





--

-- Name: payslips; Type: TABLE; Schema: public; Owner: -

--



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





--

-- Name: work_credit_policies; Type: TABLE; Schema: public; Owner: -

--



CREATE TABLE public.work_credit_policies (

    id uuid DEFAULT gen_random_uuid() NOT NULL,

    name text NOT NULL,

    minutes_per_credit integer NOT NULL,

    attendance_policy_id uuid NOT NULL,

    working_pattern_id uuid NOT NULL,

    late_arrival_grace_minutes integer NOT NULL DEFAULT 0,
    early_ot_enabled boolean NOT NULL DEFAULT false,
    early_ot_qualification_grace_minutes integer NOT NULL DEFAULT 0,
    early_ot_max_minutes integer,
    break_auto_deduct boolean NOT NULL DEFAULT true,
    break_is_overtime boolean NOT NULL DEFAULT false,
    ot_aggregation_mode text NOT NULL DEFAULT 'aggregate',

    created_by uuid NOT NULL,

    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT work_credit_policies_minutes_check CHECK ((minutes_per_credit > 0)),
    CONSTRAINT work_credit_policies_late_grace_check CHECK (late_arrival_grace_minutes >= 0),
    CONSTRAINT work_credit_policies_early_grace_check CHECK (early_ot_qualification_grace_minutes >= 0),
    CONSTRAINT work_credit_policies_early_max_check CHECK (early_ot_max_minutes IS NULL OR early_ot_max_minutes >= 0),
    CONSTRAINT work_credit_policies_ot_aggregation_check CHECK (ot_aggregation_mode IN ('aggregate', 'per_segment')),

    CONSTRAINT work_credit_policies_name_check CHECK (((length(btrim(name)) >= 1) AND (length(btrim(name)) <= 255)))

);





--

-- Name: labor_policy_group_versions labor_policy_group_versions_effective_no_overlap; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_group_versions

    ADD CONSTRAINT labor_policy_group_versions_effective_no_overlap EXCLUDE USING gist (group_id WITH =, daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)'::text) WITH &&);





--

-- Name: labor_policy_group_versions labor_policy_group_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_group_versions

    ADD CONSTRAINT labor_policy_group_versions_pkey PRIMARY KEY (id);





--

-- Name: labor_policy_groups labor_policy_groups_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_groups

    ADD CONSTRAINT labor_policy_groups_pkey PRIMARY KEY (id);





--

-- Name: labor_policy_memberships labor_policy_memberships_effective_no_overlap; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_memberships

    ADD CONSTRAINT labor_policy_memberships_effective_no_overlap EXCLUDE USING gist (employment_id WITH =, daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)'::text) WITH &&);





--

-- Name: labor_policy_memberships labor_policy_memberships_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_memberships

    ADD CONSTRAINT labor_policy_memberships_pkey PRIMARY KEY (id);





--

-- Name: pay_basis_policies pay_basis_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.pay_basis_policies

    ADD CONSTRAINT pay_basis_policies_pkey PRIMARY KEY (id);





--

-- Name: payroll_calculation_runs payroll_calculation_runs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_calculation_runs

    ADD CONSTRAINT payroll_calculation_runs_pkey PRIMARY KEY (id);





--

-- Name: payroll_formula_rule_versions payroll_formula_published_effective_no_overlap; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rule_versions

    ADD CONSTRAINT payroll_formula_published_effective_no_overlap EXCLUDE USING gist (rule_id WITH =, daterange(effective_from, 'infinity'::date, '[)'::text) WITH &&) WHERE ((status = 'published'::text));





--

-- Name: payroll_formula_rule_versions payroll_formula_rule_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rule_versions

    ADD CONSTRAINT payroll_formula_rule_versions_pkey PRIMARY KEY (id);





--

-- Name: payroll_formula_rule_versions payroll_formula_rule_versions_rule_id_version_number_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rule_versions

    ADD CONSTRAINT payroll_formula_rule_versions_rule_id_version_number_key UNIQUE (rule_id, version_number);





--

-- Name: payroll_formula_rules payroll_formula_rules_code_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rules

    ADD CONSTRAINT payroll_formula_rules_code_key UNIQUE (code);





--

-- Name: payroll_formula_rules payroll_formula_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rules

    ADD CONSTRAINT payroll_formula_rules_pkey PRIMARY KEY (id);





--

-- Name: payroll_incentive_assignments payroll_incentive_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_incentive_assignments

    ADD CONSTRAINT payroll_incentive_assignments_pkey PRIMARY KEY (id);





--

-- Name: payroll_incentive_policies payroll_incentive_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_incentive_policies

    ADD CONSTRAINT payroll_incentive_policies_pkey PRIMARY KEY (id);





--

-- Name: payroll_incentive_tiers payroll_incentive_tiers_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_incentive_tiers

    ADD CONSTRAINT payroll_incentive_tiers_pkey PRIMARY KEY (id);





--

-- Name: payroll_input_batches payroll_input_batches_idempotency_key_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_input_batches

    ADD CONSTRAINT payroll_input_batches_idempotency_key_key UNIQUE (idempotency_key);





--

-- Name: payroll_input_batches payroll_input_batches_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_input_batches

    ADD CONSTRAINT payroll_input_batches_pkey PRIMARY KEY (id);





--

-- Name: payroll_input_corrections payroll_input_corrections_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_input_corrections

    ADD CONSTRAINT payroll_input_corrections_pkey PRIMARY KEY (id);





--

-- Name: payroll_inputs payroll_inputs_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_inputs

    ADD CONSTRAINT payroll_inputs_pkey PRIMARY KEY (id);





--

-- Name: payroll_inputs payroll_inputs_unique_metric; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_inputs

    ADD CONSTRAINT payroll_inputs_unique_metric UNIQUE (batch_id, employment_id, metric_definition_id);





--

-- Name: payroll_metric_definitions payroll_metric_definitions_code_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_metric_definitions

    ADD CONSTRAINT payroll_metric_definitions_code_key UNIQUE (code);





--

-- Name: payroll_metric_definitions payroll_metric_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_metric_definitions

    ADD CONSTRAINT payroll_metric_definitions_pkey PRIMARY KEY (id);





--

-- Name: payroll_rule_assignments payroll_rule_assignments_organization_rule_definition_id_a_excl; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_rule_assignments

    ADD CONSTRAINT payroll_rule_assignments_organization_rule_definition_id_a_excl EXCLUDE USING gist (rule_definition_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&);





--

-- Name: payroll_rule_assignments payroll_rule_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_rule_assignments

    ADD CONSTRAINT payroll_rule_assignments_pkey PRIMARY KEY (id);





--

-- Name: payroll_rule_definitions payroll_rule_definitions_organization_code_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_rule_definitions

    ADD CONSTRAINT payroll_rule_definitions_organization_code_key UNIQUE (code);





--

-- Name: payroll_rule_definitions payroll_rule_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_rule_definitions

    ADD CONSTRAINT payroll_rule_definitions_pkey PRIMARY KEY (id);





--

-- Name: payroll_rule_tiers payroll_rule_tiers_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_rule_tiers

    ADD CONSTRAINT payroll_rule_tiers_pkey PRIMARY KEY (id);





--

-- Name: payroll_rule_tiers payroll_rule_tiers_rule_definition_id_tier_order_key; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_rule_tiers

    ADD CONSTRAINT payroll_rule_tiers_rule_definition_id_tier_order_key UNIQUE (rule_definition_id, tier_order);





--

-- Name: payslip_incentives payslip_incentives_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslip_incentives

    ADD CONSTRAINT payslip_incentives_pkey PRIMARY KEY (id);





--

-- Name: payslip_rule_evaluations payslip_rule_evaluations_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslip_rule_evaluations

    ADD CONSTRAINT payslip_rule_evaluations_pkey PRIMARY KEY (id);





--

-- Name: payslip_segments payslip_segments_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslip_segments

    ADD CONSTRAINT payslip_segments_pkey PRIMARY KEY (id);





--

-- Name: payslips payslips_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslips

    ADD CONSTRAINT payslips_pkey PRIMARY KEY (id);





--

-- Name: work_credit_policies work_credit_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_credit_policies

    ADD CONSTRAINT work_credit_policies_pkey PRIMARY KEY (id);





--

-- Name: idx_incentive_tiers_policy_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_incentive_tiers_policy_id ON public.payroll_incentive_tiers USING btree (policy_id);





--

-- Name: idx_incentive_tiers_policy_priority; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_incentive_tiers_policy_priority ON public.payroll_incentive_tiers USING btree (policy_id, priority);





--

-- Name: idx_incentive_tiers_rule_config; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_incentive_tiers_rule_config ON public.payroll_incentive_tiers USING gin (rule_config);





--

-- Name: idx_payroll_incentive_assignments_assignee; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_incentive_assignments_assignee ON public.payroll_incentive_assignments USING btree (assignee_type, assignee_id);





--

-- Name: idx_payroll_incentive_assignments_lookup; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_incentive_assignments_lookup ON public.payroll_incentive_assignments USING btree (assignee_type, assignee_id);





--

-- Name: idx_payroll_incentive_policies_deleted_at; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_incentive_policies_deleted_at ON public.payroll_incentive_policies USING btree (deleted_at) WHERE (deleted_at IS NULL);





--

-- Name: idx_payroll_incentive_policies_temporal; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_incentive_policies_temporal ON public.payroll_incentive_policies USING btree (effective_from, effective_to);





--

-- Name: idx_payroll_rule_assignments_assignee; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_rule_assignments_assignee ON public.payroll_rule_assignments USING btree (assignee_type, assignee_id);





--

-- Name: idx_payroll_rule_assignments_rule; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_rule_assignments_rule ON public.payroll_rule_assignments USING btree (rule_definition_id);





--

-- Name: idx_payroll_rule_assignments_scope; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_rule_assignments_scope ON public.payroll_rule_assignments USING btree (assignee_type, assignee_id);





--

-- Name: idx_payroll_rule_assignments_temporal; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_rule_assignments_temporal ON public.payroll_rule_assignments USING btree (effective_from, effective_to);





--

-- Name: idx_payroll_rule_defs_category; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_rule_defs_category ON public.payroll_rule_definitions USING btree (category);





--

-- Name: idx_payroll_rule_defs_rule_config; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_rule_defs_rule_config ON public.payroll_rule_definitions USING gin (rule_config);





--

-- Name: idx_payroll_rule_tiers_definition; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payroll_rule_tiers_definition ON public.payroll_rule_tiers USING btree (rule_definition_id);





--

-- Name: idx_payslip_incentives_payslip_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payslip_incentives_payslip_id ON public.payslip_incentives USING btree (payslip_id);





--

-- Name: idx_payslip_rule_evaluations_payslip_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payslip_rule_evaluations_payslip_id ON public.payslip_rule_evaluations USING btree (payslip_id);





--

-- Name: idx_payslip_segments_department_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payslip_segments_department_id ON public.payslip_segments USING btree (department_id);





--

-- Name: idx_payslip_segments_payslip_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payslip_segments_payslip_id ON public.payslip_segments USING btree (payslip_id);





--

-- Name: idx_payslips_employment_id; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payslips_employment_id ON public.payslips USING btree (employment_id);





--

-- Name: idx_payslips_period; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payslips_period ON public.payslips USING btree (period_start, period_end);





--

-- Name: idx_payslips_status; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX idx_payslips_status ON public.payslips USING btree (status);





--

-- Name: labor_policy_group_versions_effective_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX labor_policy_group_versions_effective_idx ON public.labor_policy_group_versions USING btree (group_id, effective_from DESC, id DESC);





--

-- Name: labor_policy_memberships_employment_effective_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX labor_policy_memberships_employment_effective_idx ON public.labor_policy_memberships USING btree (employment_id, effective_from DESC, id DESC);





--

-- Name: payroll_calculation_runs_employment_period_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payroll_calculation_runs_employment_period_idx ON public.payroll_calculation_runs USING btree (employment_id, period_year, period_month, created_at DESC);





--

-- Name: payroll_calculation_runs_formula_payslip_unique; Type: INDEX; Schema: public; Owner: -

--



CREATE UNIQUE INDEX payroll_calculation_runs_formula_payslip_unique ON public.payroll_calculation_runs USING btree (payslip_id, formula_version_id) WHERE ((payslip_id IS NOT NULL) AND (formula_version_id IS NOT NULL));





--

-- Name: payroll_calculation_runs_payslip_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payroll_calculation_runs_payslip_idx ON public.payroll_calculation_runs USING btree (payslip_id, created_at DESC);





--

-- Name: payroll_formula_versions_effective_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payroll_formula_versions_effective_idx ON public.payroll_formula_rule_versions USING btree (rule_id, effective_from DESC, version_number DESC);





--

-- Name: payroll_input_batches_period_status_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payroll_input_batches_period_status_idx ON public.payroll_input_batches USING btree (period_year, period_month, status);





--

-- Name: payroll_input_corrections_input_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payroll_input_corrections_input_idx ON public.payroll_input_corrections USING btree (input_id, created_at DESC);





--

-- Name: payroll_inputs_batch_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payroll_inputs_batch_idx ON public.payroll_inputs USING btree (batch_id);





--

-- Name: payroll_inputs_employment_metric_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payroll_inputs_employment_metric_idx ON public.payroll_inputs USING btree (employment_id, metric_definition_id, created_at DESC);





--

-- Name: payslip_segments_labor_policy_version_idx; Type: INDEX; Schema: public; Owner: -

--



CREATE INDEX payslip_segments_labor_policy_version_idx ON public.payslip_segments USING btree (labor_policy_group_version_id) WHERE (labor_policy_group_version_id IS NOT NULL);





--

-- Name: payroll_incentive_assignments trg_validate_assignee_fk; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.payroll_incentive_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();





--

-- Name: payroll_rule_assignments trg_validate_assignee_fk; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.payroll_rule_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();





--

-- Name: payroll_rule_definitions trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.payroll_rule_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: payslips trg_updated_at; Type: TRIGGER; Schema: public; Owner: -

--



CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.payslips FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();





--

-- Name: labor_policy_group_versions labor_policy_group_versions_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_group_versions

    ADD CONSTRAINT labor_policy_group_versions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);





--

-- Name: labor_policy_group_versions labor_policy_group_versions_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_group_versions

    ADD CONSTRAINT labor_policy_group_versions_group_id_fkey FOREIGN KEY (group_id) REFERENCES public.labor_policy_groups(id);





--

-- Name: labor_policy_group_versions labor_policy_group_versions_overtime_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_group_versions

    ADD CONSTRAINT labor_policy_group_versions_overtime_policy_id_fkey FOREIGN KEY (overtime_policy_id) REFERENCES public.overtime_policy_definitions(id);





--

-- Name: labor_policy_group_versions labor_policy_group_versions_pay_basis_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_group_versions

    ADD CONSTRAINT labor_policy_group_versions_pay_basis_policy_id_fkey FOREIGN KEY (pay_basis_policy_id) REFERENCES public.pay_basis_policies(id);





--

-- Name: labor_policy_group_versions labor_policy_group_versions_work_credit_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_group_versions

    ADD CONSTRAINT labor_policy_group_versions_work_credit_policy_id_fkey FOREIGN KEY (work_credit_policy_id) REFERENCES public.work_credit_policies(id);





--

-- Name: labor_policy_groups labor_policy_groups_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_groups

    ADD CONSTRAINT labor_policy_groups_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);





--

-- Name: labor_policy_memberships labor_policy_memberships_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_memberships

    ADD CONSTRAINT labor_policy_memberships_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);





--

-- Name: labor_policy_memberships labor_policy_memberships_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_memberships

    ADD CONSTRAINT labor_policy_memberships_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id);





--

-- Name: labor_policy_memberships labor_policy_memberships_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.labor_policy_memberships

    ADD CONSTRAINT labor_policy_memberships_group_id_fkey FOREIGN KEY (group_id) REFERENCES public.labor_policy_groups(id);





--

-- Name: pay_basis_policies pay_basis_policies_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.pay_basis_policies

    ADD CONSTRAINT pay_basis_policies_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);





--

-- Name: pay_basis_policies pay_basis_policies_rate_component_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.pay_basis_policies

    ADD CONSTRAINT pay_basis_policies_rate_component_definition_id_fkey FOREIGN KEY (rate_component_definition_id) REFERENCES public.salary_component_definitions(id);





--

-- Name: payroll_calculation_runs payroll_calculation_runs_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_calculation_runs

    ADD CONSTRAINT payroll_calculation_runs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id);





--

-- Name: payroll_calculation_runs payroll_calculation_runs_formula_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_calculation_runs

    ADD CONSTRAINT payroll_calculation_runs_formula_version_id_fkey FOREIGN KEY (formula_version_id) REFERENCES public.payroll_formula_rule_versions(id);





--

-- Name: payroll_calculation_runs payroll_calculation_runs_payslip_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_calculation_runs

    ADD CONSTRAINT payroll_calculation_runs_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id);





--

-- Name: payroll_formula_rule_versions payroll_formula_rule_versions_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rule_versions

    ADD CONSTRAINT payroll_formula_rule_versions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);





--

-- Name: payroll_formula_rule_versions payroll_formula_rule_versions_published_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rule_versions

    ADD CONSTRAINT payroll_formula_rule_versions_published_by_fkey FOREIGN KEY (published_by) REFERENCES public.users(id);





--

-- Name: payroll_formula_rule_versions payroll_formula_rule_versions_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rule_versions

    ADD CONSTRAINT payroll_formula_rule_versions_rule_id_fkey FOREIGN KEY (rule_id) REFERENCES public.payroll_formula_rules(id);





--

-- Name: payroll_formula_rules payroll_formula_rules_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_formula_rules

    ADD CONSTRAINT payroll_formula_rules_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);





--

-- Name: payroll_incentive_assignments payroll_incentive_assignments_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_incentive_assignments

    ADD CONSTRAINT payroll_incentive_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE;





--

-- Name: payroll_incentive_tiers payroll_incentive_tiers_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_incentive_tiers

    ADD CONSTRAINT payroll_incentive_tiers_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.payroll_incentive_policies(id) ON DELETE CASCADE;





--

-- Name: payroll_input_batches payroll_input_batches_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_input_batches

    ADD CONSTRAINT payroll_input_batches_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);





--

-- Name: payroll_input_batches payroll_input_batches_locked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_input_batches

    ADD CONSTRAINT payroll_input_batches_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);





--

-- Name: payroll_input_batches payroll_input_batches_submitted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_input_batches

    ADD CONSTRAINT payroll_input_batches_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id);





--

-- Name: payroll_input_corrections payroll_input_corrections_corrected_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_input_corrections

    ADD CONSTRAINT payroll_input_corrections_corrected_by_fkey FOREIGN KEY (corrected_by) REFERENCES public.users(id);





--

-- Name: payroll_input_corrections payroll_input_corrections_input_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_input_corrections

    ADD CONSTRAINT payroll_input_corrections_input_id_fkey FOREIGN KEY (input_id) REFERENCES public.payroll_inputs(id);





--

-- Name: payroll_inputs payroll_inputs_batch_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_inputs

    ADD CONSTRAINT payroll_inputs_batch_id_fkey FOREIGN KEY (batch_id) REFERENCES public.payroll_input_batches(id);





--

-- Name: payroll_inputs payroll_inputs_corrected_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_inputs

    ADD CONSTRAINT payroll_inputs_corrected_by_fkey FOREIGN KEY (corrected_by) REFERENCES public.users(id);





--

-- Name: payroll_inputs payroll_inputs_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_inputs

    ADD CONSTRAINT payroll_inputs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id);





--

-- Name: payroll_inputs payroll_inputs_metric_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_inputs

    ADD CONSTRAINT payroll_inputs_metric_definition_id_fkey FOREIGN KEY (metric_definition_id) REFERENCES public.payroll_metric_definitions(id);





--

-- Name: payroll_rule_assignments payroll_rule_assignments_rule_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_rule_assignments

    ADD CONSTRAINT payroll_rule_assignments_rule_definition_id_fkey FOREIGN KEY (rule_definition_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE RESTRICT;





--

-- Name: payroll_rule_tiers payroll_rule_tiers_rule_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payroll_rule_tiers

    ADD CONSTRAINT payroll_rule_tiers_rule_definition_id_fkey FOREIGN KEY (rule_definition_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE CASCADE;





--

-- Name: payslip_incentives payslip_incentives_payslip_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslip_incentives

    ADD CONSTRAINT payslip_incentives_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;





--

-- Name: payslip_rule_evaluations payslip_rule_evaluations_payslip_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslip_rule_evaluations

    ADD CONSTRAINT payslip_rule_evaluations_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;





--

-- Name: payslip_segments payslip_segments_department_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslip_segments

    ADD CONSTRAINT payslip_segments_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id);





--

-- Name: payslip_segments payslip_segments_labor_policy_group_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslip_segments

    ADD CONSTRAINT payslip_segments_labor_policy_group_version_id_fkey FOREIGN KEY (labor_policy_group_version_id) REFERENCES public.labor_policy_group_versions(id);





--

-- Name: payslip_segments payslip_segments_payslip_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslip_segments

    ADD CONSTRAINT payslip_segments_payslip_id_fkey FOREIGN KEY (payslip_id) REFERENCES public.payslips(id) ON DELETE CASCADE;





--

-- Name: payslips payslips_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.payslips

    ADD CONSTRAINT payslips_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;





--

-- Name: salary_component_definitions salary_component_definitions_linked_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.salary_component_definitions

    ADD CONSTRAINT salary_component_definitions_linked_rule_id_fkey FOREIGN KEY (linked_rule_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE SET NULL;





--

-- Name: work_credit_policies work_credit_policies_attendance_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_credit_policies

    ADD CONSTRAINT work_credit_policies_attendance_policy_id_fkey FOREIGN KEY (attendance_policy_id) REFERENCES public.attendance_policies(id);





--

-- Name: work_credit_policies work_credit_policies_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_credit_policies

    ADD CONSTRAINT work_credit_policies_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);





--

-- Name: work_credit_policies work_credit_policies_working_pattern_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -

--



ALTER TABLE ONLY public.work_credit_policies

    ADD CONSTRAINT work_credit_policies_working_pattern_id_fkey FOREIGN KEY (working_pattern_id) REFERENCES public.working_patterns(id);





--

-- Name: department_payroll_reconciliation; Type: VIEW; Schema: public; Owner: -

--



CREATE VIEW public.department_payroll_reconciliation AS

 SELECT d.id AS department_id,

    d.name AS department_name,

    d.std_working_days AS legacy_std_working_days,

    latest.effective_from AS canonical_effective_from,

    latest.value_decimal AS canonical_std_working_days,

        CASE

            WHEN (d.std_working_days IS NULL) THEN 'missing_legacy_value'::text

            WHEN (latest.department_id IS NULL) THEN 'missing_canonical_config'::text

            WHEN (latest.value_decimal = d.std_working_days) THEN 'matched'::text

            ELSE 'value_mismatch'::text

        END AS reconciliation_status

   FROM (public.departments d

     LEFT JOIN LATERAL ( SELECT ce.scope_id AS department_id,

            ce.effective_from,

            ce.value_decimal

           FROM public.config_entries ce

          WHERE (((ce.scope_type)::text = 'department'::text) AND (ce.scope_id = d.id) AND ((ce.config_key)::text = 'payroll.fixed_std_days'::text))

          ORDER BY ce.effective_from DESC, ce.created_at DESC, ce.id DESC

         LIMIT 1) latest ON (true))

  WHERE (d.deleted_at IS NULL);





--

INSERT INTO public.schema_migrations(version) VALUES ('fresh_payroll_v1');

COMMIT;
