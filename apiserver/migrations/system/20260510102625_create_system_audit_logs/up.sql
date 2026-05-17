CREATE TABLE public.system_audit_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_id uuid REFERENCES public.users(id) ON DELETE SET NULL,
    action character varying(100) NOT NULL,
    details jsonb,
    ip_address inet,
    user_agent text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);