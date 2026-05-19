-- Move workspace-related tables from tenant to system database
-- Tables: workspace_members, workspace_roles, workspace_role_permissions,
--         member_extra_roles, workspace_member_extra_permissions

-- These tables are cross-cutting (user-workspace mapping) and belong in system DB.

CREATE TABLE public.workspace_members (
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    role_id uuid NOT NULL
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

CREATE TABLE public.workspace_role_permissions (
    role_id uuid NOT NULL,
    permission_name text NOT NULL
);

CREATE TABLE public.member_extra_roles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    user_id uuid NOT NULL,
    assigned_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    expires_at timestamp with time zone NOT NULL,
    role_id uuid NOT NULL
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

-- Primary Keys (must be created before FKs that reference them)

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_pkey PRIMARY KEY (workspace_id, user_id);

ALTER TABLE ONLY public.workspace_roles
    ADD CONSTRAINT workspace_roles_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.workspace_role_permissions
    ADD CONSTRAINT workspace_role_permissions_pkey PRIMARY KEY (role_id, permission_name);

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_permissions_pkey PRIMARY KEY (id);

-- Foreign Keys (now valid since all tables are in system DB)

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id);

ALTER TABLE ONLY public.workspace_role_permissions
    ADD CONSTRAINT workspace_role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_workspace_id_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.member_extra_roles
    ADD CONSTRAINT member_extra_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.workspace_roles(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_perms_ws_fkey FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workspace_member_extra_permissions
    ADD CONSTRAINT workspace_member_extra_perms_user_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- Triggers

CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.workspace_roles
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- Indexes

CREATE INDEX idx_workspace_members_user_id ON public.workspace_members USING btree (user_id);

CREATE INDEX idx_workspace_members_workspace_id ON public.workspace_members USING btree (workspace_id);

CREATE UNIQUE INDEX idx_system_roles_unique_name ON public.workspace_roles USING btree (name) WHERE (workspace_id IS NULL);

CREATE INDEX idx_workspace_roles_scope_type ON public.workspace_roles USING btree (scope_type);

CREATE UNIQUE INDEX idx_workspace_roles_unique_name ON public.workspace_roles USING btree (workspace_id, name) WHERE (workspace_id IS NOT NULL);

CREATE INDEX idx_member_extra_roles_lookup ON public.member_extra_roles USING btree (workspace_id, user_id, expires_at);

CREATE INDEX idx_extra_perms_lookup ON public.workspace_member_extra_permissions USING btree (workspace_id, user_id, expires_at);

-- Seed data: default workspace roles and permissions

INSERT INTO workspace_roles (id, workspace_id, name, description, color, is_system, scope_type)
VALUES
  ('a0000000-0000-0000-0000-000000000001', NULL, 'owner', 'Workspace owner with full access', 'rose', TRUE, 'global'),
  ('a0000000-0000-0000-0000-000000000002', NULL, 'admin', 'Administrator with full access', 'rose', TRUE, 'global'),
  ('a0000000-0000-0000-0000-000000000003', NULL, 'hr_manager', 'Manage people and time', 'indigo', TRUE, 'global'),
  ('a0000000-0000-0000-0000-000000000004', NULL, 'shift_manager', 'Manage shifts', 'amber', TRUE, 'global'),
  ('a0000000-0000-0000-0000-000000000005', NULL, 'team_lead', 'Manage team members', 'blue', TRUE, 'scoped'),
  ('a0000000-0000-0000-0000-000000000006', NULL, 'employee', 'Default employee access', 'slate', TRUE, 'scoped')
ON CONFLICT DO NOTHING;

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

INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000005', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000005', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:submit'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:approve')
ON CONFLICT DO NOTHING;

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
