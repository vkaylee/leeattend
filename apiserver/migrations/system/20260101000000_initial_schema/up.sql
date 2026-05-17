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
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

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

CREATE TABLE public.sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    token text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    revoked_at timestamp with time zone
);

CREATE TABLE public.system_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.user_system_roles (
    user_id uuid NOT NULL,
    role_id uuid NOT NULL,
    assigned_by uuid,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    email character varying(255) NOT NULL,
    password_hash character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    full_name character varying(100),
    avatar_url text,
    bio text,
    preferred_language character varying(10)
);

CREATE TABLE public.workspaces (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
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