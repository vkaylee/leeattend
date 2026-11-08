-- Upgrade test fixture for single-organization migration chain
-- Establishes valid standalone baseline state satisfying 20261106000000_remove_system_roles preflight
-- Synthetic test IDs, no PII

BEGIN;

INSERT INTO public.workspaces (
    id, name, date_format, time_format, timezone, theme_primary_color, default_language, logo_url, tagline
) VALUES (
    '11111111-1111-4111-8111-111111111111',
    'Acme Corp',
    'DD/MM/YYYY',
    '24h',
    'Asia/Ho_Chi_Minh',
    '#7c3aed',
    'vi',
    'https://example.com/logo.png',
    'Testing Cutover'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.users (
    id, email, password_hash, full_name, preferred_language
) VALUES (
    '22222222-2222-4222-8222-222222222222',
    'fixture.admin@example.invalid',
    '$argon2id$v=19$m=65536,t=3,p=4$dummyhashforfixtureonly$dummyhash',
    'Fixture Admin',
    'vi'
), (
    '33333333-3333-4333-8333-333333333333',
    'fixture.user@example.invalid',
    '$argon2id$v=19$m=65536,t=3,p=4$dummyhashforfixtureonly$dummyhash',
    'Fixture User',
    'vi'
) ON CONFLICT (id) DO NOTHING;

-- Global admin role (satisfies 060 preflight)
INSERT INTO public.workspace_roles (
    id, workspace_id, name, description, color, is_system, scope_type
) VALUES (
    '44444444-4444-4444-8444-444444444444',
    NULL,
    'admin',
    'Global Organization Admin',
    'purple',
    true,
    'global'
), (
    '55555555-5555-4555-8555-555555555555',
    '11111111-1111-4111-8111-111111111111',
    'employee',
    'Local Workspace Member',
    'slate',
    false,
    'scoped'
) ON CONFLICT (id) DO NOTHING;

-- Assign admin user to global admin role
INSERT INTO public.workspace_members (
    workspace_id, user_id, role_id
) VALUES (
    '11111111-1111-4111-8111-111111111111',
    '22222222-2222-4222-8222-222222222222',
    '44444444-4444-4444-8444-444444444444'
), (
    '11111111-1111-4111-8111-111111111111',
    '33333333-3333-4333-8333-333333333333',
    '55555555-5555-4555-8555-555555555555'
) ON CONFLICT (workspace_id, user_id) DO NOTHING;

-- Baseline role permissions (include old workspace:settings:* permissions to test normalization)
INSERT INTO public.workspace_role_permissions (role_id, permission_name) VALUES
    ('44444444-4444-4444-8444-444444444444', 'workspace:settings:read'),
    ('44444444-4444-4444-8444-444444444444', 'workspace:settings:write'),
    ('44444444-4444-4444-8444-444444444444', 'employees:read'),
    ('55555555-5555-4555-8555-555555555555', 'attendance:self')
ON CONFLICT (role_id, permission_name) DO NOTHING;

-- Extra user permissions
INSERT INTO public.workspace_member_extra_permissions (
    id, workspace_id, user_id, permission, expires_at
) VALUES (
    '66666666-6666-4666-8666-666666666666',
    '11111111-1111-4111-8111-111111111111',
    '33333333-3333-4333-8333-333333333333',
    'workspace:settings:read',
    now() + interval '30 days'
) ON CONFLICT (id) DO NOTHING;

-- Config entries (scope_type='workspace' to verify rewrite)
INSERT INTO public.workspace_config_entries (
    id, workspace_id, scope_type, scope_id, config_key, value_text, effective_from
) VALUES (
    '77777777-7777-4777-8777-777777777777',
    '11111111-1111-4111-8111-111111111111',
    'workspace',
    NULL,
    'payroll.cutoff_day',
    '25',
    '2026-01-01'
) ON CONFLICT (id) DO NOTHING;

-- Storage config
INSERT INTO public.storage_configs (
    id, workspace_id, endpoint_url, region, bucket_name, access_key_enc, secret_key_enc, encryption_key_version
) VALUES (
    '88888888-8888-4888-8888-888888888888',
    '11111111-1111-4111-8111-111111111111',
    'http://minio:9000',
    'us-east-1',
    'leeattend',
    'enc:test',
    'enc:test',
    'v1'
) ON CONFLICT (id) DO NOTHING;

-- Compliance rules
INSERT INTO public.workspace_compliance_rules (
    id, workspace_id, min_rest_period_hours, max_weekly_hours, max_monthly_ot_hours
) VALUES (
    '99999999-9999-4999-8999-999999999999',
    '11111111-1111-4111-8111-111111111111',
    12, 48, 40
) ON CONFLICT (id) DO NOTHING;

-- Organization logo file (to verify files rename, category and owner_type update)
INSERT INTO public.workspace_files (
    id, workspace_id, file_name, storage_key, file_size, mime_type, category, owner_type, owner_id
) VALUES (
    'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
    '11111111-1111-4111-8111-111111111111',
    'logo.png',
    'org/logo.png',
    1024,
    'image/png',
    'workspace_logo',
    'workspace',
    '11111111-1111-4111-8111-111111111111'
) ON CONFLICT (id) DO NOTHING;

-- Department & Employee
INSERT INTO public.departments (
    id, workspace_id, name
) VALUES (
    'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    '11111111-1111-4111-8111-111111111111',
    'Engineering'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.employees (
    id, workspace_id, user_id, employee_id_number, full_name, hire_date, original_hire_date
) VALUES (
    'cccccccc-cccc-4ccc-8ccc-cccccccccccc',
    '11111111-1111-4111-8111-111111111111',
    '33333333-3333-4333-8333-333333333333',
    'EMP001',
    'Fixture Employee',
    '2026-01-01',
    '2026-01-01'
);
-- Employer, Location, and Employment
INSERT INTO public.employers (
    id, workspace_id, name, legal_address
) VALUES (
    '12121212-1212-4212-8212-121212121212',
    '11111111-1111-4111-8111-111111111111',
    'Acme Employer',
    '123 Corporate Blvd'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.work_locations (
    id, workspace_id, name, is_headquarters
) VALUES (
    '13131313-1313-4313-8313-131313131313',
    '11111111-1111-4111-8111-111111111111',
    'Headquarters',
    true
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.employments (
    id, workspace_id, employee_id, employer_id, department_id, primary_location_id,
    status, start_date, job_title
) VALUES (
    '14141414-1414-4414-8414-141414141414',
    '11111111-1111-4111-8111-111111111111',
    'cccccccc-cccc-4ccc-8ccc-cccccccccccc',
    '12121212-1212-4212-8212-121212121212',
    'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    '13131313-1313-4313-8313-131313131313',
    'active',
    '2026-01-01',
    'Software Engineer'
) ON CONFLICT (id) DO NOTHING;

-- Attendance Daily Summary and Default Policy
INSERT INTO public.attendance_policies (
    id, workspace_id, name, is_default
) VALUES (
    '15151515-1515-4515-8515-151515151515',
    '11111111-1111-4111-8111-111111111111',
    'Default Policy',
    true
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.attendance_daily_summaries (
    id, workspace_id, employment_id, date, actual_work_minutes
) VALUES (
    '16161616-1616-4616-8616-161616161616',
    '11111111-1111-4111-8111-111111111111',
    '14141414-1414-4414-8414-141414141414',
    '2026-01-02',
    480
) ON CONFLICT (id) DO NOTHING;

-- Payslip row
INSERT INTO public.payslips (
    id, workspace_id, employment_id, period_start, period_end, base_pay_amount, net_pay_amount
) VALUES (
    '17171717-1717-4717-8717-171717171717',
    '11111111-1111-4111-8111-111111111111',
    '14141414-1414-4414-8414-141414141414',
    '2026-01-01',
    '2026-01-31',
    20000000,
    18000000
) ON CONFLICT (id) DO NOTHING;

-- Policy definitions and assignments (to verify assignee_type rewrite and triggers)
INSERT INTO public.leave_types (
    id, workspace_id, code, name
) VALUES (
    'dddddddd-dddd-4ddd-8ddd-dddddddddddd',
    '11111111-1111-4111-8111-111111111111',
    'AL',
    'Annual Leave'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.leave_policy_definitions (
    id, workspace_id, name, leave_type_id
) VALUES (
    'eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee',
    '11111111-1111-4111-8111-111111111111',
    'Standard Leave Policy',
    'dddddddd-dddd-4ddd-8ddd-dddddddddddd'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.leave_policy_assignments (
    id, workspace_id, assignee_type, assignee_id, policy_id, effective_from
) VALUES (
    'ffffffff-ffff-4fff-8fff-ffffffffffff',
    '11111111-1111-4111-8111-111111111111',
    'workspace',
    '11111111-1111-4111-8111-111111111111',
    'eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee',
    '2026-01-01'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.payroll_rule_definitions (
    id, workspace_id, name, code
) VALUES (
    '00000000-0000-4000-8000-000000000001',
    '11111111-1111-4111-8111-111111111111',
    'Base Salary',
    'BASE'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO public.payroll_rule_assignments (
    id, workspace_id, rule_definition_id, assignee_type, assignee_id, effective_from
) VALUES (
    '00000000-0000-4000-8000-000000000002',
    '11111111-1111-4111-8111-111111111111',
    '00000000-0000-4000-8000-000000000001',
    'workspace',
    '11111111-1111-4111-8111-111111111111',
    '2026-01-01'
) ON CONFLICT (id) DO NOTHING;

COMMIT;
