-- LeeAttend single-tenant baseline migration.
-- System and tenant objects share one PostgreSQL database.
-- Workspace columns remain in this phase for runtime compatibility; their
-- removal is coupled to backend/API contract changes in later phases.

--
-- PostgreSQL database dump
--

--
-- Name: fn_set_updated_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE OR REPLACE FUNCTION public.fn_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

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
    workspace_id uuid NOT NULL,
    last_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    max_concurrent_workers integer DEFAULT 2 NOT NULL,
    success_cooldown_secs integer DEFAULT 300 NOT NULL,
    retention_days integer DEFAULT 365 NOT NULL
);

--
-- Name: infra_job_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.infra_job_events (
    id uuid NOT NULL,
    job_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
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
-- Name: TABLE infra_job_events; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.infra_job_events IS 'Append-only, PII-safe lifecycle transitions for asynchronous jobs.';

--
-- Name: infra_job_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.infra_job_records (
    id uuid NOT NULL,
    workspace_id uuid NOT NULL,
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
    CONSTRAINT infra_job_records_state_check CHECK (((state)::text = ANY ((ARRAY['dispatching'::character varying, 'queued'::character varying, 'processing'::character varying, 'completed'::character varying, 'failed'::character varying, 'retryable'::character varying, 'dead_letter'::character varying, 'lost'::character varying])::text[])))
);

--
-- Name: TABLE infra_job_records; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.infra_job_records IS 'Authoritative operational lifecycle for asynchronous jobs. Raw payloads and PII are intentionally excluded.';

--
-- Name: member_extra_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.member_extra_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    assigned_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    expires_at timestamp with time zone NOT NULL,
    role_id uuid NOT NULL
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
    CONSTRAINT monitor_alert_events_severity_check CHECK (((severity)::text = ANY ((ARRAY['info'::character varying, 'warning'::character varying, 'critical'::character varying])::text[])))
);

--
-- Name: TABLE monitor_alert_events; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.monitor_alert_events IS 'Append-only, PII-safe lifecycle events for persisted monitor alerts.';

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
    CONSTRAINT monitor_alerts_severity_check CHECK (((severity)::text = ANY ((ARRAY['info'::character varying, 'warning'::character varying, 'critical'::character varying])::text[]))),
    CONSTRAINT monitor_alerts_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'resolved'::character varying])::text[])))
);

--
-- Name: TABLE monitor_alerts; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.monitor_alerts IS 'Persisted, PII-safe infrastructure monitor alerts. Raw evaluator payloads are intentionally excluded.';

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
-- Name: storage_configs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.storage_configs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
-- Name: system_db_clusters; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.system_db_clusters (
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    admin_database_url character varying(255) NOT NULL,
    region character varying(50) DEFAULT 'ap-southeast-1'::character varying NOT NULL,
    max_tenants integer DEFAULT 1000 NOT NULL,
    active_tenants integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
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
-- Name: system_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.system_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
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
-- Name: user_system_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_system_roles (
    user_id uuid NOT NULL,
    role_id uuid NOT NULL,
    assigned_by uuid,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL
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
-- Name: workspace_member_extra_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.workspace_member_extra_permissions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    permission text NOT NULL,
    assigned_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    expires_at timestamp with time zone NOT NULL,
    assigned_by uuid
);

--
-- Name: workspace_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.workspace_members (
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    role_id uuid NOT NULL
);

--
-- Name: workspace_role_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.workspace_role_permissions (
    role_id uuid NOT NULL,
    permission_name text NOT NULL
);

--
-- Name: workspace_roles; Type: TABLE; Schema: public; Owner: -
--

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
    CONSTRAINT workspace_roles_scope_type_check CHECK (((scope_type)::text = ANY (ARRAY[('global'::character varying)::text, ('scoped'::character varying)::text])))
);

--
-- Name: workspaces; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.workspaces (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(100) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    date_format character varying(20) DEFAULT 'DD/MM/YYYY'::character varying NOT NULL,
    time_format character varying(10) DEFAULT '24h'::character varying NOT NULL,
    timezone character varying(50) DEFAULT 'Asia/Ho_Chi_Minh'::character varying NOT NULL,
    theme_primary_color text DEFAULT '#7c3aed'::text,
    default_language character varying(10) DEFAULT 'vi'::character varying NOT NULL,
    logo_url text,
    tagline text,
    custom_db_url_encrypted text,
    CONSTRAINT chk_workspace_name CHECK ((length((name)::text) > 0))
);

--
-- Name: authorized_agents authorized_agents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.authorized_agents
    ADD CONSTRAINT authorized_agents_pkey PRIMARY KEY (id);

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
-- Name: member_extra_roles member_extra_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_pkey PRIMARY KEY (id);

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
-- Name: storage_configs storage_configs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_configs
    ADD CONSTRAINT storage_configs_pkey PRIMARY KEY (id);

--
-- Name: storage_configs storage_configs_workspace_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_configs
    ADD CONSTRAINT storage_configs_workspace_id_key UNIQUE (workspace_id);

--
-- Name: system_audit_logs system_audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_audit_logs
    ADD CONSTRAINT system_audit_logs_pkey PRIMARY KEY (id);

--
-- Name: system_db_clusters system_db_clusters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_db_clusters
    ADD CONSTRAINT system_db_clusters_pkey PRIMARY KEY (id);

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
-- Name: system_roles system_roles_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_roles
    ADD CONSTRAINT system_roles_name_key UNIQUE (name);

--
-- Name: system_roles system_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_roles
    ADD CONSTRAINT system_roles_pkey PRIMARY KEY (id);

--
-- Name: system_settings system_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_settings
    ADD CONSTRAINT system_settings_pkey PRIMARY KEY (key);

--
-- Name: user_system_roles user_system_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_pkey PRIMARY KEY (user_id, role_id);

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
-- Name: workspace_member_extra_permissions workspace_member_extra_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_permissions_pkey PRIMARY KEY (id);

--
-- Name: workspace_members workspace_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_pkey PRIMARY KEY (workspace_id, user_id);

--
-- Name: workspace_role_permissions workspace_role_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_role_permissions
    ADD CONSTRAINT workspace_role_permissions_pkey PRIMARY KEY (role_id, permission_name);

--
-- Name: workspace_roles workspace_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_roles
    ADD CONSTRAINT workspace_roles_pkey PRIMARY KEY (id);

--
-- Name: workspaces workspaces_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspaces
    ADD CONSTRAINT workspaces_pkey PRIMARY KEY (id);

--
-- Name: idx_authorized_agents_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_authorized_agents_workspace ON public.authorized_agents USING btree (workspace_id);

--
-- Name: idx_extra_perms_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_extra_perms_lookup ON public.workspace_member_extra_permissions USING btree (workspace_id, user_id, expires_at);

--
-- Name: idx_infra_job_events_job_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_infra_job_events_job_created ON public.infra_job_events USING btree (job_id, created_at, id);

--
-- Name: idx_infra_job_events_workspace_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_infra_job_events_workspace_created ON public.infra_job_events USING btree (workspace_id, created_at DESC);

--
-- Name: idx_infra_job_records_monitor_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_infra_job_records_monitor_created ON public.infra_job_records USING btree (created_at DESC, id DESC);

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
-- Name: idx_infra_job_records_workspace_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_infra_job_records_workspace_created ON public.infra_job_records USING btree (workspace_id, created_at DESC, id DESC);

--
-- Name: idx_member_extra_roles_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_member_extra_roles_lookup ON public.member_extra_roles USING btree (workspace_id, user_id, expires_at);

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
-- Name: idx_system_audit_logs_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_system_audit_logs_action ON public.system_audit_logs USING btree (action);

--
-- Name: idx_system_audit_logs_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_system_audit_logs_created_at ON public.system_audit_logs USING btree (created_at DESC);

--
-- Name: idx_system_roles_unique_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_system_roles_unique_name ON public.workspace_roles USING btree (name) WHERE (workspace_id IS NULL);

--
-- Name: idx_user_system_roles_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_system_roles_role_id ON public.user_system_roles USING btree (role_id);

--
-- Name: idx_user_system_roles_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_system_roles_user_id ON public.user_system_roles USING btree (user_id);

--
-- Name: idx_workspace_members_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_workspace_members_user_id ON public.workspace_members USING btree (user_id);

--
-- Name: idx_workspace_members_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_workspace_members_workspace_id ON public.workspace_members USING btree (workspace_id);

--
-- Name: idx_workspace_roles_scope_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_workspace_roles_scope_type ON public.workspace_roles USING btree (scope_type);

--
-- Name: idx_workspace_roles_unique_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_workspace_roles_unique_name ON public.workspace_roles USING btree (workspace_id, name) WHERE (workspace_id IS NOT NULL);

--
-- Name: uq_monitor_alerts_active_fingerprint; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_monitor_alerts_active_fingerprint ON public.monitor_alerts USING btree (fingerprint) WHERE (resolved_at IS NULL);

--
-- Name: infra_job_records trg_infra_job_records_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_infra_job_records_updated_at BEFORE UPDATE ON public.infra_job_records FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: monitor_alerts trg_monitor_alerts_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_monitor_alerts_updated_at BEFORE UPDATE ON public.monitor_alerts FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: system_settings trg_system_settings_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_system_settings_updated_at BEFORE UPDATE ON public.system_settings FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: workspace_roles trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.workspace_roles FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: storage_configs trg_updated_at_storage_configs; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at_storage_configs BEFORE UPDATE ON public.storage_configs FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: infra_job_events infra_job_events_job_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.infra_job_events
    ADD CONSTRAINT infra_job_events_job_id_fkey FOREIGN KEY (job_id) REFERENCES public.infra_job_records(id) ON DELETE RESTRICT;

--
-- Name: member_extra_roles member_extra_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id) ON DELETE CASCADE;

--
-- Name: member_extra_roles member_extra_roles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

--
-- Name: member_extra_roles member_extra_roles_workspace_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

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
-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

--
-- Name: storage_configs storage_configs_workspace_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_configs
    ADD CONSTRAINT storage_configs_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

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
-- Name: user_system_roles user_system_roles_assigned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES public.users(id) ON DELETE SET NULL;

--
-- Name: user_system_roles user_system_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.system_roles(id) ON DELETE CASCADE;

--
-- Name: user_system_roles user_system_roles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

--
-- Name: users users_avatar_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_avatar_file_id_fkey FOREIGN KEY (avatar_file_id) REFERENCES public.system_files(id);

--
-- Name: workspace_member_extra_permissions workspace_member_extra_perms_user_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_perms_user_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

--
-- Name: workspace_member_extra_permissions workspace_member_extra_perms_ws_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_perms_ws_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

--
-- Name: workspace_members workspace_members_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id);

--
-- Name: workspace_members workspace_members_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

--
-- Name: workspace_members workspace_members_workspace_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

--
-- Name: workspace_role_permissions workspace_role_permissions_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_role_permissions
    ADD CONSTRAINT workspace_role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id) ON DELETE CASCADE;

--
-- PostgreSQL database dump complete
--

--
-- PostgreSQL database dump
--

--
-- Name: btree_gist; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS btree_gist WITH SCHEMA public;

--
-- Name: EXTENSION btree_gist; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION btree_gist IS 'support for indexing common datatypes in GiST';

--
-- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;

--
-- Name: EXTENSION pg_trgm; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';

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
    ELSIF NEW.assignee_type = 'user' THEN
        -- User ID belongs to System DB, validated by application layer
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

CREATE OR REPLACE FUNCTION public.check_attendance_target_exists() RETURNS trigger
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

CREATE OR REPLACE FUNCTION public.enqueue_roster_cache_invalidation() RETURNS trigger
    LANGUAGE plpgsql
    SET search_path TO 'public'
    AS $$
DECLARE
    changed_row JSONB;
    workspace_id_value UUID;
    previous_workspace_id UUID;
BEGIN
    -- UPDATEs can move a source row between workspaces. Process both row
    -- images so the old workspace cannot retain a stale snapshot.
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
            -- Use the row's version_id directly. For DELETE, the interval row
            -- no longer exists when an AFTER trigger executes.
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

--
-- Name: fn_cascade_delete_department_assignments(); Type: FUNCTION; Schema: public; Owner: -
--

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

--
-- Name: fn_cascade_delete_employee_assignments(); Type: FUNCTION; Schema: public; Owner: -
--

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

--
-- Name: fn_set_updated_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE OR REPLACE FUNCTION public.fn_set_updated_at() RETURNS trigger
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

--
-- Name: fn_validate_target_fk(); Type: FUNCTION; Schema: public; Owner: -
--

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
-- Name: approval_action_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.approval_action_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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

--
-- Name: assignment_rules; Type: TABLE; Schema: public; Owner: -
--

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
    attachment_file_ids uuid[]
);

--
-- Name: attendance_daily_summaries; Type: TABLE; Schema: public; Owner: -
--

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
    employment_id uuid NOT NULL,
    standard_minutes integer DEFAULT 0 NOT NULL,
    early_ot_minutes integer DEFAULT 0 NOT NULL,
    late_ot_minutes integer DEFAULT 0 NOT NULL,
    break_ot_minutes integer DEFAULT 0 NOT NULL,
    is_weekend boolean DEFAULT false NOT NULL,
    late_minutes integer DEFAULT 0 NOT NULL,
    early_departure_minutes integer DEFAULT 0 NOT NULL
);

--
-- Name: attendance_import_jobs; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: attendance_log_errors; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: attendance_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attendance_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
    workspace_id uuid NOT NULL,
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
    CONSTRAINT attendance_source_pull_states_status_check CHECK (((status)::text = ANY ((ARRAY['idle'::character varying, 'running'::character varying, 'failed'::character varying])::text[])))
);

--
-- Name: attendance_sources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attendance_sources (
    id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    provider_source_id text NOT NULL,
    is_enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

--
-- Name: attendance_staging_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attendance_staging_logs (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT attendance_staging_logs_id_not_null1 NOT NULL,
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
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
    workspace_id uuid CONSTRAINT attendance_staging_logs_workspace_id_not_null1 NOT NULL,
    raw_user_id text CONSTRAINT attendance_staging_logs_raw_user_id_not_null1 NOT NULL,
    punch_time timestamp with time zone CONSTRAINT attendance_staging_logs_punch_time_not_null1 NOT NULL,
    raw_data jsonb,
    fingerprint text CONSTRAINT attendance_staging_logs_fingerprint_not_null1 NOT NULL,
    status text DEFAULT 'pending'::text CONSTRAINT attendance_staging_logs_status_not_null1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() CONSTRAINT attendance_staging_logs_created_at_not_null NOT NULL
);

--
-- Name: audit_logs; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_default; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m01; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m02; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m03; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m04; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m05; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m06; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m07; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m08; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m09; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m10; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m11; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: audit_logs_y2026m12; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: behalf_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.behalf_assignments (
    id uuid DEFAULT gen_random_uuid() CONSTRAINT ot_claim_behalf_assignments_id_not_null NOT NULL,
    workspace_id uuid CONSTRAINT ot_claim_behalf_assignments_workspace_id_not_null NOT NULL,
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
-- Name: condition_type_definitions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.condition_type_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
-- Name: COLUMN condition_type_definitions.code; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.condition_type_definitions.code IS 'Immutable uppercase machine key (e.g., EARLY_LEAVE, MATERNITY)';

--
-- Name: COLUMN condition_type_definitions.name_i18n; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.condition_type_definitions.name_i18n IS 'I18n names, e.g. {"en":"Early Leave","vi":"Về sớm"}';

--
-- Name: COLUMN condition_type_definitions.category; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.condition_type_definitions.category IS 'Grouping: working_hours, leave, general';

--
-- Name: COLUMN condition_type_definitions.affects_rules; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.condition_type_definitions.affects_rules IS 'JSON map of PolicyRules fields this type affects, e.g. {"authorized_early_leave_minutes":true,"is_early_leave_paid":true}';

--
-- Name: COLUMN condition_type_definitions.is_system; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.condition_type_definitions.is_system IS 'System types cannot be deleted by users';

--
-- Name: custom_field_definitions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.custom_field_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
    workspace_id uuid NOT NULL,
    active_view text DEFAULT 'overview'::text NOT NULL,
    visible_widgets jsonb DEFAULT '{}'::jsonb NOT NULL,
    widget_order jsonb DEFAULT '[]'::jsonb NOT NULL,
    collapsed_sections jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: departments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.departments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    manager_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    std_working_days numeric(5,2),
    deleted_at timestamp with time zone,
    default_working_pattern_id uuid
);

--
-- Name: document_folders; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.document_folders (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    parent_folder_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: econtract_envelopes; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: employee_assignments; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: employee_contracts; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: employee_documents; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: employee_emergency_contacts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.employee_emergency_contacts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    contact_name character varying(100) NOT NULL,
    relationship character varying(50) NOT NULL,
    phone character varying(20) NOT NULL,
    is_primary boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: employee_phases; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.employee_phases (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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

--
-- Name: employee_profiles; Type: TABLE; Schema: public; Owner: -
--

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
    avatar_file_id uuid
);

--
-- Name: employee_salary_components; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.employee_salary_components (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    component_definition_id uuid NOT NULL,
    amount numeric(15,2) DEFAULT 0.0 NOT NULL,
    effective_date date NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    employment_id uuid NOT NULL
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
-- Name: TABLE employee_shifts; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.employee_shifts IS 'DEPRECATED: This table has been replaced by employee_working_patterns (for pattern assignment) and work_segments (for scheduled work instances). Do not use for new features. Kept only for backward compatibility during migration period.';

--
-- Name: employee_working_patterns; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.employee_working_patterns (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
-- Name: COLUMN employee_working_patterns.anchor_date; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.employee_working_patterns.anchor_date IS 'Reference start date for rotation patterns. Determines phase offset per assignment.';

--
-- Name: employees; Type: TABLE; Schema: public; Owner: -
--

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
    hire_date date NOT NULL,
    original_hire_date date NOT NULL,
    termination_date date,
    is_active boolean DEFAULT true NOT NULL,
    CONSTRAINT chk_employee_full_name CHECK ((length((full_name)::text) > 0))
);

--
-- Name: employer_econtract_configs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.employer_econtract_configs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employer_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
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

--
-- Name: employer_econtract_routing_rules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.employer_econtract_routing_rules (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employer_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    signer_country character varying(10) NOT NULL,
    signing_method character varying(50) NOT NULL,
    target_provider_type character varying(50) NOT NULL,
    priority integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

--
-- Name: employers; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: employment_phase_definitions; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: employments; Type: TABLE; Schema: public; Owner: -
--

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
    manager_employment_id uuid,
    notes text,
    termination_reason character varying(50)
);

--
-- Name: external_attendance_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.external_attendance_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    external_event_id character varying(255) NOT NULL,
    occurred_at timestamp with time zone NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    received_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    source_id text NOT NULL
);

--
-- Name: feature_flags; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: file_acknowledgements; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.file_acknowledgements (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    document_id uuid NOT NULL,
    employee_id uuid NOT NULL,
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    acknowledged_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: holidays; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: leave_accrual_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.leave_accrual_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    policy_assignment_id uuid,
    accrual_date date NOT NULL,
    amount numeric(5,2) NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    leave_type_id uuid NOT NULL,
    employment_id uuid NOT NULL
);

--
-- Name: leave_balances; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: leave_policy_assignments; Type: TABLE; Schema: public; Owner: -
--

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
    CONSTRAINT leave_policy_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text, 'user'::text])))
);

--
-- Name: leave_policy_definitions; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: leave_requests; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: leave_types; Type: TABLE; Schema: public; Owner: -
--

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
    workspace_id uuid NOT NULL,
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
    workspace_id uuid NOT NULL,
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
-- Name: outbox_jobs; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: overtime_policy_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.overtime_policy_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    policy_id uuid NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT overtime_policy_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text, 'user'::text])))
);

--
-- Name: overtime_policy_definitions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.overtime_policy_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name text NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

--
-- Name: overtime_request_employees; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: overtime_requests; Type: TABLE; Schema: public; Owner: -
--

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
    attachment_file_ids uuid[],
    CONSTRAINT overtime_requests_planned_minutes_check CHECK ((planned_minutes > 0))
);

--
-- Name: overtime_rules; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: payroll_incentive_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.payroll_incentive_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    policy_id uuid NOT NULL,
    assignee_type text NOT NULL,
    assignee_id uuid,
    effective_from date NOT NULL,
    effective_to date,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT payroll_incentive_assignments_assignee_type_check CHECK ((assignee_type = ANY (ARRAY['workspace'::text, 'department'::text, 'employee'::text, 'user'::text])))
);

--
-- Name: payroll_incentive_policies; Type: TABLE; Schema: public; Owner: -
--

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
-- Name: payroll_rule_assignments; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: payroll_rule_definitions; Type: TABLE; Schema: public; Owner: -
--

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
    std_days_source character varying(50)
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
    workspace_id uuid NOT NULL,
    employment_id uuid NOT NULL
);

--
-- Name: policy_templates; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: raw_attendance_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.raw_attendance_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
-- Name: recruitment_sources; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: salary_base_groups; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: salary_component_base_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.salary_component_base_members (
    component_definition_id uuid NOT NULL,
    base_group_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: salary_component_definitions; Type: TABLE; Schema: public; Owner: -
--

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
-- Name: TABLE shift_audit_log; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.shift_audit_log IS 'Append-only tamper-proof audit trail for all shift changes. Populated by application code, not triggers.';

--
-- Name: COLUMN shift_audit_log.old_values; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_audit_log.old_values IS 'Full JSON snapshot of the superseded version including intervals. NULL on initial creation.';

--
-- Name: COLUMN shift_audit_log.new_values; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_audit_log.new_values IS 'Full JSON snapshot of the new version including intervals.';

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
    is_break_auto_deduct boolean DEFAULT true NOT NULL,
    break_is_overtime boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    early_punch_in_grace_period_minutes integer DEFAULT 0 CONSTRAINT shift_default_intervals_early_punch_in_grace_period_mi_not_null NOT NULL,
    allow_early_ot boolean DEFAULT false NOT NULL,
    max_early_ot_minutes integer,
    ot_aggregation_mode character varying(30) DEFAULT 'aggregate'::character varying NOT NULL,
    is_night_shift boolean DEFAULT false NOT NULL,
    version_id uuid NOT NULL,
    CONSTRAINT shift_default_intervals_sort_order_check CHECK ((sort_order >= 0))
);

--
-- Name: TABLE shift_default_intervals; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.shift_default_intervals IS 'Optional time blocks within a shift template. When empty, the shift start_time/end_time is used as a single virtual interval.';

--
-- Name: COLUMN shift_default_intervals.early_punch_in_grace_period_minutes; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_default_intervals.early_punch_in_grace_period_minutes IS 'Grace period (minutes) before scheduled start without late penalty';

--
-- Name: COLUMN shift_default_intervals.allow_early_ot; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_default_intervals.allow_early_ot IS 'Whether work before scheduled start counts as overtime';

--
-- Name: COLUMN shift_default_intervals.max_early_ot_minutes; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_default_intervals.max_early_ot_minutes IS 'Maximum minutes of early overtime allowed';

--
-- Name: COLUMN shift_default_intervals.ot_aggregation_mode; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_default_intervals.ot_aggregation_mode IS 'aggregate=sum all segments then apply OT threshold, per_segment=OT calculated independently per segment';

--
-- Name: COLUMN shift_default_intervals.is_night_shift; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_default_intervals.is_night_shift IS 'When true, interval hours crossing midnight are attributed to the start day';

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
    template_id uuid
);

--
-- Name: shift_swap_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.shift_swap_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
    target_template_id uuid
);

--
-- Name: shift_templates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.shift_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    name character varying(100) NOT NULL,
    current_version_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: TABLE shift_templates; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.shift_templates IS 'Stable logical identity for a shift concept. Never changes across revisions or corrections.';

--
-- Name: shift_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.shift_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    template_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
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
-- Name: TABLE shift_versions; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.shift_versions IS 'Immutable SCD Type-2 rows. One row per change. Never UPDATE, only INSERT.';

--
-- Name: COLUMN shift_versions.effective_to; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_versions.effective_to IS 'NULL for current version. Set to (new.effective_from - 1 day) when superseded by a revision, or (new.effective_from) when superseded by a correction.';

--
-- Name: COLUMN shift_versions.change_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_versions.change_type IS 'create=initial version, revise=scheduled change with new effective_from, correct=error fix keeping same effective_from';

--
-- Name: COLUMN shift_versions.superseded_by; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.shift_versions.superseded_by IS 'Points to the version that replaces this one. NULL if this is the current version.';

--
-- Name: shifts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.shifts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
-- Name: signature_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.signature_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    document_id uuid NOT NULL,
    signer_id uuid NOT NULL,
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    signed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
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
    workspace_id uuid NOT NULL,
    employment_id uuid NOT NULL,
    segment_id uuid,
    template_id uuid
);

--
-- Name: COLUMN time_entries.segment_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.time_entries.segment_id IS 'Links the actual punch session to its scheduled work segment. NULL for backward compat (entries before Work Segments migration).';

--
-- Name: work_locations; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: work_segments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_segments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
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
    CONSTRAINT work_segments_compensation_rate_check CHECK (((compensation_rate >= 0.00) AND (compensation_rate <= 1.00))),
    CONSTRAINT work_segments_gap_rate_check CHECK (((gap_compensation_rate_before >= 0.00) AND (gap_compensation_rate_before <= 1.00))),
    CONSTRAINT work_segments_sort_order_check CHECK ((sort_order >= 0)),
    CONSTRAINT work_segments_source_check CHECK (((source)::text = ANY (ARRAY['roster'::text, 'pattern'::text, 'rule'::text, 'fixed'::text, 'manual'::text, 'auto'::text])))
);

--
-- Name: TABLE work_segments; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.work_segments IS 'Work segments are the atomic unit of scheduled work. One work day can have N segments. Replaces shift_rosters as the scheduling entity.';

--
-- Name: COLUMN work_segments.segment_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.work_segments.segment_type IS 'work=regular shift, standby=paid waiting, on_call=unpaid availability, training=education, travel=business trip';

--
-- Name: COLUMN work_segments.compensation_rate; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.work_segments.compensation_rate IS '1.00 = 100% pay for this segment. For standby typically 0.30-0.50';

--
-- Name: COLUMN work_segments.gap_compensation_before; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.work_segments.gap_compensation_before IS 'How the gap BEFORE this segment is treated: unpaid (no pay), standby (partial pay), break (auto-deduct like a long break)';

--
-- Name: COLUMN work_segments.source; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.work_segments.source IS 'roster | pattern | rule | fixed | manual | auto';

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
-- Name: COLUMN working_conditions.condition_type_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.working_conditions.condition_type_id IS 'FK to condition_type_definitions. Nullable for backward compatibility with existing string-based type_ field.';

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
-- Name: COLUMN working_pattern_details.schedule_day; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.working_pattern_details.schedule_day IS 'For weekly patterns: day_of_week (0=Sun..6=Sat). For rotation patterns: cycle_day (0..N-1)';

--
-- Name: working_patterns; Type: TABLE; Schema: public; Owner: -
--

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

--
-- Name: workspace_compliance_rules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.workspace_compliance_rules (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    min_rest_period_hours integer DEFAULT 12 NOT NULL,
    max_weekly_hours integer DEFAULT 48 NOT NULL,
    max_monthly_ot_hours integer DEFAULT 40 NOT NULL,
    allow_manager_override boolean DEFAULT true NOT NULL,
    is_enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: TABLE workspace_compliance_rules; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.workspace_compliance_rules IS 'Per-workspace compliance rule thresholds for shift conflict detection and rest-period validation.';

--
-- Name: COLUMN workspace_compliance_rules.id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.workspace_compliance_rules.id IS 'Primary key.';

--
-- Name: COLUMN workspace_compliance_rules.workspace_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.workspace_compliance_rules.workspace_id IS 'Reference to the workspace these rules apply to. One row per workspace enforced by UNIQUE constraint.';

--
-- Name: COLUMN workspace_compliance_rules.min_rest_period_hours; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.workspace_compliance_rules.min_rest_period_hours IS 'Minimum rest hours required between two consecutive shifts. Default 12 per Vietnamese Labor Code.';

--
-- Name: COLUMN workspace_compliance_rules.max_weekly_hours; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.workspace_compliance_rules.max_weekly_hours IS 'Maximum total working hours allowed per calendar week (Mon-Sun). Default 48.';

--
-- Name: COLUMN workspace_compliance_rules.max_monthly_ot_hours; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.workspace_compliance_rules.max_monthly_ot_hours IS 'Maximum overtime hours allowed per calendar month. Default 40 per Vietnamese Labor Code.';

--
-- Name: COLUMN workspace_compliance_rules.allow_manager_override; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.workspace_compliance_rules.allow_manager_override IS 'When true, managers can override soft compliance warnings by providing a reason.';

--
-- Name: COLUMN workspace_compliance_rules.is_enabled; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.workspace_compliance_rules.is_enabled IS 'When false, all compliance checks are skipped for this workspace.';

--
-- Name: workspace_config_entries; Type: TABLE; Schema: public; Owner: -
--

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
    CONSTRAINT workspace_config_entries_scope_type_check CHECK (((scope_type)::text = ANY (ARRAY[('workspace'::character varying)::text, ('employer'::character varying)::text, ('department'::character varying)::text, ('employee'::character varying)::text, ('employment'::character varying)::text])))
);

--
-- Name: workspace_files; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.workspace_files (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    file_name character varying(255) NOT NULL,
    storage_key character varying(500) NOT NULL,
    file_size bigint NOT NULL,
    mime_type character varying(100) NOT NULL,
    content_hash character varying(64),
    category character varying(50) NOT NULL,
    folder_id uuid,
    owner_type character varying(50) NOT NULL,
    owner_id uuid NOT NULL,
    uploaded_by uuid,
    is_active boolean DEFAULT true NOT NULL,
    expires_at timestamp with time zone,
    scan_status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    scanned_at timestamp with time zone,
    version integer DEFAULT 1 NOT NULL,
    parent_file_id uuid,
    is_latest boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: attendance_staging_logs_default; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_default DEFAULT;

--
-- Name: attendance_staging_logs_y2026m01; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m01 FOR VALUES FROM ('2026-01-01 00:00:00+00') TO ('2026-02-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m02; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m02 FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m03; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m03 FOR VALUES FROM ('2026-03-01 00:00:00+00') TO ('2026-04-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m04; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m04 FOR VALUES FROM ('2026-04-01 00:00:00+00') TO ('2026-05-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m05; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m05 FOR VALUES FROM ('2026-05-01 00:00:00+00') TO ('2026-06-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m06; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m06 FOR VALUES FROM ('2026-06-01 00:00:00+00') TO ('2026-07-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m07; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m07 FOR VALUES FROM ('2026-07-01 00:00:00+00') TO ('2026-08-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m08; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m08 FOR VALUES FROM ('2026-08-01 00:00:00+00') TO ('2026-09-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m09; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m09 FOR VALUES FROM ('2026-09-01 00:00:00+00') TO ('2026-10-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m10; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m10 FOR VALUES FROM ('2026-10-01 00:00:00+00') TO ('2026-11-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m11; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m11 FOR VALUES FROM ('2026-11-01 00:00:00+00') TO ('2026-12-01 00:00:00+00');

--
-- Name: attendance_staging_logs_y2026m12; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_staging_logs ATTACH PARTITION public.attendance_staging_logs_y2026m12 FOR VALUES FROM ('2026-12-01 00:00:00+00') TO ('2027-01-01 00:00:00+00');

--
-- Name: audit_logs_default; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_default DEFAULT;

--
-- Name: audit_logs_y2026m01; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m01 FOR VALUES FROM ('2026-01-01 00:00:00+00') TO ('2026-02-01 00:00:00+00');

--
-- Name: audit_logs_y2026m02; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m02 FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');

--
-- Name: audit_logs_y2026m03; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m03 FOR VALUES FROM ('2026-03-01 00:00:00+00') TO ('2026-04-01 00:00:00+00');

--
-- Name: audit_logs_y2026m04; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m04 FOR VALUES FROM ('2026-04-01 00:00:00+00') TO ('2026-05-01 00:00:00+00');

--
-- Name: audit_logs_y2026m05; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m05 FOR VALUES FROM ('2026-05-01 00:00:00+00') TO ('2026-06-01 00:00:00+00');

--
-- Name: audit_logs_y2026m06; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m06 FOR VALUES FROM ('2026-06-01 00:00:00+00') TO ('2026-07-01 00:00:00+00');

--
-- Name: audit_logs_y2026m07; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m07 FOR VALUES FROM ('2026-07-01 00:00:00+00') TO ('2026-08-01 00:00:00+00');

--
-- Name: audit_logs_y2026m08; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m08 FOR VALUES FROM ('2026-08-01 00:00:00+00') TO ('2026-09-01 00:00:00+00');

--
-- Name: audit_logs_y2026m09; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m09 FOR VALUES FROM ('2026-09-01 00:00:00+00') TO ('2026-10-01 00:00:00+00');

--
-- Name: audit_logs_y2026m10; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m10 FOR VALUES FROM ('2026-10-01 00:00:00+00') TO ('2026-11-01 00:00:00+00');

--
-- Name: audit_logs_y2026m11; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m11 FOR VALUES FROM ('2026-11-01 00:00:00+00') TO ('2026-12-01 00:00:00+00');

--
-- Name: audit_logs_y2026m12; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_y2026m12 FOR VALUES FROM ('2026-12-01 00:00:00+00') TO ('2027-01-01 00:00:00+00');

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
-- Name: attendance_policy_versions attendance_policy_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_policy_versions
    ADD CONSTRAINT attendance_policy_versions_pkey PRIMARY KEY (id);

--
-- Name: attendance_source_pull_states attendance_source_pull_states_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_source_pull_states
    ADD CONSTRAINT attendance_source_pull_states_pkey PRIMARY KEY (id);

--
-- Name: attendance_source_pull_states attendance_source_pull_states_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_source_pull_states
    ADD CONSTRAINT attendance_source_pull_states_unique UNIQUE (workspace_id, source_id);

--
-- Name: attendance_sources attendance_sources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_sources
    ADD CONSTRAINT attendance_sources_pkey PRIMARY KEY (id);

--
-- Name: attendance_sources attendance_sources_workspace_provider_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_sources
    ADD CONSTRAINT attendance_sources_workspace_provider_unique UNIQUE (workspace_id, provider_source_id);

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
-- Name: condition_type_definitions condition_type_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.condition_type_definitions
    ADD CONSTRAINT condition_type_definitions_pkey PRIMARY KEY (id);

--
-- Name: condition_type_definitions condition_type_definitions_workspace_id_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.condition_type_definitions
    ADD CONSTRAINT condition_type_definitions_workspace_id_code_key UNIQUE (workspace_id, code);

--
-- Name: custom_field_definitions custom_field_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.custom_field_definitions
    ADD CONSTRAINT custom_field_definitions_pkey PRIMARY KEY (id);

--
-- Name: custom_field_values custom_field_values_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.custom_field_values
    ADD CONSTRAINT custom_field_values_pkey PRIMARY KEY (id);

--
-- Name: dashboard_preferences dashboard_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dashboard_preferences
    ADD CONSTRAINT dashboard_preferences_pkey PRIMARY KEY (id);

--
-- Name: dashboard_preferences dashboard_preferences_user_id_workspace_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dashboard_preferences
    ADD CONSTRAINT dashboard_preferences_user_id_workspace_id_key UNIQUE (user_id, workspace_id);

--
-- Name: departments departments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_pkey PRIMARY KEY (id);

--
-- Name: document_folders document_folders_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_folders
    ADD CONSTRAINT document_folders_pkey PRIMARY KEY (id);

--
-- Name: econtract_envelopes econtract_envelopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.econtract_envelopes
    ADD CONSTRAINT econtract_envelopes_pkey PRIMARY KEY (id);

--
-- Name: employee_assignments employee_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_pkey PRIMARY KEY (id);

--
-- Name: employee_contracts employee_contracts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_pkey PRIMARY KEY (id);

--
-- Name: employee_documents employee_documents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_pkey PRIMARY KEY (id);

--
-- Name: employee_emergency_contacts employee_emergency_contacts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_emergency_contacts
    ADD CONSTRAINT employee_emergency_contacts_pkey PRIMARY KEY (id);

--
-- Name: employee_phases employee_phases_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_pkey PRIMARY KEY (id);

--
-- Name: employee_profiles employee_profiles_employee_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_employee_id_key UNIQUE (employee_id);

--
-- Name: employee_profiles employee_profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_pkey PRIMARY KEY (id);

--
-- Name: employee_salary_components employee_salary_components_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_salary_components
    ADD CONSTRAINT employee_salary_components_pkey PRIMARY KEY (id);

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
-- Name: employees employees_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_pkey PRIMARY KEY (id);

--
-- Name: employer_econtract_configs employer_econtract_configs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employer_econtract_configs
    ADD CONSTRAINT employer_econtract_configs_pkey PRIMARY KEY (id);

--
-- Name: employer_econtract_routing_rules employer_econtract_routing_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employer_econtract_routing_rules
    ADD CONSTRAINT employer_econtract_routing_rules_pkey PRIMARY KEY (id);

--
-- Name: employers employers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employers
    ADD CONSTRAINT employers_pkey PRIMARY KEY (id);

--
-- Name: employment_phase_definitions employment_phase_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_pkey PRIMARY KEY (id);

--
-- Name: employment_phase_definitions employment_phase_definitions_workspace_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employment_phase_definitions
    ADD CONSTRAINT employment_phase_definitions_workspace_id_name_key UNIQUE (workspace_id, name);

--
-- Name: employments employments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_pkey PRIMARY KEY (id);

--
-- Name: external_attendance_events external_attendance_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.external_attendance_events
    ADD CONSTRAINT external_attendance_events_pkey PRIMARY KEY (id);

--
-- Name: external_attendance_events external_attendance_events_source_identity_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.external_attendance_events
    ADD CONSTRAINT external_attendance_events_source_identity_unique UNIQUE (workspace_id, source_id, external_event_id);

--
-- Name: feature_flags feature_flags_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.feature_flags
    ADD CONSTRAINT feature_flags_pkey PRIMARY KEY (id);

--
-- Name: feature_flags feature_flags_workspace_id_flag_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.feature_flags
    ADD CONSTRAINT feature_flags_workspace_id_flag_key_key UNIQUE (workspace_id, flag_key);

--
-- Name: file_acknowledgements file_acknowledgements_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_acknowledgements
    ADD CONSTRAINT file_acknowledgements_pkey PRIMARY KEY (id);

--
-- Name: holidays holidays_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.holidays
    ADD CONSTRAINT holidays_pkey PRIMARY KEY (id);

--
-- Name: holidays holidays_workspace_id_date_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.holidays
    ADD CONSTRAINT holidays_workspace_id_date_key UNIQUE (workspace_id, date);

--
-- Name: leave_accrual_logs leave_accrual_logs_employment_leave_type_id_date_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_employment_leave_type_id_date_key UNIQUE (employment_id, leave_type_id, accrual_date);

--
-- Name: leave_accrual_logs leave_accrual_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_pkey PRIMARY KEY (id);

--
-- Name: leave_balances leave_balances_employment_id_leave_type_id_year_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_employment_id_leave_type_id_year_key UNIQUE (employment_id, leave_type_id, year);

--
-- Name: leave_balances leave_balances_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_pkey PRIMARY KEY (id);

--
-- Name: leave_policy_assignments leave_policy_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_pkey PRIMARY KEY (id);

--
-- Name: leave_policy_assignments leave_policy_assignments_workspace_id_assignee_type_coales_excl; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_workspace_id_assignee_type_coales_excl EXCLUDE USING gist (workspace_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&) WHERE ((deleted_at IS NULL));

--
-- Name: leave_policy_definitions leave_policy_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_policy_definitions
    ADD CONSTRAINT leave_policy_definitions_pkey PRIMARY KEY (id);

--
-- Name: leave_requests leave_requests_no_overlap_active; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_no_overlap_active EXCLUDE USING gist (employment_id WITH =, daterange(start_date, end_date, '[]'::text) WITH &&) WHERE ((status = ANY (ARRAY['approved'::public.leave_status_type, 'pre_approved'::public.leave_status_type])));

--
-- Name: leave_requests leave_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_pkey PRIMARY KEY (id);

--
-- Name: leave_types leave_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_types
    ADD CONSTRAINT leave_types_pkey PRIMARY KEY (id);

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
-- Name: attendance_policy_assignments no_overlapping_policy_assignments; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_policy_assignments
    ADD CONSTRAINT no_overlapping_policy_assignments EXCLUDE USING gist (target_id WITH =, target_type WITH =, tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamp with time zone)) WITH &&);

--
-- Name: attendance_policy_versions no_overlapping_policy_versions; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_policy_versions
    ADD CONSTRAINT no_overlapping_policy_versions EXCLUDE USING gist (policy_id WITH =, tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamp with time zone)) WITH &&);

--
-- Name: behalf_assignments ot_claim_behalf_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT ot_claim_behalf_assignments_pkey PRIMARY KEY (id);

--
-- Name: outbox_jobs outbox_jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.outbox_jobs
    ADD CONSTRAINT outbox_jobs_pkey PRIMARY KEY (id);

--
-- Name: overtime_policy_assignments overtime_policy_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_pkey PRIMARY KEY (id);

--
-- Name: overtime_policy_assignments overtime_policy_assignments_workspace_id_assignee_type_coa_excl; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_workspace_id_assignee_type_coa_excl EXCLUDE USING gist (workspace_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&);

--
-- Name: overtime_policy_definitions overtime_policy_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_policy_definitions
    ADD CONSTRAINT overtime_policy_definitions_pkey PRIMARY KEY (id);

--
-- Name: overtime_request_employees overtime_request_employees_overtime_request_id_employment_id_ke; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_overtime_request_id_employment_id_ke UNIQUE (overtime_request_id, employment_id);

--
-- Name: overtime_request_employees overtime_request_employees_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_pkey PRIMARY KEY (id);

--
-- Name: overtime_requests overtime_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_pkey PRIMARY KEY (id);

--
-- Name: overtime_rules overtime_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_rules
    ADD CONSTRAINT overtime_rules_pkey PRIMARY KEY (id);

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
-- Name: payroll_rule_assignments payroll_rule_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_pkey PRIMARY KEY (id);

--
-- Name: payroll_rule_assignments payroll_rule_assignments_workspace_id_rule_definition_id_a_excl; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payroll_rule_assignments
    ADD CONSTRAINT payroll_rule_assignments_workspace_id_rule_definition_id_a_excl EXCLUDE USING gist (workspace_id WITH =, rule_definition_id WITH =, assignee_type WITH =, COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, daterange(effective_from, effective_to, '[)'::text) WITH &&);

--
-- Name: payroll_rule_definitions payroll_rule_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payroll_rule_definitions
    ADD CONSTRAINT payroll_rule_definitions_pkey PRIMARY KEY (id);

--
-- Name: payroll_rule_definitions payroll_rule_definitions_workspace_id_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payroll_rule_definitions
    ADD CONSTRAINT payroll_rule_definitions_workspace_id_code_key UNIQUE (workspace_id, code);

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
-- Name: policy_templates policy_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.policy_templates
    ADD CONSTRAINT policy_templates_pkey PRIMARY KEY (id);

--
-- Name: raw_attendance_logs raw_attendance_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT raw_attendance_logs_pkey PRIMARY KEY (id);

--
-- Name: recruitment_sources recruitment_sources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recruitment_sources
    ADD CONSTRAINT recruitment_sources_pkey PRIMARY KEY (id);

--
-- Name: salary_base_groups salary_base_groups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salary_base_groups
    ADD CONSTRAINT salary_base_groups_pkey PRIMARY KEY (id);

--
-- Name: salary_component_base_members salary_component_base_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salary_component_base_members
    ADD CONSTRAINT salary_component_base_members_pkey PRIMARY KEY (component_definition_id, base_group_id);

--
-- Name: salary_component_definitions salary_component_definitions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_pkey PRIMARY KEY (id);

--
-- Name: salary_component_definitions salary_component_definitions_workspace_id_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_workspace_id_code_key UNIQUE (workspace_id, code);

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
-- Name: shift_templates shift_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.shift_templates
    ADD CONSTRAINT shift_templates_pkey PRIMARY KEY (id);

--
-- Name: shift_templates shift_templates_workspace_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.shift_templates
    ADD CONSTRAINT shift_templates_workspace_id_name_key UNIQUE (workspace_id, name);

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
-- Name: signature_requests signature_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.signature_requests
    ADD CONSTRAINT signature_requests_pkey PRIMARY KEY (id);

--
-- Name: time_entries time_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.time_entries
    ADD CONSTRAINT time_entries_pkey PRIMARY KEY (id);

--
-- Name: behalf_assignments uniq_behalf_assignment; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT uniq_behalf_assignment UNIQUE (workspace_id, assignee_user_id, target_employee_id, target_department_id, assignment_type);

--
-- Name: raw_attendance_logs unique_raw_log; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT unique_raw_log UNIQUE (workspace_id, employment_id, punch_time, punch, status);

--
-- Name: custom_field_values uq_definition_entity; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.custom_field_values
    ADD CONSTRAINT uq_definition_entity UNIQUE (definition_id, entity_id);

--
-- Name: employer_econtract_configs uq_employer_provider; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employer_econtract_configs
    ADD CONSTRAINT uq_employer_provider UNIQUE (employer_id, provider_type);

--
-- Name: econtract_envelopes uq_envelope_provider; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.econtract_envelopes
    ADD CONSTRAINT uq_envelope_provider UNIQUE (provider_type, external_envelope_id);

--
-- Name: custom_field_definitions uq_workspace_entity_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.custom_field_definitions
    ADD CONSTRAINT uq_workspace_entity_key UNIQUE (workspace_id, entity_type, field_key);

--
-- Name: work_locations work_locations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_locations
    ADD CONSTRAINT work_locations_pkey PRIMARY KEY (id);

--
-- Name: work_segments work_segments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_pkey PRIMARY KEY (id);

--
-- Name: working_conditions working_conditions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_pkey PRIMARY KEY (id);

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
-- Name: workspace_compliance_rules workspace_compliance_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_compliance_rules
    ADD CONSTRAINT workspace_compliance_rules_pkey PRIMARY KEY (id);

--
-- Name: workspace_compliance_rules workspace_compliance_rules_workspace_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_compliance_rules
    ADD CONSTRAINT workspace_compliance_rules_workspace_id_key UNIQUE (workspace_id);

--
-- Name: workspace_config_entries workspace_config_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_config_entries
    ADD CONSTRAINT workspace_config_entries_pkey PRIMARY KEY (id);

--
-- Name: workspace_config_entries workspace_config_entries_workspace_id_scope_type_scope_id_c_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_config_entries
    ADD CONSTRAINT workspace_config_entries_workspace_id_scope_type_scope_id_c_key UNIQUE (workspace_id, scope_type, scope_id, config_key, effective_from);

--
-- Name: workspace_files workspace_files_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_files
    ADD CONSTRAINT workspace_files_pkey PRIMARY KEY (id);

--
-- Name: workspace_files workspace_files_storage_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_files
    ADD CONSTRAINT workspace_files_storage_key_key UNIQUE (storage_key);

--
-- Name: attendance_policies_workspace_default_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX attendance_policies_workspace_default_idx ON public.attendance_policies USING btree (workspace_id) WHERE (is_default = true);

--
-- Name: idx_staging_logs_fingerprint; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_staging_logs_fingerprint ON ONLY public.attendance_staging_logs USING btree (fingerprint);

--
-- Name: attendance_staging_logs_default_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_default_fingerprint_idx ON public.attendance_staging_logs_default USING btree (fingerprint);

--
-- Name: idx_staging_logs_workspace_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_staging_logs_workspace_status ON ONLY public.attendance_staging_logs USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_default_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_default_workspace_id_status_idx ON public.attendance_staging_logs_default USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m01_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m01_fingerprint_idx ON public.attendance_staging_logs_y2026m01 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m01_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m01_workspace_id_status_idx ON public.attendance_staging_logs_y2026m01 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m02_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m02_fingerprint_idx ON public.attendance_staging_logs_y2026m02 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m02_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m02_workspace_id_status_idx ON public.attendance_staging_logs_y2026m02 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m03_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m03_fingerprint_idx ON public.attendance_staging_logs_y2026m03 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m03_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m03_workspace_id_status_idx ON public.attendance_staging_logs_y2026m03 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m04_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m04_fingerprint_idx ON public.attendance_staging_logs_y2026m04 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m04_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m04_workspace_id_status_idx ON public.attendance_staging_logs_y2026m04 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m05_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m05_fingerprint_idx ON public.attendance_staging_logs_y2026m05 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m05_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m05_workspace_id_status_idx ON public.attendance_staging_logs_y2026m05 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m06_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m06_fingerprint_idx ON public.attendance_staging_logs_y2026m06 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m06_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m06_workspace_id_status_idx ON public.attendance_staging_logs_y2026m06 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m07_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m07_fingerprint_idx ON public.attendance_staging_logs_y2026m07 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m07_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m07_workspace_id_status_idx ON public.attendance_staging_logs_y2026m07 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m08_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m08_fingerprint_idx ON public.attendance_staging_logs_y2026m08 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m08_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m08_workspace_id_status_idx ON public.attendance_staging_logs_y2026m08 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m09_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m09_fingerprint_idx ON public.attendance_staging_logs_y2026m09 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m09_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m09_workspace_id_status_idx ON public.attendance_staging_logs_y2026m09 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m10_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m10_fingerprint_idx ON public.attendance_staging_logs_y2026m10 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m10_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m10_workspace_id_status_idx ON public.attendance_staging_logs_y2026m10 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m11_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m11_fingerprint_idx ON public.attendance_staging_logs_y2026m11 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m11_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m11_workspace_id_status_idx ON public.attendance_staging_logs_y2026m11 USING btree (workspace_id, status);

--
-- Name: attendance_staging_logs_y2026m12_fingerprint_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m12_fingerprint_idx ON public.attendance_staging_logs_y2026m12 USING btree (fingerprint);

--
-- Name: attendance_staging_logs_y2026m12_workspace_id_status_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX attendance_staging_logs_y2026m12_workspace_id_status_idx ON public.attendance_staging_logs_y2026m12 USING btree (workspace_id, status);

--
-- Name: idx_audit_logs_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_action ON ONLY public.audit_logs USING btree (action);

--
-- Name: audit_logs_default_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_default_action_idx ON public.audit_logs_default USING btree (action);

--
-- Name: idx_audit_logs_target_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_target_user ON ONLY public.audit_logs USING btree (target_user_id);

--
-- Name: audit_logs_default_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_default_target_user_id_idx ON public.audit_logs_default USING btree (target_user_id);

--
-- Name: idx_audit_logs_workspace_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_workspace_created ON ONLY public.audit_logs USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_default_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_default_workspace_id_created_at_idx ON public.audit_logs_default USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m01_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m01_action_idx ON public.audit_logs_y2026m01 USING btree (action);

--
-- Name: audit_logs_y2026m01_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m01_target_user_id_idx ON public.audit_logs_y2026m01 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m01_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m01_workspace_id_created_at_idx ON public.audit_logs_y2026m01 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m02_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m02_action_idx ON public.audit_logs_y2026m02 USING btree (action);

--
-- Name: audit_logs_y2026m02_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m02_target_user_id_idx ON public.audit_logs_y2026m02 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m02_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m02_workspace_id_created_at_idx ON public.audit_logs_y2026m02 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m03_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m03_action_idx ON public.audit_logs_y2026m03 USING btree (action);

--
-- Name: audit_logs_y2026m03_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m03_target_user_id_idx ON public.audit_logs_y2026m03 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m03_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m03_workspace_id_created_at_idx ON public.audit_logs_y2026m03 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m04_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m04_action_idx ON public.audit_logs_y2026m04 USING btree (action);

--
-- Name: audit_logs_y2026m04_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m04_target_user_id_idx ON public.audit_logs_y2026m04 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m04_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m04_workspace_id_created_at_idx ON public.audit_logs_y2026m04 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m05_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m05_action_idx ON public.audit_logs_y2026m05 USING btree (action);

--
-- Name: audit_logs_y2026m05_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m05_target_user_id_idx ON public.audit_logs_y2026m05 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m05_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m05_workspace_id_created_at_idx ON public.audit_logs_y2026m05 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m06_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m06_action_idx ON public.audit_logs_y2026m06 USING btree (action);

--
-- Name: audit_logs_y2026m06_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m06_target_user_id_idx ON public.audit_logs_y2026m06 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m06_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m06_workspace_id_created_at_idx ON public.audit_logs_y2026m06 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m07_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m07_action_idx ON public.audit_logs_y2026m07 USING btree (action);

--
-- Name: audit_logs_y2026m07_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m07_target_user_id_idx ON public.audit_logs_y2026m07 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m07_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m07_workspace_id_created_at_idx ON public.audit_logs_y2026m07 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m08_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m08_action_idx ON public.audit_logs_y2026m08 USING btree (action);

--
-- Name: audit_logs_y2026m08_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m08_target_user_id_idx ON public.audit_logs_y2026m08 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m08_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m08_workspace_id_created_at_idx ON public.audit_logs_y2026m08 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m09_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m09_action_idx ON public.audit_logs_y2026m09 USING btree (action);

--
-- Name: audit_logs_y2026m09_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m09_target_user_id_idx ON public.audit_logs_y2026m09 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m09_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m09_workspace_id_created_at_idx ON public.audit_logs_y2026m09 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m10_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m10_action_idx ON public.audit_logs_y2026m10 USING btree (action);

--
-- Name: audit_logs_y2026m10_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m10_target_user_id_idx ON public.audit_logs_y2026m10 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m10_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m10_workspace_id_created_at_idx ON public.audit_logs_y2026m10 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m11_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m11_action_idx ON public.audit_logs_y2026m11 USING btree (action);

--
-- Name: audit_logs_y2026m11_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m11_target_user_id_idx ON public.audit_logs_y2026m11 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m11_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m11_workspace_id_created_at_idx ON public.audit_logs_y2026m11 USING btree (workspace_id, created_at DESC);

--
-- Name: audit_logs_y2026m12_action_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m12_action_idx ON public.audit_logs_y2026m12 USING btree (action);

--
-- Name: audit_logs_y2026m12_target_user_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m12_target_user_id_idx ON public.audit_logs_y2026m12 USING btree (target_user_id);

--
-- Name: audit_logs_y2026m12_workspace_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX audit_logs_y2026m12_workspace_id_created_at_idx ON public.audit_logs_y2026m12 USING btree (workspace_id, created_at DESC);

--
-- Name: dashboard_preferences_user_workspace_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX dashboard_preferences_user_workspace_unique ON public.dashboard_preferences USING btree (user_id, workspace_id);

--
-- Name: idx_aal_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_aal_entity ON public.approval_action_log USING btree (entity_type, entity_id);

--
-- Name: idx_aal_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_aal_workspace ON public.approval_action_log USING btree (workspace_id, acted_at);

--
-- Name: idx_assignment_rules_pattern_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_assignment_rules_pattern_id ON public.assignment_rules USING btree (pattern_id);

--
-- Name: idx_assignment_rules_priority; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_assignment_rules_priority ON public.assignment_rules USING btree (priority DESC);

--
-- Name: idx_assignment_rules_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_assignment_rules_workspace_id ON public.assignment_rules USING btree (workspace_id);

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
-- Name: idx_attendance_import_jobs_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attendance_import_jobs_workspace_id ON public.attendance_import_jobs USING btree (workspace_id);

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
-- Name: idx_attendance_sources_workspace_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attendance_sources_workspace_enabled ON public.attendance_sources USING btree (workspace_id, is_enabled);

--
-- Name: idx_aw_active_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_aw_active_unique ON public.approval_workflows USING btree (workspace_id, entity_type) WHERE (is_active = true);

--
-- Name: idx_aw_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_aw_workspace ON public.approval_workflows USING btree (workspace_id);

--
-- Name: idx_awi_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_awi_entity ON public.approval_workflow_instances USING btree (entity_type, entity_id);

--
-- Name: idx_awi_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_awi_workspace ON public.approval_workflow_instances USING btree (workspace_id);

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

CREATE INDEX idx_behalf_assignments_assignee ON public.behalf_assignments USING btree (workspace_id, assignee_user_id, assignment_type);

--
-- Name: idx_behalf_assignments_employer; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_behalf_assignments_employer ON public.behalf_assignments USING btree (employer_id);

--
-- Name: idx_cfd_filterable; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cfd_filterable ON public.custom_field_definitions USING btree (workspace_id, entity_type, is_filterable) WHERE ((deleted_at IS NULL) AND (is_filterable = true));

--
-- Name: idx_cfd_workspace_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_cfd_workspace_entity ON public.custom_field_definitions USING btree (workspace_id, entity_type) WHERE (deleted_at IS NULL);

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
-- Name: idx_daily_summaries_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_daily_summaries_status ON public.attendance_daily_summaries USING btree (ot_status);

--
-- Name: idx_daily_summaries_workspace_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_daily_summaries_workspace_date ON public.attendance_daily_summaries USING btree (workspace_id, date);

--
-- Name: idx_dashboard_prefs_user_ws; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dashboard_prefs_user_ws ON public.dashboard_preferences USING btree (user_id, workspace_id);

--
-- Name: idx_departments_manager_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_departments_manager_id ON public.departments USING btree (manager_id);

--
-- Name: idx_departments_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_departments_workspace_id ON public.departments USING btree (workspace_id);

--
-- Name: idx_document_folders_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_document_folders_parent ON public.document_folders USING btree (parent_folder_id);

--
-- Name: idx_document_folders_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_document_folders_workspace ON public.document_folders USING btree (workspace_id);

--
-- Name: idx_econtract_envelopes_contract; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_econtract_envelopes_contract ON public.econtract_envelopes USING btree (contract_id);

--
-- Name: idx_econtract_envelopes_external_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_econtract_envelopes_external_id ON public.econtract_envelopes USING btree (external_envelope_id);

--
-- Name: idx_econtract_routing_rules; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_econtract_routing_rules ON public.employer_econtract_routing_rules USING btree (employer_id, signer_country);

--
-- Name: idx_employee_assignments_employment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_assignments_employment_id ON public.employee_assignments USING btree (employment_id);

--
-- Name: idx_employee_contracts_dates; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_contracts_dates ON public.employee_contracts USING btree (start_date, end_date);

--
-- Name: idx_employee_contracts_employment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_contracts_employment_id ON public.employee_contracts USING btree (employment_id);

--
-- Name: idx_employee_contracts_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_contracts_status ON public.employee_contracts USING btree (status);

--
-- Name: idx_employee_contracts_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_contracts_workspace_id ON public.employee_contracts USING btree (workspace_id);

--
-- Name: idx_employee_documents_employee_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_documents_employee_id ON public.employee_documents USING btree (employee_id);

--
-- Name: idx_employee_documents_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_documents_expires_at ON public.employee_documents USING btree (expires_at) WHERE ((expires_at IS NOT NULL) AND (is_active = true));

--
-- Name: idx_employee_documents_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_documents_type ON public.employee_documents USING btree (document_type);

--
-- Name: idx_employee_documents_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_documents_workspace_id ON public.employee_documents USING btree (workspace_id);

--
-- Name: idx_employee_emergency_contacts_employee_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_emergency_contacts_employee_id ON public.employee_emergency_contacts USING btree (employee_id);

--
-- Name: idx_employee_emergency_contacts_primary; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_employee_emergency_contacts_primary ON public.employee_emergency_contacts USING btree (employee_id) WHERE (is_primary = true);

--
-- Name: idx_employee_phases_active_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_phases_active_lookup ON public.employee_phases USING btree (employment_id, status, start_date, end_date) WHERE (status = 'active'::text);

--
-- Name: idx_employee_phases_definition; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_phases_definition ON public.employee_phases USING btree (phase_definition_id);

--
-- Name: idx_employee_phases_employment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_phases_employment ON public.employee_phases USING btree (employment_id);

--
-- Name: idx_employee_phases_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_phases_status ON public.employee_phases USING btree (workspace_id, status);

--
-- Name: idx_employee_phases_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_phases_workspace ON public.employee_phases USING btree (workspace_id);

--
-- Name: idx_employee_profiles_employee_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_profiles_employee_id ON public.employee_profiles USING btree (employee_id);

--
-- Name: idx_employee_salary_comp_def_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_salary_comp_def_id ON public.employee_salary_components USING btree (component_definition_id);

--
-- Name: idx_employee_salary_components_effective_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_salary_components_effective_date ON public.employee_salary_components USING btree (effective_date);

--
-- Name: idx_employee_salary_components_employment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_salary_components_employment_id ON public.employee_salary_components USING btree (employment_id);

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
-- Name: idx_employee_working_patterns_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employee_working_patterns_workspace ON public.employee_working_patterns USING btree (workspace_id);

--
-- Name: idx_employees_id_number_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employees_id_number_search ON public.employees USING gin (employee_id_number public.gin_trgm_ops);

--
-- Name: idx_employees_name_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employees_name_search ON public.employees USING gin (full_name public.gin_trgm_ops);

--
-- Name: idx_employees_recruitment_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employees_recruitment_source ON public.employees USING btree (recruitment_source_id);

--
-- Name: idx_employees_tags; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employees_tags ON public.employees USING gin (tags);

--
-- Name: idx_employees_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employees_user_id ON public.employees USING btree (user_id);

--
-- Name: idx_employees_workspace_employee_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_employees_workspace_employee_id ON public.employees USING btree (workspace_id, employee_id_number) WHERE (deleted_at IS NULL);

--
-- Name: idx_employees_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employees_workspace_id ON public.employees USING btree (workspace_id);

--
-- Name: idx_employees_workspace_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_employees_workspace_user_id ON public.employees USING btree (workspace_id, user_id) WHERE (user_id IS NOT NULL);

--
-- Name: idx_employer_econtract_configs_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employer_econtract_configs_lookup ON public.employer_econtract_configs USING btree (employer_id, workspace_id);

--
-- Name: idx_employers_parent_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employers_parent_id ON public.employers USING btree (parent_id);

--
-- Name: idx_employers_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employers_workspace_id ON public.employers USING btree (workspace_id);

--
-- Name: idx_employment_phase_defs_rule_config; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employment_phase_defs_rule_config ON public.employment_phase_definitions USING gin (rule_config);

--
-- Name: idx_employment_phase_defs_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employment_phase_defs_type ON public.employment_phase_definitions USING btree (workspace_id, phase_type);

--
-- Name: idx_employment_phase_defs_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employment_phase_defs_workspace ON public.employment_phase_definitions USING btree (workspace_id);

--
-- Name: idx_employments_date_range; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employments_date_range ON public.employments USING btree (start_date, end_date);

--
-- Name: idx_employments_employee_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employments_employee_id ON public.employments USING btree (employee_id);

--
-- Name: idx_employments_employer_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employments_employer_id ON public.employments USING btree (employer_id);

--
-- Name: idx_employments_query; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employments_query ON public.employments USING btree (workspace_id, employee_id, start_date, end_date) WHERE (deleted_at IS NULL);

--
-- Name: idx_employments_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_employments_workspace_id ON public.employments USING btree (workspace_id);

--
-- Name: idx_errors_fingerprint; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_errors_fingerprint ON public.attendance_log_errors USING btree (workspace_id, fingerprint) WHERE (fingerprint IS NOT NULL);

--
-- Name: idx_external_attendance_events_cleanup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_external_attendance_events_cleanup ON public.external_attendance_events USING btree (workspace_id, received_at);

--
-- Name: idx_external_attendance_events_source_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_external_attendance_events_source_time ON public.external_attendance_events USING btree (workspace_id, source_id, occurred_at);

--
-- Name: idx_feature_flags_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_feature_flags_lookup ON public.feature_flags USING btree (workspace_id, flag_key);

--
-- Name: idx_feature_flags_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_feature_flags_workspace ON public.feature_flags USING btree (workspace_id);

--
-- Name: idx_file_acknowledgements_document; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_file_acknowledgements_document ON public.file_acknowledgements USING btree (document_id);

--
-- Name: idx_file_acknowledgements_employee; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_file_acknowledgements_employee ON public.file_acknowledgements USING btree (employee_id);

--
-- Name: idx_file_acknowledgements_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_file_acknowledgements_workspace ON public.file_acknowledgements USING btree (workspace_id);

--
-- Name: idx_holidays_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_holidays_lookup ON public.holidays USING btree (workspace_id, date);

--
-- Name: idx_holidays_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_holidays_workspace_id ON public.holidays USING btree (workspace_id);

--
-- Name: idx_holidays_workspace_location_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_holidays_workspace_location_date ON public.holidays USING btree (workspace_id, location_id, date);

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
-- Name: idx_lc_employment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lc_employment ON public.lifecycle_checklists USING btree (workspace_id, employment_id);

--
-- Name: idx_lc_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lc_status ON public.lifecycle_checklists USING btree (workspace_id, status) WHERE ((status)::text = 'in_progress'::text);

--
-- Name: idx_lc_workspace_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lc_workspace_type ON public.lifecycle_checklists USING btree (workspace_id, checklist_type);

--
-- Name: idx_lci_assignee_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lci_assignee_status ON public.lifecycle_checklist_items USING btree (assignee_user_id, status) WHERE ((status)::text = 'pending'::text);

--
-- Name: idx_lci_checklist; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lci_checklist ON public.lifecycle_checklist_items USING btree (checklist_id);

--
-- Name: idx_lct_unique_default; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_lct_unique_default ON public.lifecycle_checklist_templates USING btree (workspace_id, checklist_type) WHERE ((is_default = true) AND (deleted_at IS NULL));

--
-- Name: idx_lct_workspace_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lct_workspace_type ON public.lifecycle_checklist_templates USING btree (workspace_id, checklist_type) WHERE (deleted_at IS NULL);

--
-- Name: idx_lcti_template; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lcti_template ON public.lifecycle_checklist_template_items USING btree (template_id);

--
-- Name: idx_leave_accrual_logs_leave_type_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_accrual_logs_leave_type_id ON public.leave_accrual_logs USING btree (leave_type_id);

--
-- Name: idx_leave_accrual_logs_policy_assignment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_accrual_logs_policy_assignment_id ON public.leave_accrual_logs USING btree (policy_assignment_id);

--
-- Name: idx_leave_accrual_logs_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_accrual_logs_workspace ON public.leave_accrual_logs USING btree (workspace_id);

--
-- Name: idx_leave_balances_employment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_balances_employment_id ON public.leave_balances USING btree (employment_id);

--
-- Name: idx_leave_balances_leave_type_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_balances_leave_type_id ON public.leave_balances USING btree (leave_type_id);

--
-- Name: idx_leave_policy_assignments_assignee; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_policy_assignments_assignee ON public.leave_policy_assignments USING btree (assignee_type, assignee_id);

--
-- Name: idx_leave_policy_assignments_policy_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_policy_assignments_policy_id ON public.leave_policy_assignments USING btree (policy_id);

--
-- Name: idx_leave_policy_assignments_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_policy_assignments_workspace ON public.leave_policy_assignments USING btree (workspace_id);

--
-- Name: idx_leave_policy_definitions_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_policy_definitions_workspace ON public.leave_policy_definitions USING btree (workspace_id);

--
-- Name: idx_leave_policy_definitions_ws_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_leave_policy_definitions_ws_name ON public.leave_policy_definitions USING btree (workspace_id, name);

--
-- Name: idx_leave_policy_defs_rule_config; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_policy_defs_rule_config ON public.leave_policy_definitions USING gin (rule_config);

--
-- Name: idx_leave_requests_approved_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_requests_approved_by ON public.leave_requests USING btree (approved_by_user_id);

--
-- Name: idx_leave_requests_date_range; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_requests_date_range ON public.leave_requests USING btree (start_date, end_date);

--
-- Name: idx_leave_requests_employment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_requests_employment_id ON public.leave_requests USING btree (employment_id);

--
-- Name: idx_leave_requests_leave_type_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_requests_leave_type_id ON public.leave_requests USING btree (leave_type_id);

--
-- Name: idx_leave_requests_requested_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_requests_requested_by ON public.leave_requests USING btree (requested_by_user_id);

--
-- Name: idx_leave_requests_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_requests_status ON public.leave_requests USING btree (status);

--
-- Name: idx_leave_types_policy_config; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_types_policy_config ON public.leave_types USING gin (policy_config);

--
-- Name: idx_leave_types_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_types_workspace_id ON public.leave_types USING btree (workspace_id);

--
-- Name: idx_otr_dept; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_otr_dept ON public.overtime_requests USING btree (department_id);

--
-- Name: idx_otr_requested_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_otr_requested_by ON public.overtime_requests USING btree (requested_by_user_id);

--
-- Name: idx_otr_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_otr_status ON public.overtime_requests USING btree (workspace_id, status);

--
-- Name: idx_otr_workspace_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_otr_workspace_date ON public.overtime_requests USING btree (workspace_id, planned_date);

--
-- Name: idx_otre_consent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_otre_consent ON public.overtime_request_employees USING btree (consent_status);

--
-- Name: idx_otre_request; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_otre_request ON public.overtime_request_employees USING btree (overtime_request_id);

--
-- Name: idx_outbox_jobs_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_outbox_jobs_created_at ON public.outbox_jobs USING btree (created_at);

--
-- Name: idx_outbox_jobs_workspace_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_outbox_jobs_workspace_status ON public.outbox_jobs USING btree (workspace_id, status);

--
-- Name: idx_overtime_policy_assignments_assignee; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_policy_assignments_assignee ON public.overtime_policy_assignments USING btree (assignee_type, assignee_id);

--
-- Name: idx_overtime_policy_assignments_policy_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_policy_assignments_policy_id ON public.overtime_policy_assignments USING btree (policy_id);

--
-- Name: idx_overtime_policy_assignments_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_policy_assignments_scope ON public.overtime_policy_assignments USING btree (workspace_id, assignee_type, assignee_id);

--
-- Name: idx_overtime_policy_definitions_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_policy_definitions_workspace ON public.overtime_policy_definitions USING btree (workspace_id);

--
-- Name: idx_overtime_request_employees_employment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_request_employees_employment_id ON public.overtime_request_employees USING btree (employment_id);

--
-- Name: idx_overtime_rules_policy_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_rules_policy_id ON public.overtime_rules USING btree (policy_id);

--
-- Name: idx_payroll_incentive_assignments_assignee; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_incentive_assignments_assignee ON public.payroll_incentive_assignments USING btree (assignee_type, assignee_id);

--
-- Name: idx_payroll_incentive_assignments_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_incentive_assignments_lookup ON public.payroll_incentive_assignments USING btree (workspace_id, assignee_type, assignee_id);

--
-- Name: idx_payroll_incentive_policies_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_incentive_policies_deleted_at ON public.payroll_incentive_policies USING btree (workspace_id, deleted_at) WHERE (deleted_at IS NULL);

--
-- Name: idx_payroll_incentive_policies_temporal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_incentive_policies_temporal ON public.payroll_incentive_policies USING btree (workspace_id, effective_from, effective_to);

--
-- Name: idx_payroll_incentive_policies_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_incentive_policies_workspace ON public.payroll_incentive_policies USING btree (workspace_id);

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

CREATE INDEX idx_payroll_rule_assignments_scope ON public.payroll_rule_assignments USING btree (workspace_id, assignee_type, assignee_id);

--
-- Name: idx_payroll_rule_assignments_temporal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_rule_assignments_temporal ON public.payroll_rule_assignments USING btree (workspace_id, effective_from, effective_to);

--
-- Name: idx_payroll_rule_assignments_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_rule_assignments_workspace ON public.payroll_rule_assignments USING btree (workspace_id);

--
-- Name: idx_payroll_rule_defs_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_rule_defs_category ON public.payroll_rule_definitions USING btree (workspace_id, category);

--
-- Name: idx_payroll_rule_defs_rule_config; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_rule_defs_rule_config ON public.payroll_rule_definitions USING gin (rule_config);

--
-- Name: idx_payroll_rule_defs_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payroll_rule_defs_workspace ON public.payroll_rule_definitions USING btree (workspace_id);

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
-- Name: idx_payslips_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_payslips_workspace_id ON public.payslips USING btree (workspace_id);

--
-- Name: idx_policy_templates_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_policy_templates_category ON public.policy_templates USING btree (category);

--
-- Name: idx_policy_templates_country; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_policy_templates_country ON public.policy_templates USING btree (country_code) WHERE (country_code IS NOT NULL);

--
-- Name: idx_policy_templates_system_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_policy_templates_system_unique ON public.policy_templates USING btree (name, version) WHERE (workspace_id IS NULL);

--
-- Name: idx_policy_templates_workspace_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_policy_templates_workspace_unique ON public.policy_templates USING btree (workspace_id, name) WHERE (workspace_id IS NOT NULL);

--
-- Name: idx_raw_attendance_logs_employment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_raw_attendance_logs_employment_id ON public.raw_attendance_logs USING btree (employment_id);

--
-- Name: idx_raw_logs_punch_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_raw_logs_punch_time ON public.raw_attendance_logs USING btree (punch_time);

--
-- Name: idx_raw_logs_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_raw_logs_workspace_id ON public.raw_attendance_logs USING btree (workspace_id);

--
-- Name: idx_recruitment_sources_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_recruitment_sources_workspace ON public.recruitment_sources USING btree (workspace_id);

--
-- Name: idx_salary_base_groups_global_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_salary_base_groups_global_slug ON public.salary_base_groups USING btree (slug) WHERE (workspace_id IS NULL);

--
-- Name: idx_salary_base_groups_workspace_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_salary_base_groups_workspace_slug ON public.salary_base_groups USING btree (workspace_id, slug) WHERE (workspace_id IS NOT NULL);

--
-- Name: idx_salary_comp_defs_linked_rule; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_salary_comp_defs_linked_rule ON public.salary_component_definitions USING btree (linked_rule_id) WHERE (linked_rule_id IS NOT NULL);

--
-- Name: idx_salary_comp_defs_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_salary_comp_defs_workspace ON public.salary_component_definitions USING btree (workspace_id);

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

--
-- Name: idx_shift_swap_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_swap_date ON public.shift_swap_requests USING btree (swap_date);

--
-- Name: idx_shift_swap_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_swap_status ON public.shift_swap_requests USING btree (workspace_id, status);

--
-- Name: idx_shift_swap_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_swap_workspace ON public.shift_swap_requests USING btree (workspace_id);

--
-- Name: idx_shift_swap_ws_requester; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_swap_ws_requester ON public.shift_swap_requests USING btree (workspace_id, requester_id);

--
-- Name: idx_shift_swap_ws_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_swap_ws_target ON public.shift_swap_requests USING btree (workspace_id, target_id);

--
-- Name: idx_shift_templates_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_templates_workspace ON public.shift_templates USING btree (workspace_id);

--
-- Name: idx_shift_versions_template; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_versions_template ON public.shift_versions USING btree (template_id);

--
-- Name: idx_shift_versions_template_effective; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_versions_template_effective ON public.shift_versions USING btree (template_id, effective_from DESC, effective_to);

--
-- Name: idx_shift_versions_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shift_versions_workspace ON public.shift_versions USING btree (workspace_id);

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
-- Name: idx_shifts_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_shifts_workspace_id ON public.shifts USING btree (workspace_id);

--
-- Name: idx_signature_requests_document; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_signature_requests_document ON public.signature_requests USING btree (document_id);

--
-- Name: idx_signature_requests_signer; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_signature_requests_signer ON public.signature_requests USING btree (signer_id);

--
-- Name: idx_signature_requests_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_signature_requests_workspace ON public.signature_requests USING btree (workspace_id);

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
-- Name: idx_time_entries_workspace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_time_entries_workspace_id ON public.time_entries USING btree (workspace_id);

--
-- Name: idx_uq_shift_swap_pair_date; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_uq_shift_swap_pair_date ON public.shift_swap_requests USING btree (workspace_id, requester_id, target_id, swap_date) WHERE ((status)::text <> ALL (ARRAY[('cancelled'::character varying)::text, ('rejected'::character varying)::text]));

--
-- Name: idx_wce_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_wce_active ON public.workspace_config_entries USING btree (workspace_id, config_key) WHERE (effective_to IS NULL);

--
-- Name: idx_wce_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_wce_lookup ON public.workspace_config_entries USING btree (workspace_id, config_key, effective_from);

--
-- Name: idx_wce_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_wce_scope ON public.workspace_config_entries USING btree (workspace_id, scope_type, scope_id);

--
-- Name: idx_wf_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_wf_category ON public.workspace_files USING btree (workspace_id, category) WHERE (is_active = true);

--
-- Name: idx_wf_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_wf_expires ON public.workspace_files USING btree (expires_at) WHERE ((expires_at IS NOT NULL) AND (is_active = true));

--
-- Name: idx_wf_latest; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_wf_latest ON public.workspace_files USING btree (workspace_id, owner_type, owner_id, is_latest) WHERE (is_latest = true);

--
-- Name: idx_wf_owner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_wf_owner ON public.workspace_files USING btree (workspace_id, owner_type, owner_id) WHERE (is_active = true);

--
-- Name: idx_wf_scan_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_wf_scan_pending ON public.workspace_files USING btree (scan_status) WHERE ((scan_status)::text = 'pending'::text);

--
-- Name: idx_work_locations_hq; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_work_locations_hq ON public.work_locations USING btree (workspace_id) WHERE (is_headquarters = true);

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
-- Name: idx_work_segments_interval_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_work_segments_interval_unique ON public.work_segments USING btree (employment_id, date, interval_id) WHERE (interval_id IS NOT NULL);

--
-- Name: idx_work_segments_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_segments_status ON public.work_segments USING btree (segment_status);

--
-- Name: idx_working_conditions_condition_type_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_working_conditions_condition_type_id ON public.working_conditions USING btree (condition_type_id);

--
-- Name: idx_working_conditions_employment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_working_conditions_employment_id ON public.working_conditions USING btree (employment_id);

--
-- Name: idx_working_patterns_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_working_patterns_workspace ON public.working_patterns USING btree (workspace_id);

--
-- Name: idx_workspace_compliance_rules_workspace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_workspace_compliance_rules_workspace ON public.workspace_compliance_rules USING btree (workspace_id);

--
-- Name: leave_types_workspace_id_code_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX leave_types_workspace_id_code_key ON public.leave_types USING btree (workspace_id, lower((code)::text));

--
-- Name: uq_departments_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_departments_name ON public.departments USING btree (workspace_id, lower((name)::text)) WHERE (deleted_at IS NULL);

--
-- Name: uq_holidays_workspace_date; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_holidays_workspace_date ON public.holidays USING btree (workspace_id, date, name) WHERE (deleted_at IS NULL);

--
-- Name: uq_leave_policy_definitions_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_leave_policy_definitions_name ON public.leave_policy_definitions USING btree (workspace_id, lower(name)) WHERE (deleted_at IS NULL);

--
-- Name: uq_working_patterns_name; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_working_patterns_name ON public.working_patterns USING btree (workspace_id, lower(name)) WHERE (deleted_at IS NULL);

--
-- Name: attendance_staging_logs_default_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_default_fingerprint_idx;

--
-- Name: attendance_staging_logs_default_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_default_pkey;

--
-- Name: attendance_staging_logs_default_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_default_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m01_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m01_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m01_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m01_pkey;

--
-- Name: attendance_staging_logs_y2026m01_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m01_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m02_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m02_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m02_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m02_pkey;

--
-- Name: attendance_staging_logs_y2026m02_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m02_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m03_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m03_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m03_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m03_pkey;

--
-- Name: attendance_staging_logs_y2026m03_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m03_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m04_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m04_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m04_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m04_pkey;

--
-- Name: attendance_staging_logs_y2026m04_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m04_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m05_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m05_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m05_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m05_pkey;

--
-- Name: attendance_staging_logs_y2026m05_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m05_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m06_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m06_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m06_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m06_pkey;

--
-- Name: attendance_staging_logs_y2026m06_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m06_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m07_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m07_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m07_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m07_pkey;

--
-- Name: attendance_staging_logs_y2026m07_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m07_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m08_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m08_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m08_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m08_pkey;

--
-- Name: attendance_staging_logs_y2026m08_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m08_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m09_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m09_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m09_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m09_pkey;

--
-- Name: attendance_staging_logs_y2026m09_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m09_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m10_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m10_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m10_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m10_pkey;

--
-- Name: attendance_staging_logs_y2026m10_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m10_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m11_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m11_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m11_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m11_pkey;

--
-- Name: attendance_staging_logs_y2026m11_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m11_workspace_id_status_idx;

--
-- Name: attendance_staging_logs_y2026m12_fingerprint_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_fingerprint ATTACH PARTITION public.attendance_staging_logs_y2026m12_fingerprint_idx;

--
-- Name: attendance_staging_logs_y2026m12_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.attendance_staging_logs_pkey1 ATTACH PARTITION public.attendance_staging_logs_y2026m12_pkey;

--
-- Name: attendance_staging_logs_y2026m12_workspace_id_status_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_staging_logs_workspace_status ATTACH PARTITION public.attendance_staging_logs_y2026m12_workspace_id_status_idx;

--
-- Name: audit_logs_default_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_default_action_idx;

--
-- Name: audit_logs_default_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_default_pkey;

--
-- Name: audit_logs_default_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_default_target_user_id_idx;

--
-- Name: audit_logs_default_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_default_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m01_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m01_action_idx;

--
-- Name: audit_logs_y2026m01_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m01_pkey;

--
-- Name: audit_logs_y2026m01_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m01_target_user_id_idx;

--
-- Name: audit_logs_y2026m01_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m01_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m02_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m02_action_idx;

--
-- Name: audit_logs_y2026m02_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m02_pkey;

--
-- Name: audit_logs_y2026m02_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m02_target_user_id_idx;

--
-- Name: audit_logs_y2026m02_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m02_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m03_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m03_action_idx;

--
-- Name: audit_logs_y2026m03_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m03_pkey;

--
-- Name: audit_logs_y2026m03_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m03_target_user_id_idx;

--
-- Name: audit_logs_y2026m03_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m03_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m04_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m04_action_idx;

--
-- Name: audit_logs_y2026m04_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m04_pkey;

--
-- Name: audit_logs_y2026m04_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m04_target_user_id_idx;

--
-- Name: audit_logs_y2026m04_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m04_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m05_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m05_action_idx;

--
-- Name: audit_logs_y2026m05_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m05_pkey;

--
-- Name: audit_logs_y2026m05_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m05_target_user_id_idx;

--
-- Name: audit_logs_y2026m05_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m05_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m06_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m06_action_idx;

--
-- Name: audit_logs_y2026m06_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m06_pkey;

--
-- Name: audit_logs_y2026m06_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m06_target_user_id_idx;

--
-- Name: audit_logs_y2026m06_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m06_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m07_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m07_action_idx;

--
-- Name: audit_logs_y2026m07_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m07_pkey;

--
-- Name: audit_logs_y2026m07_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m07_target_user_id_idx;

--
-- Name: audit_logs_y2026m07_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m07_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m08_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m08_action_idx;

--
-- Name: audit_logs_y2026m08_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m08_pkey;

--
-- Name: audit_logs_y2026m08_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m08_target_user_id_idx;

--
-- Name: audit_logs_y2026m08_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m08_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m09_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m09_action_idx;

--
-- Name: audit_logs_y2026m09_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m09_pkey;

--
-- Name: audit_logs_y2026m09_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m09_target_user_id_idx;

--
-- Name: audit_logs_y2026m09_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m09_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m10_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m10_action_idx;

--
-- Name: audit_logs_y2026m10_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m10_pkey;

--
-- Name: audit_logs_y2026m10_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m10_target_user_id_idx;

--
-- Name: audit_logs_y2026m10_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m10_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m11_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m11_action_idx;

--
-- Name: audit_logs_y2026m11_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m11_pkey;

--
-- Name: audit_logs_y2026m11_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m11_target_user_id_idx;

--
-- Name: audit_logs_y2026m11_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m11_workspace_id_created_at_idx;

--
-- Name: audit_logs_y2026m12_action_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_action ATTACH PARTITION public.audit_logs_y2026m12_action_idx;

--
-- Name: audit_logs_y2026m12_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_y2026m12_pkey;

--
-- Name: audit_logs_y2026m12_target_user_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target_user ATTACH PARTITION public.audit_logs_y2026m12_target_user_id_idx;

--
-- Name: audit_logs_y2026m12_workspace_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_workspace_created ATTACH PARTITION public.audit_logs_y2026m12_workspace_id_created_at_idx;

--
-- Name: departments trg_cascade_delete_dept_assignments; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_cascade_delete_dept_assignments BEFORE DELETE ON public.departments FOR EACH ROW EXECUTE FUNCTION public.fn_cascade_delete_department_assignments();

--
-- Name: employees trg_cascade_delete_emp_assignments; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_cascade_delete_emp_assignments BEFORE DELETE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.fn_cascade_delete_employee_assignments();

--
-- Name: assignment_rules trg_roster_cache_invalidation_assignment_rules; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_assignment_rules AFTER INSERT OR DELETE OR UPDATE ON public.assignment_rules FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

--
-- Name: employee_shifts trg_roster_cache_invalidation_employee_shifts; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_employee_shifts AFTER INSERT OR DELETE OR UPDATE ON public.employee_shifts FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('employment');

--
-- Name: employee_working_patterns trg_roster_cache_invalidation_employee_working_patterns; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_employee_working_patterns AFTER INSERT OR DELETE OR UPDATE ON public.employee_working_patterns FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

--
-- Name: employees trg_roster_cache_invalidation_employees; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_employees AFTER INSERT OR DELETE OR UPDATE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

--
-- Name: employments trg_roster_cache_invalidation_employments; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_employments AFTER INSERT OR DELETE OR UPDATE ON public.employments FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

--
-- Name: shift_default_intervals trg_roster_cache_invalidation_shift_default_intervals; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_shift_default_intervals AFTER INSERT OR DELETE OR UPDATE ON public.shift_default_intervals FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('shift_interval');

--
-- Name: shift_templates trg_roster_cache_invalidation_shift_templates; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_shift_templates AFTER INSERT OR DELETE OR UPDATE ON public.shift_templates FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

--
-- Name: shift_versions trg_roster_cache_invalidation_shift_versions; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_shift_versions AFTER INSERT OR DELETE OR UPDATE ON public.shift_versions FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

--
-- Name: work_segments trg_roster_cache_invalidation_work_segments; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_work_segments AFTER INSERT OR DELETE OR UPDATE ON public.work_segments FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

--
-- Name: working_pattern_details trg_roster_cache_invalidation_working_pattern_details; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_working_pattern_details AFTER INSERT OR DELETE OR UPDATE ON public.working_pattern_details FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('working_pattern');

--
-- Name: working_patterns trg_roster_cache_invalidation_working_patterns; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_roster_cache_invalidation_working_patterns AFTER INSERT OR DELETE OR UPDATE ON public.working_patterns FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

--
-- Name: assignment_rules trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.assignment_rules FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: attendance_adjustment_requests trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_adjustment_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: attendance_daily_summaries trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_daily_summaries FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: attendance_import_jobs trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_import_jobs FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: attendance_policies trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_policies FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: attendance_policy_assignments trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.attendance_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: dashboard_preferences trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.dashboard_preferences FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: departments trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.departments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: employee_phases trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_phases FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: employee_profiles trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_profiles FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: employee_salary_components trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_salary_components FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: employee_shifts trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_shifts FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: employee_working_patterns trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employee_working_patterns FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: employees trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employees FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: employment_phase_definitions trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.employment_phase_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: feature_flags trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.feature_flags FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: holidays trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.holidays FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: leave_balances trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_balances FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: leave_policy_definitions trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_policy_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: leave_requests trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.leave_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: overtime_policy_definitions trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.overtime_policy_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: overtime_rules trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.overtime_rules FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: payroll_rule_definitions trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.payroll_rule_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: payslips trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.payslips FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: policy_templates trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.policy_templates FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: recruitment_sources trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.recruitment_sources FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: salary_base_groups trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.salary_base_groups FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: salary_component_definitions trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.salary_component_definitions FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: shift_default_intervals trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_default_intervals FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: shift_rosters trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_rosters FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: shift_swap_requests trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shift_swap_requests FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: shifts trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.shifts FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: work_segments trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.work_segments FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: working_patterns trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.working_patterns FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: workspace_compliance_rules trg_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.workspace_compliance_rules FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

--
-- Name: leave_policy_assignments trg_validate_assignee_fk; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.leave_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

--
-- Name: overtime_policy_assignments trg_validate_assignee_fk; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.overtime_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

--
-- Name: payroll_incentive_assignments trg_validate_assignee_fk; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.payroll_incentive_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

--
-- Name: payroll_rule_assignments trg_validate_assignee_fk; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_validate_assignee_fk BEFORE INSERT OR UPDATE OF assignee_type, assignee_id ON public.payroll_rule_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

--
-- Name: attendance_policy_assignments trg_validate_target_fk; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_validate_target_fk BEFORE INSERT OR UPDATE OF target_type, target_id ON public.attendance_policy_assignments FOR EACH ROW EXECUTE FUNCTION public.fn_validate_target_fk();

--
-- Name: outbox_jobs trigger_outbox_jobs_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trigger_outbox_jobs_updated_at BEFORE UPDATE ON public.outbox_jobs FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

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
-- Name: attendance_daily_summaries attendance_daily_summaries_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

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
-- Name: behalf_assignments behalf_assignments_employer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT behalf_assignments_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE SET NULL;

--
-- Name: custom_field_values custom_field_values_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.custom_field_values
    ADD CONSTRAINT custom_field_values_definition_id_fkey FOREIGN KEY (definition_id) REFERENCES public.custom_field_definitions(id) ON DELETE CASCADE;

--
-- Name: departments departments_default_working_pattern_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_default_working_pattern_id_fkey FOREIGN KEY (default_working_pattern_id) REFERENCES public.working_patterns(id) ON DELETE SET NULL;

--
-- Name: departments departments_manager_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_manager_id_fkey FOREIGN KEY (manager_id) REFERENCES public.employees(id) ON DELETE SET NULL;

--
-- Name: document_folders document_folders_parent_folder_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_folders
    ADD CONSTRAINT document_folders_parent_folder_id_fkey FOREIGN KEY (parent_folder_id) REFERENCES public.document_folders(id) ON DELETE CASCADE;

--
-- Name: econtract_envelopes econtract_envelopes_contract_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.econtract_envelopes
    ADD CONSTRAINT econtract_envelopes_contract_id_fkey FOREIGN KEY (contract_id) REFERENCES public.employee_contracts(id) ON DELETE CASCADE;

--
-- Name: econtract_envelopes econtract_envelopes_employer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.econtract_envelopes
    ADD CONSTRAINT econtract_envelopes_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;

--
-- Name: employee_assignments employee_assignments_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

--
-- Name: employee_assignments employee_assignments_home_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_home_location_id_fkey FOREIGN KEY (home_location_id) REFERENCES public.work_locations(id);

--
-- Name: employee_assignments employee_assignments_host_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_assignments
    ADD CONSTRAINT employee_assignments_host_location_id_fkey FOREIGN KEY (host_location_id) REFERENCES public.work_locations(id);

--
-- Name: employee_contracts employee_contracts_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.employee_documents(id) ON DELETE SET NULL;

--
-- Name: employee_contracts employee_contracts_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

--
-- Name: employee_documents employee_documents_employee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_documents
    ADD CONSTRAINT employee_documents_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

--
-- Name: employee_emergency_contacts employee_emergency_contacts_employee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_emergency_contacts
    ADD CONSTRAINT employee_emergency_contacts_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

--
-- Name: employee_phases employee_phases_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;

--
-- Name: employee_phases employee_phases_phase_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_phases
    ADD CONSTRAINT employee_phases_phase_definition_id_fkey FOREIGN KEY (phase_definition_id) REFERENCES public.employment_phase_definitions(id) ON DELETE RESTRICT;

--
-- Name: employee_profiles employee_profiles_avatar_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_avatar_file_id_fkey FOREIGN KEY (avatar_file_id) REFERENCES public.workspace_files(id);

--
-- Name: employee_profiles employee_profiles_employee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

--
-- Name: employee_salary_components employee_salary_components_component_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_salary_components
    ADD CONSTRAINT employee_salary_components_component_definition_id_fkey FOREIGN KEY (component_definition_id) REFERENCES public.salary_component_definitions(id) ON DELETE CASCADE;

--
-- Name: employee_salary_components employee_salary_components_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employee_salary_components
    ADD CONSTRAINT employee_salary_components_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;

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
-- Name: employees employees_recruitment_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_recruitment_source_id_fkey FOREIGN KEY (recruitment_source_id) REFERENCES public.recruitment_sources(id) ON DELETE SET NULL;

--
-- Name: employer_econtract_configs employer_econtract_configs_employer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employer_econtract_configs
    ADD CONSTRAINT employer_econtract_configs_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE CASCADE;

--
-- Name: employer_econtract_routing_rules employer_econtract_routing_rules_employer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employer_econtract_routing_rules
    ADD CONSTRAINT employer_econtract_routing_rules_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE CASCADE;

--
-- Name: employers employers_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employers
    ADD CONSTRAINT employers_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.employers(id) ON DELETE SET NULL;

--
-- Name: employments employments_department_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id) ON DELETE SET NULL;

--
-- Name: employments employments_employee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE RESTRICT;

--
-- Name: employments employments_employer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT;

--
-- Name: employments employments_manager_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_manager_employment_id_fkey FOREIGN KEY (manager_employment_id) REFERENCES public.employments(id) ON DELETE SET NULL;

--
-- Name: employments employments_primary_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_primary_location_id_fkey FOREIGN KEY (primary_location_id) REFERENCES public.work_locations(id) ON DELETE SET NULL;

--
-- Name: file_acknowledgements file_acknowledgements_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_acknowledgements
    ADD CONSTRAINT file_acknowledgements_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.workspace_files(id) ON DELETE CASCADE;

--
-- Name: file_acknowledgements file_acknowledgements_employee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_acknowledgements
    ADD CONSTRAINT file_acknowledgements_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

--
-- Name: workspace_files fk_workspace_files_folder_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_files
    ADD CONSTRAINT fk_workspace_files_folder_id FOREIGN KEY (folder_id) REFERENCES public.document_folders(id) ON DELETE SET NULL;

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
-- Name: leave_accrual_logs leave_accrual_logs_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE CASCADE;

--
-- Name: leave_accrual_logs leave_accrual_logs_leave_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

--
-- Name: leave_accrual_logs leave_accrual_logs_policy_assignment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_accrual_logs
    ADD CONSTRAINT leave_accrual_logs_policy_assignment_id_fkey FOREIGN KEY (policy_assignment_id) REFERENCES public.leave_policy_assignments(id) ON DELETE SET NULL;

--
-- Name: leave_balances leave_balances_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

--
-- Name: leave_balances leave_balances_leave_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_balances
    ADD CONSTRAINT leave_balances_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

--
-- Name: leave_policy_assignments leave_policy_assignments_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_policy_assignments
    ADD CONSTRAINT leave_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.leave_policy_definitions(id) ON DELETE RESTRICT;

--
-- Name: leave_policy_definitions leave_policy_definitions_leave_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_policy_definitions
    ADD CONSTRAINT leave_policy_definitions_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

--
-- Name: leave_requests leave_requests_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

--
-- Name: leave_requests leave_requests_leave_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_leave_type_id_fkey FOREIGN KEY (leave_type_id) REFERENCES public.leave_types(id) ON DELETE RESTRICT;

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
-- Name: behalf_assignments ot_claim_behalf_assignments_target_department_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT ot_claim_behalf_assignments_target_department_id_fkey FOREIGN KEY (target_department_id) REFERENCES public.departments(id) ON DELETE CASCADE;

--
-- Name: behalf_assignments ot_claim_behalf_assignments_target_employee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.behalf_assignments
    ADD CONSTRAINT ot_claim_behalf_assignments_target_employee_id_fkey FOREIGN KEY (target_employee_id) REFERENCES public.employees(id) ON DELETE CASCADE;

--
-- Name: overtime_policy_assignments overtime_policy_assignments_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_policy_assignments
    ADD CONSTRAINT overtime_policy_assignments_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.overtime_policy_definitions(id) ON DELETE RESTRICT;

--
-- Name: overtime_request_employees overtime_request_employees_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

--
-- Name: overtime_request_employees overtime_request_employees_matched_summary_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_request_employees
    ADD CONSTRAINT overtime_request_employees_matched_summary_id_fkey FOREIGN KEY (matched_summary_id) REFERENCES public.attendance_daily_summaries(id);

--
-- Name: overtime_requests overtime_requests_department_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id);

--
-- Name: overtime_requests overtime_requests_workflow_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_requests
    ADD CONSTRAINT overtime_requests_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.approval_workflows(id);

--
-- Name: overtime_rules overtime_rules_policy_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_rules
    ADD CONSTRAINT overtime_rules_policy_id_fkey FOREIGN KEY (policy_id) REFERENCES public.overtime_policy_definitions(id) ON DELETE CASCADE;

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
-- Name: raw_attendance_logs raw_attendance_logs_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.raw_attendance_logs
    ADD CONSTRAINT raw_attendance_logs_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

--
-- Name: recruitment_sources recruitment_sources_referrer_employee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.recruitment_sources
    ADD CONSTRAINT recruitment_sources_referrer_employee_id_fkey FOREIGN KEY (referrer_employee_id) REFERENCES public.employees(id) ON DELETE SET NULL;

--
-- Name: salary_component_base_members salary_component_base_members_base_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salary_component_base_members
    ADD CONSTRAINT salary_component_base_members_base_group_id_fkey FOREIGN KEY (base_group_id) REFERENCES public.salary_base_groups(id) ON DELETE CASCADE;

--
-- Name: salary_component_base_members salary_component_base_members_component_definition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salary_component_base_members
    ADD CONSTRAINT salary_component_base_members_component_definition_id_fkey FOREIGN KEY (component_definition_id) REFERENCES public.salary_component_definitions(id) ON DELETE CASCADE;

--
-- Name: salary_component_definitions salary_component_definitions_linked_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salary_component_definitions
    ADD CONSTRAINT salary_component_definitions_linked_rule_id_fkey FOREIGN KEY (linked_rule_id) REFERENCES public.payroll_rule_definitions(id) ON DELETE SET NULL;

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

--
-- Name: shift_swap_requests shift_swap_requests_requester_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.shift_swap_requests
    ADD CONSTRAINT shift_swap_requests_requester_id_fkey FOREIGN KEY (requester_id) REFERENCES public.employees(id) ON DELETE CASCADE;

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
-- Name: signature_requests signature_requests_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.signature_requests
    ADD CONSTRAINT signature_requests_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.workspace_files(id) ON DELETE CASCADE;

--
-- Name: signature_requests signature_requests_signer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.signature_requests
    ADD CONSTRAINT signature_requests_signer_id_fkey FOREIGN KEY (signer_id) REFERENCES public.employees(id) ON DELETE CASCADE;

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
-- Name: work_segments work_segments_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.shift_templates(id) ON DELETE RESTRICT;

--
-- Name: working_conditions working_conditions_condition_type_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_condition_type_id_fkey FOREIGN KEY (condition_type_id) REFERENCES public.condition_type_definitions(id) ON DELETE SET NULL;

--
-- Name: working_conditions working_conditions_employment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_employment_id_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT;

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
-- Name: workspace_files workspace_files_parent_file_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.workspace_files
    ADD CONSTRAINT workspace_files_parent_file_id_fkey FOREIGN KEY (parent_file_id) REFERENCES public.workspace_files(id);

--
-- PostgreSQL database dump complete
--
