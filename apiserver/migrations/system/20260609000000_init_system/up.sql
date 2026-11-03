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
