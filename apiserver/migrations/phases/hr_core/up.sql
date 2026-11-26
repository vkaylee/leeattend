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

-- Fresh install phase: hr_core
CREATE TABLE public.departments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    manager_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    std_working_days numeric(5,2),
    deleted_at timestamp with time zone,
    default_working_pattern_id uuid
);

CREATE TABLE public.econtract_envelopes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    contract_id uuid NOT NULL,
    employer_id uuid NOT NULL,
    provider_type character varying(50) NOT NULL,
    external_envelope_id character varying(255) NOT NULL,
    status character varying(50) DEFAULT 'draft'::character varying NOT NULL,
    signer_statuses jsonb DEFAULT '[]'::jsonb NOT NULL,
    audit_trail_metadata jsonb,
    signed_document_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.employee_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
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
    relationship_id uuid,
    CONSTRAINT chk_contract_dates CHECK (((end_date IS NULL) OR (start_date <= end_date))),
    CONSTRAINT chk_indefinite_end_date CHECK ((((contract_type = 'indefinite'::public.contract_type_enum) AND (end_date IS NULL)) OR (contract_type <> 'indefinite'::public.contract_type_enum)))
);

CREATE TABLE public.employee_documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    tenure_id uuid,
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
    employment_id uuid NOT NULL,
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
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    avatar_file_id uuid,
    national_id_lookup_hash character(64),
    phone_lookup_hash character(64),
    CONSTRAINT employee_profiles_identity_lookup_hash_check CHECK (((national_id_lookup_hash IS NULL) OR ((national_id_lookup_hash)::text ~ '^[0-9a-f]{64}$'::text)) AND ((phone_lookup_hash IS NULL) OR ((phone_lookup_hash)::text ~ '^[0-9a-f]{64}$'::text)))
);

CREATE TABLE public.employee_salary_components (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    component_definition_id uuid NOT NULL,
    amount numeric(15,2) DEFAULT 0.0 NOT NULL,
    effective_date date NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    employment_id uuid NOT NULL
);

CREATE TABLE public.employee_tenures (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    employee_number character varying(50) NOT NULL,
    employee_number_normalized text GENERATED ALWAYS AS (upper(btrim((employee_number)::text))) STORED NOT NULL,
    user_id uuid,
    hire_type character varying(20) DEFAULT 'initial'::character varying NOT NULL,
    prior_service_credit_days integer DEFAULT 0 NOT NULL,
    start_date date NOT NULL,
    end_date date,
    termination_reason text,
    rehire_eligible boolean DEFAULT true NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    CONSTRAINT employee_tenures_dates_check CHECK (((end_date IS NULL) OR (start_date <= end_date))),
    CONSTRAINT employee_tenures_hire_type_check CHECK (((hire_type)::text = ANY ((ARRAY['initial'::character varying, 'rehire'::character varying])::text[]))),
    CONSTRAINT employee_tenures_number_check CHECK ((length(btrim((employee_number)::text)) > 0)),
    CONSTRAINT employee_tenures_prior_service_check CHECK ((prior_service_credit_days >= 0))
);

CREATE TABLE public.employees (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    tags text[] DEFAULT '{}'::text[],
    full_name character varying(255) NOT NULL,
    recruitment_source_id uuid,
    deleted_at timestamp with time zone,
    CONSTRAINT chk_employee_full_name CHECK ((length((full_name)::text) > 0))
);

CREATE TABLE public.employer_econtract_configs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employer_id uuid NOT NULL,
    provider_type character varying(50) NOT NULL,
    api_url character varying(255) NOT NULL,
    client_id character varying(255) NOT NULL,
    client_secret_encrypted text NOT NULL,
    webhook_secret_encrypted text,
    hsm_key_alias character varying(100),
    default_auth_type character varying(50) DEFAULT 'sms'::character varying NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.employer_econtract_routing_rules (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employer_id uuid NOT NULL,
    signer_country character varying(10) NOT NULL,
    signing_method character varying(50) NOT NULL,
    target_provider_type character varying(50) NOT NULL,
    priority integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.employers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(255) NOT NULL,
    tax_code character varying(50),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    legal_address character varying(255),
    deleted_at timestamp with time zone,
    parent_id uuid,
    relationship_type character varying(100),
    business_type character varying(50),
    registration_number character varying(100),
    phone character varying(20),
    email character varying(255),
    website character varying(255),
    incorporation_date date,
    country_code character varying(3),
    currency_code character varying(3) DEFAULT 'VND'::character varying
);

CREATE TABLE public.employment_phase_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    description text,
    phase_type text DEFAULT 'custom'::text NOT NULL,
    rule_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    display_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE public.employment_relationships (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    employer_id uuid NOT NULL,
    status character varying(50) DEFAULT 'active'::character varying NOT NULL,
    start_date date NOT NULL,
    end_date date,
    termination_reason character varying(50),
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    tenure_id uuid NOT NULL,
    CONSTRAINT employment_relationships_dates_check CHECK (((end_date IS NULL) OR (start_date <= end_date)))
);

CREATE TABLE public.employments (
    id uuid NOT NULL,
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
    manager_employment_id uuid,
    notes text,
    termination_reason character varying(50),
    relationship_id uuid NOT NULL,
    CONSTRAINT employments_dates_check CHECK (((end_date IS NULL) OR (start_date <= end_date)))
);

CREATE TABLE public.leave_accrual_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_assignment_id uuid,
    accrual_date date NOT NULL,
    amount numeric(5,2) NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    leave_type_id uuid NOT NULL,
    employment_id uuid NOT NULL
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
    assignee_type text NOT NULL,
    assignee_id uuid,
    policy_id uuid NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    deleted_at timestamp with time zone,
    CONSTRAINT leave_policy_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['organization'::text, 'department'::text, 'employee'::text, 'user'::text])))
);

CREATE TABLE public.leave_policy_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
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
    attachment_file_ids uuid[] DEFAULT '{}'::uuid[],
    CONSTRAINT chk_start_before_end CHECK ((start_date <= end_date))
);

CREATE TABLE public.leave_types (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code character varying(50) NOT NULL,
    name text NOT NULL,
    is_paid boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    policy_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    deleted_at timestamp with time zone
);

CREATE TABLE public.overtime_policy_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public.overtime_policy_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_id uuid NOT NULL,
    version_no integer NOT NULL,
    status text DEFAULT 'draft'::text NOT NULL,
    row_version bigint DEFAULT 1 NOT NULL,
    daily_cap_minutes integer,
    monthly_cap_hours numeric(8,2),
    yearly_cap_hours numeric(8,2),
    rules jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    published_by uuid,
    published_at timestamp with time zone,
    CONSTRAINT overtime_policy_versions_version_no_check CHECK (version_no >= 1),
    CONSTRAINT overtime_policy_versions_row_version_check CHECK (row_version >= 1),
    CONSTRAINT overtime_policy_versions_status_check CHECK (status IN ('draft', 'published', 'archived')),
    CONSTRAINT overtime_policy_versions_caps_check CHECK (
        (daily_cap_minutes IS NULL OR daily_cap_minutes >= 0)
        AND (monthly_cap_hours IS NULL OR monthly_cap_hours >= 0)
        AND (yearly_cap_hours IS NULL OR yearly_cap_hours >= 0)
    ),
    CONSTRAINT overtime_policy_versions_rules_array_check CHECK (jsonb_typeof(rules) = 'array'),
    CONSTRAINT overtime_policy_versions_published_meta_check CHECK (
        (status = 'draft' AND published_by IS NULL AND published_at IS NULL)
        OR (status IN ('published', 'archived') AND published_by IS NOT NULL AND published_at IS NOT NULL)
    )
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
    CONSTRAINT overtime_request_employees_consent_status_check CHECK (((consent_status)::text = ANY (ARRAY[('pending'::character varying)::text, ('accepted'::character varying)::text, ('declined'::character varying)::text])))
);

CREATE TABLE public.overtime_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
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
    attachment_file_ids uuid[],
    CONSTRAINT overtime_requests_planned_minutes_check CHECK ((planned_minutes > 0))
);

CREATE TABLE public.recruitment_sources (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
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

CREATE TABLE public.salary_component_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(100) NOT NULL,
    code character varying(50) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    component_type text DEFAULT 'allowance'::text NOT NULL,
    linked_rule_id uuid
);

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.econtract_envelopes
    ADD CONSTRAINT econtract_envelopes_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.econtract_envelopes
    ADD CONSTRAINT uq_envelope_provider UNIQUE (provider_type, external_envelope_id);

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_pkey PRIMARY KEY (id);

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

ALTER TABLE ONLY public.employee_tenures
    ADD CONSTRAINT employee_tenures_identity_key UNIQUE (id, employee_id);

ALTER TABLE ONLY public.employee_tenures
    ADD CONSTRAINT employee_tenures_permanent_code_owner EXCLUDE USING gist (employee_number_normalized WITH =, employee_id WITH <>);

ALTER TABLE ONLY public.employee_tenures
    ADD CONSTRAINT employee_tenures_no_overlap_person EXCLUDE USING gist (employee_id WITH =, daterange(start_date, COALESCE((end_date + 1), 'infinity'::date), '[)'::text) WITH &&) WHERE ((deleted_at IS NULL));

ALTER TABLE ONLY public.employee_tenures
    ADD CONSTRAINT employee_tenures_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employer_econtract_configs
    ADD CONSTRAINT employer_econtract_configs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employer_econtract_configs
    ADD CONSTRAINT uq_employer_provider UNIQUE (employer_id, provider_type);

ALTER TABLE ONLY public.employer_econtract_routing_rules
    ADD CONSTRAINT employer_econtract_routing_rules_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employers
    ADD CONSTRAINT employers_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_organization_name_key UNIQUE (name);

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employment_relationships
    ADD CONSTRAINT employment_relationships_identity_key UNIQUE (id, employee_id, employer_id);

ALTER TABLE ONLY public.employment_relationships
    ADD CONSTRAINT employment_relationships_no_overlap EXCLUDE USING gist (employee_id WITH =, employer_id WITH =, daterange(start_date, COALESCE((end_date + 1), 'infinity'::date), '[)'::text) WITH &&) WHERE ((deleted_at IS NULL));

ALTER TABLE ONLY public.employment_relationships
    ADD CONSTRAINT employment_relationships_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_employment_leave_type_id_date_key UNIQUE (employment_id, leave_type_id, accrual_date);

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_employment_id_leave_type_id_year_key UNIQUE (employment_id, leave_type_id, year);

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_organization_assignee_type_coales_excl EXCLUDE USING gist (assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&) WHERE ((deleted_at IS NULL));

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_policy_definitions
    ADD CONSTRAINT leave_policy_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_no_overlap_active EXCLUDE USING gist (employment_id WITH =, daterange(start_date, end_date, '[]'::text) WITH &&) WHERE ((status = ANY (ARRAY['approved'::public.leave_status_type, 'pre_approved'::public.leave_status_type])));

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.leave_types
    ADD CONSTRAINT leave_types_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_policy_definitions
    ADD CONSTRAINT overtime_policy_definitions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_policy_versions
    ADD CONSTRAINT overtime_policy_versions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_policy_versions
    ADD CONSTRAINT overtime_policy_versions_policy_version_unique UNIQUE (policy_id, version_no);

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_overtime_request_id_employment_id_ke UNIQUE (overtime_request_id, employment_id);

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.recruitment_sources
    ADD CONSTRAINT recruitment_sources_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_organization_code_key UNIQUE (code);

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_pkey PRIMARY KEY (id);

CREATE INDEX idx_departments_manager_id ON public.departments USING btree (manager_id);

CREATE INDEX idx_econtract_envelopes_contract ON public.econtract_envelopes USING btree (contract_id);

CREATE INDEX idx_econtract_envelopes_external_id ON public.econtract_envelopes USING btree (external_envelope_id);

CREATE INDEX idx_econtract_routing_rules ON public.employer_econtract_routing_rules USING btree (employer_id, signer_country);

CREATE INDEX idx_employee_assignments_employment_id ON public.employee_assignments USING btree (employment_id);

CREATE INDEX idx_employee_contracts_dates ON public.employee_contracts USING btree (start_date, end_date);

CREATE INDEX idx_employee_contracts_employment_id ON public.employee_contracts USING btree (employment_id);

CREATE INDEX idx_employee_contracts_relationship_id ON public.employee_contracts USING btree (relationship_id) WHERE ((relationship_id IS NOT NULL) AND (deleted_at IS NULL));

CREATE INDEX idx_employee_contracts_status ON public.employee_contracts USING btree (status);

CREATE INDEX idx_employee_documents_employee_id ON public.employee_documents USING btree (employee_id);

CREATE INDEX idx_employee_documents_expires_at ON public.employee_documents USING btree (expires_at) WHERE ((expires_at IS NOT NULL) AND (is_active = true));

CREATE INDEX idx_employee_documents_type ON public.employee_documents USING btree (document_type);

CREATE INDEX idx_employee_emergency_contacts_employee_id ON public.employee_emergency_contacts USING btree (employee_id);

CREATE UNIQUE INDEX idx_employee_emergency_contacts_primary ON public.employee_emergency_contacts USING btree (employee_id) WHERE (is_primary = true);

CREATE INDEX idx_employee_phases_active_lookup ON public.employee_phases USING btree (employment_id, status, start_date, end_date) WHERE (status = 'active'::text);

CREATE INDEX idx_employee_phases_definition ON public.employee_phases USING btree (phase_definition_id);

CREATE INDEX idx_employee_phases_employment ON public.employee_phases USING btree (employment_id);

CREATE INDEX idx_employee_phases_status ON public.employee_phases USING btree (status);

CREATE INDEX idx_employee_profiles_employee_id ON public.employee_profiles USING btree (employee_id);

CREATE INDEX idx_employee_profiles_national_id_lookup_hash ON public.employee_profiles USING btree (national_id_lookup_hash) WHERE (national_id_lookup_hash IS NOT NULL);

CREATE INDEX idx_employee_profiles_phone_lookup_hash ON public.employee_profiles USING btree (phone_lookup_hash) WHERE (phone_lookup_hash IS NOT NULL);

CREATE INDEX idx_employee_salary_comp_def_id ON public.employee_salary_components USING btree (component_definition_id);

CREATE INDEX idx_employee_salary_components_effective_date ON public.employee_salary_components USING btree (effective_date);

CREATE INDEX idx_employee_salary_components_employment_id ON public.employee_salary_components USING btree (employment_id);

CREATE INDEX idx_employee_documents_tenure_id ON public.employee_documents USING btree (tenure_id) WHERE (tenure_id IS NOT NULL);

CREATE UNIQUE INDEX idx_employee_tenures_active_user ON public.employee_tenures USING btree (user_id) WHERE ((user_id IS NOT NULL) AND (end_date IS NULL) AND (deleted_at IS NULL));

CREATE INDEX idx_employee_tenures_employee ON public.employee_tenures USING btree (employee_id, start_date DESC, id DESC) WHERE (deleted_at IS NULL);

CREATE INDEX idx_employee_tenures_number ON public.employee_tenures USING btree (employee_number, start_date DESC, id DESC) WHERE (deleted_at IS NULL);

CREATE INDEX idx_employee_tenures_number_normalized ON public.employee_tenures USING btree (employee_number_normalized, start_date DESC, id DESC) WHERE (deleted_at IS NULL);

CREATE INDEX idx_employee_tenures_number_search ON public.employee_tenures USING gin (employee_number public.gin_trgm_ops);

CREATE INDEX idx_employees_name_search ON public.employees USING gin (full_name public.gin_trgm_ops);

CREATE INDEX idx_employees_recruitment_source ON public.employees USING btree (recruitment_source_id);

CREATE INDEX idx_employees_tags ON public.employees USING gin (tags);

CREATE INDEX idx_file_acknowledgements_tenure_id ON public.file_acknowledgements USING btree (tenure_id) WHERE (tenure_id IS NOT NULL);

CREATE INDEX idx_employer_econtract_configs_lookup ON public.employer_econtract_configs USING btree (employer_id);

CREATE INDEX idx_employers_parent_id ON public.employers USING btree (parent_id);

CREATE INDEX idx_employment_phase_defs_rule_config ON public.employment_phase_definitions USING gin (rule_config);

CREATE INDEX idx_employment_phase_defs_type ON public.employment_phase_definitions USING btree (phase_type);

CREATE INDEX idx_employment_relationships_employee ON public.employment_relationships USING btree (employee_id, start_date DESC, id DESC) WHERE (deleted_at IS NULL);

CREATE INDEX idx_employment_relationships_employer ON public.employment_relationships USING btree (employer_id, start_date DESC, id DESC) WHERE (deleted_at IS NULL);

CREATE INDEX idx_employment_relationships_tenure_id ON public.employment_relationships USING btree (tenure_id, start_date DESC, id DESC) WHERE (deleted_at IS NULL);

CREATE INDEX idx_employments_date_range ON public.employments USING btree (start_date, end_date);

CREATE INDEX idx_employments_employee_id ON public.employments USING btree (employee_id);

CREATE INDEX idx_employments_employer_id ON public.employments USING btree (employer_id);

CREATE INDEX idx_employments_query ON public.employments USING btree (employee_id, start_date, end_date) WHERE (deleted_at IS NULL);

CREATE INDEX idx_employments_relationship_id ON public.employments USING btree (relationship_id) WHERE ((relationship_id IS NOT NULL) AND (deleted_at IS NULL));

CREATE INDEX idx_leave_accrual_logs_leave_type_id ON public.leave_accrual_logs USING btree (leave_type_id);

CREATE INDEX idx_leave_accrual_logs_policy_assignment_id ON public.leave_accrual_logs USING btree (policy_assignment_id);

CREATE INDEX idx_leave_balances_employment_id ON public.leave_balances USING btree (employment_id);

CREATE INDEX idx_leave_balances_leave_type_id ON public.leave_balances USING btree (leave_type_id);

CREATE INDEX idx_leave_policy_assignments_assignee ON public.leave_policy_assignments USING btree (assignee_type, assignee_id);

CREATE INDEX idx_leave_policy_assignments_policy_id ON public.leave_policy_assignments USING btree (policy_id);

CREATE UNIQUE INDEX idx_leave_policy_definitions_ws_name ON public.leave_policy_definitions USING btree (name);

CREATE INDEX idx_leave_policy_defs_rule_config ON public.leave_policy_definitions USING gin (rule_config);

CREATE INDEX idx_leave_requests_approved_by ON public.leave_requests USING btree (approved_by_user_id);

CREATE INDEX idx_leave_requests_date_range ON public.leave_requests USING btree (start_date, end_date);

CREATE INDEX idx_leave_requests_employment_id ON public.leave_requests USING btree (employment_id);

CREATE INDEX idx_leave_requests_leave_type_id ON public.leave_requests USING btree (leave_type_id);

CREATE INDEX idx_leave_requests_requested_by ON public.leave_requests USING btree (requested_by_user_id);

CREATE INDEX idx_leave_requests_status ON public.leave_requests USING btree (status);

CREATE INDEX idx_leave_types_policy_config ON public.leave_types USING gin (policy_config);

CREATE INDEX idx_otr_dept ON public.overtime_requests USING btree (department_id);

CREATE INDEX idx_otr_organization_date ON public.overtime_requests USING btree (planned_date);

CREATE INDEX idx_otr_requested_by ON public.overtime_requests USING btree (requested_by_user_id);

CREATE INDEX idx_otr_status ON public.overtime_requests USING btree (status);

CREATE INDEX idx_otre_consent ON public.overtime_request_employees USING btree (consent_status);

CREATE INDEX idx_otre_request ON public.overtime_request_employees USING btree (overtime_request_id);

CREATE INDEX idx_overtime_policy_versions_policy_status ON public.overtime_policy_versions USING btree (policy_id, status, version_no DESC);

CREATE INDEX idx_overtime_request_employees_employment_id ON public.overtime_request_employees USING btree (employment_id);

CREATE INDEX idx_salary_comp_defs_linked_rule ON public.salary_component_definitions USING btree (linked_rule_id) WHERE (linked_rule_id IS NOT NULL);

CREATE UNIQUE INDEX leave_types_organization_code_key ON public.leave_types USING btree (lower((code)::text));

CREATE UNIQUE INDEX uq_departments_name ON public.departments USING btree (lower((name)::text)) WHERE (deleted_at IS NULL);

CREATE UNIQUE INDEX uq_leave_policy_definitions_name ON public.leave_policy_definitions USING btree (lower(name)) WHERE (deleted_at IS NULL);

CREATE TRIGGER trg_cascade_delete_dept_assignments BEFORE DELETE ON public.departments FOR EACH ROW EXECUTE FUNCTION public.fn_cascade_delete_department_assignments();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.departments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_validate_contract_relationship_link BEFORE INSERT OR UPDATE OF employment_id, relationship_id ON public.employee_contracts FOR EACH ROW EXECUTE FUNCTION public.fn_validate_contract_relationship_link();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_phases FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_profiles FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_salary_components FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_roster_cache_invalidation_employee_tenures AFTER INSERT OR DELETE OR UPDATE ON public.employee_tenures FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_tenures FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_validate_tenure_bounds_guard BEFORE UPDATE OF start_date, end_date, deleted_at ON public.employee_tenures FOR EACH ROW EXECUTE FUNCTION public.fn_validate_tenure_bounds_guard();

CREATE TRIGGER trg_cascade_delete_emp_assignments BEFORE DELETE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.fn_cascade_delete_employee_assignments();

CREATE TRIGGER trg_roster_cache_invalidation_employees AFTER INSERT OR DELETE OR UPDATE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employment_phase_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employment_relationships FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_validate_relationship_tenure_bounds BEFORE INSERT OR UPDATE OF tenure_id, employee_id, start_date, end_date, deleted_at ON public.employment_relationships FOR EACH ROW EXECUTE FUNCTION public.fn_validate_relationship_tenure_bounds();

CREATE TRIGGER trg_roster_cache_invalidation_employments AFTER INSERT OR DELETE OR UPDATE ON public.employments FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_validate_segment_relationship_bounds BEFORE INSERT OR UPDATE OF relationship_id, employee_id, employer_id, start_date, end_date, deleted_at ON public.employments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_segment_relationship_bounds();

CREATE TRIGGER trg_validate_segment_relationship_change BEFORE UPDATE OF relationship_id ON public.employments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_segment_relationship_change();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_balances FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.leave_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_policy_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.overtime_policy_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE OR REPLACE FUNCTION public.fn_guard_overtime_policy_version_immutability() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status IN ('published', 'archived') THEN
            RAISE EXCEPTION 'published or archived overtime policy version cannot be deleted'
                USING ERRCODE = '23514';
        END IF;
        RETURN OLD;
    END IF;

    IF OLD.status = 'archived' THEN
        RAISE EXCEPTION 'archived overtime policy version is immutable'
            USING ERRCODE = '23514';
    END IF;

    IF OLD.status = 'published' THEN
        IF NEW.status NOT IN ('published', 'archived') THEN
            RAISE EXCEPTION 'published overtime policy version cannot revert to draft'
                USING ERRCODE = '23514';
        END IF;

        IF NEW.policy_id IS DISTINCT FROM OLD.policy_id
           OR NEW.version_no IS DISTINCT FROM OLD.version_no
           OR NEW.daily_cap_minutes IS DISTINCT FROM OLD.daily_cap_minutes
           OR NEW.monthly_cap_hours IS DISTINCT FROM OLD.monthly_cap_hours
           OR NEW.yearly_cap_hours IS DISTINCT FROM OLD.yearly_cap_hours
           OR NEW.rules IS DISTINCT FROM OLD.rules
           OR NEW.created_by IS DISTINCT FROM OLD.created_by
           OR NEW.created_at IS DISTINCT FROM OLD.created_at
           OR NEW.published_by IS DISTINCT FROM OLD.published_by
           OR NEW.published_at IS DISTINCT FROM OLD.published_at THEN
            RAISE EXCEPTION 'published overtime policy version rules are immutable'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.overtime_policy_versions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_guard_overtime_policy_version_immutability BEFORE UPDATE OR DELETE ON public.overtime_policy_versions FOR EACH ROW EXECUTE FUNCTION public.fn_guard_overtime_policy_version_immutability();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.recruitment_sources FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.salary_component_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT behalf_assignments_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT ot_claim_behalf_assignments_target_department_id_fkey FOREIGN KEY (target_department_id) REFERENCES public.departments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT ot_claim_behalf_assignments_target_employee_id_fkey FOREIGN KEY (target_employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_manager_id_fkey FOREIGN KEY (manager_id) REFERENCES public.employees(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.econtract_envelopes
    ADD CONSTRAINT econtract_envelopes_contract_id_fkey FOREIGN KEY (contract_id) REFERENCES public.employee_contracts(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.econtract_envelopes
    ADD CONSTRAINT econtract_envelopes_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;

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

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_relationship_id_fkey FOREIGN KEY (relationship_id) REFERENCES public.employment_relationships(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_tenure_identity_fkey FOREIGN KEY (tenure_id, employee_id) REFERENCES public.employee_tenures(id, employee_id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_emergency_contacts
    ADD CONSTRAINT employee_emergency_contacts_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_phase_definition_id_fkey FOREIGN KEY (phase_definition_id) REFERENCES public.employment_phase_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_avatar_file_id_fkey FOREIGN KEY (avatar_file_id) REFERENCES public.files(id);

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_salary_components
    ADD CONSTRAINT employee_salary_components_component_definition_id_fkey FOREIGN KEY (component_definition_id) REFERENCES public.salary_component_definitions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_salary_components
    ADD CONSTRAINT employee_salary_components_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employee_tenures
    ADD CONSTRAINT employee_tenures_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_tenures
    ADD CONSTRAINT employee_tenures_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_recruitment_source_id_fkey FOREIGN KEY (recruitment_source_id) REFERENCES public.recruitment_sources(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employer_econtract_configs
    ADD CONSTRAINT employer_econtract_configs_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employer_econtract_routing_rules
    ADD CONSTRAINT employer_econtract_routing_rules_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.employers
    ADD CONSTRAINT employers_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.employers(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.employment_relationships
    ADD CONSTRAINT employment_relationships_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employment_relationships
    ADD CONSTRAINT employment_relationships_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employment_relationships
    ADD CONSTRAINT employment_relationships_tenure_identity_fkey FOREIGN KEY (tenure_id, employee_id) REFERENCES public.employee_tenures(id, employee_id) ON DELETE RESTRICT;

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

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_relationship_id_fkey FOREIGN KEY (relationship_id) REFERENCES public.employment_relationships(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_relationship_identity_fkey FOREIGN KEY (relationship_id, employee_id, employer_id) REFERENCES public.employment_relationships(id, employee_id, employer_id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.file_acknowledgements
    ADD CONSTRAINT file_acknowledgements_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.file_acknowledgements
    ADD CONSTRAINT file_acknowledgements_tenure_identity_fkey FOREIGN KEY (tenure_id, employee_id) REFERENCES public.employee_tenures(id, employee_id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;

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

ALTER TABLE ONLY public.overtime_policy_versions
    ADD CONSTRAINT overtime_policy_versions_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.overtime_policy_definitions(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.overtime_policy_versions
    ADD CONSTRAINT overtime_policy_versions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.overtime_policy_versions
    ADD CONSTRAINT overtime_policy_versions_published_by_fkey FOREIGN KEY (published_by) REFERENCES public.users(id);

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id);

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.approval_workflows(id);

ALTER TABLE ONLY public.recruitment_sources
    ADD CONSTRAINT recruitment_sources_referrer_employee_id_fkey FOREIGN KEY (referrer_employee_id) REFERENCES public.employees(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.signature_requests
    ADD CONSTRAINT signature_requests_signer_id_fkey FOREIGN KEY (signer_id) REFERENCES public.employees(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

INSERT INTO public.schema_migrations(version) VALUES ('fresh_hr_core_v1');

COMMIT;
