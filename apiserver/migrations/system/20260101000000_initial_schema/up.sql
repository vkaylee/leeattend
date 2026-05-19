CREATE TABLE public.audit_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    workspace_id uuid NOT NULL,
    actor_id uuid,
    target_user_id uuid,
    target_resource_id uuid,
    action character varying(255) NOT NULL,
    details jsonb,
    ip_address text,
    user_agent text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.authorized_agents (
    id character varying(50) NOT NULL PRIMARY KEY,
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

CREATE TABLE public.sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    user_id uuid NOT NULL,
    token text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    revoked_at timestamp with time zone
);

CREATE TABLE public.system_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    name text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.user_system_roles (
    user_id uuid NOT NULL,
    role_id uuid NOT NULL,
    assigned_by uuid,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    email character varying(255) NOT NULL UNIQUE,
    password_hash character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    full_name character varying(100),
    avatar_url text,
    bio text,
    preferred_language character varying(10)
);

CREATE TABLE public.workspaces (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    name character varying(100) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    date_format character varying(20) DEFAULT 'DD/MM/YYYY'::character varying NOT NULL,
    time_format character varying(10) DEFAULT '24h'::character varying NOT NULL,
    timezone character varying(50) DEFAULT 'Asia/Ho_Chi_Minh'::character varying NOT NULL,
    theme_primary_color text DEFAULT '#7c3aed'::text,
    default_language character varying(10) DEFAULT 'vi'::character varying NOT NULL,
    holiday_overnight_strategy character varying(30) DEFAULT 'split_at_midnight'::character varying NOT NULL,
    default_holiday_ot_multiplier numeric(3,1) DEFAULT 3.0 NOT NULL,
    CONSTRAINT chk_workspace_name CHECK ((length((name)::text) > 0))
);

-- ============================================================
-- Foreign Keys (within system database)
-- ============================================================

ALTER TABLE ONLY public.audit_logs
    ADD CONSTRAINT audit_logs_actor_id_fkey FOREIGN KEY (actor_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.audit_logs
    ADD CONSTRAINT audit_logs_target_user_id_fkey FOREIGN KEY (target_user_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.audit_logs
    ADD CONSTRAINT audit_logs_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.authorized_agents
    ADD CONSTRAINT authorized_agents_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_token_key UNIQUE (token);

ALTER TABLE ONLY public.system_roles
    ADD CONSTRAINT system_roles_name_key UNIQUE (name);

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.system_roles(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_system_roles
    ADD CONSTRAINT user_system_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- ============================================================
-- Indexes
-- ============================================================

CREATE INDEX idx_audit_logs_actor_id ON public.audit_logs USING btree (actor_id);

CREATE INDEX idx_audit_logs_created_at ON public.audit_logs USING btree (created_at);

CREATE INDEX idx_audit_logs_target_user_id ON public.audit_logs USING btree (target_user_id);

CREATE INDEX idx_audit_logs_workspace_id ON public.audit_logs USING btree (workspace_id);

CREATE INDEX idx_authorized_agents_trusted ON public.authorized_agents USING btree (is_trusted);

CREATE INDEX idx_authorized_agents_workspace ON public.authorized_agents USING btree (workspace_id);

CREATE INDEX idx_sessions_token ON public.sessions USING btree (token);

CREATE INDEX idx_sessions_user_id ON public.sessions USING btree (user_id);

CREATE INDEX idx_user_system_roles_role_id ON public.user_system_roles USING btree (role_id);

CREATE INDEX idx_user_system_roles_user_id ON public.user_system_roles USING btree (user_id);

-- ============================================================
-- Seed Data: Platform-level system roles
-- ============================================================

INSERT INTO system_roles (id, name, description) VALUES
    ('c0000000-0000-0000-0000-000000000001', 'platform_admin', 'Full platform control: create/suspend workspaces, assign system roles, manage infrastructure'),
    ('c0000000-0000-0000-0000-000000000002', 'support',        'Read-only cross-workspace access for debugging customer issues'),
    ('c0000000-0000-0000-0000-000000000003', 'billing_admin',  'Manage billing, subscription plans, and invoicing')
ON CONFLICT DO NOTHING;