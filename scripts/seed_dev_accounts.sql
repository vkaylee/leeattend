-- =============================================================================
-- Seed Complete Development Accounts with 5 Core On-Premise Roles
-- Password for all accounts: development-password
-- =============================================================================

DO $$
DECLARE
    v_ws_id uuid := '3bd6c304-bb37-440b-837b-da220bb63587';
    v_pwd_hash text := '$argon2id$v=19$m=65536,t=1,p=4$Q71XRJT8SsPnqVciqiphLA$i37H/ySyg61+aIj0BzIV3RAPSRP09elhC0wocBSTv5Y';
    v_employer_id uuid := 'e0000000-0000-0000-0000-000000000001';
    v_dept_exec uuid := 'd0000000-0000-0000-0000-000000000001';
    v_dept_hr uuid := 'd0000000-0000-0000-0000-000000000002';
    v_dept_tech uuid := 'd0000000-0000-0000-0000-000000000003';
BEGIN
    -- Ensure Workspace exists
    INSERT INTO workspaces (id, name)
    VALUES (v_ws_id, 'Development Organization')
    ON CONFLICT (id) DO NOTHING;

    -- Ensure Employer exists
    INSERT INTO employers (id, workspace_id, name, tax_code)
    VALUES (v_employer_id, v_ws_id, 'LeeAttend Enterprise Corp', '0101234567')
    ON CONFLICT (id) DO NOTHING;

    -- Ensure Departments exist
    INSERT INTO departments (id, workspace_id, name, description)
    VALUES 
        (v_dept_exec, v_ws_id, 'Ban Giám Đốc', 'Executive & Platform Management'),
        (v_dept_hr, v_ws_id, 'Phòng Nhân Sự', 'Human Resources Management'),
        (v_dept_tech, v_ws_id, 'Phòng Kỹ Thuật', 'Engineering & Operations')
    ON CONFLICT (id) DO NOTHING;

    -- Clean up deprecated SaaS accounts
    DELETE FROM users WHERE email IN ('support@example.invalid', 'billing@example.invalid', 'workspace.admin@example.invalid');

    -- =========================================================================
    -- 1. Create Core Users (5 Accounts)
    -- =========================================================================
    -- 1. admin@example.invalid (System & Company Administrator)
    INSERT INTO users (id, email, password_hash, full_name, created_at)
    VALUES ('e80200c2-eec3-4fc1-8135-de9f8f264396', 'admin@example.invalid', v_pwd_hash, 'System Administrator', NOW())
    ON CONFLICT (email) DO UPDATE SET password_hash = v_pwd_hash, full_name = 'System Administrator';

    -- 2. hr.manager@example.invalid (HR Manager)
    INSERT INTO users (id, email, password_hash, full_name, created_at)
    VALUES ('e80200c2-0000-0000-0000-000000000005', 'hr.manager@example.invalid', v_pwd_hash, 'HR Manager', NOW())
    ON CONFLICT (email) DO UPDATE SET password_hash = v_pwd_hash, full_name = 'HR Manager';

    -- 3. shift.manager@example.invalid (Shift Manager)
    INSERT INTO users (id, email, password_hash, full_name, created_at)
    VALUES ('e80200c2-0000-0000-0000-000000000006', 'shift.manager@example.invalid', v_pwd_hash, 'Shift Operations Manager', NOW())
    ON CONFLICT (email) DO UPDATE SET password_hash = v_pwd_hash, full_name = 'Shift Operations Manager';

    -- 4. team.lead@example.invalid (Team Lead)
    INSERT INTO users (id, email, password_hash, full_name, created_at)
    VALUES ('e80200c2-0000-0000-0000-000000000007', 'team.lead@example.invalid', v_pwd_hash, 'Technical Team Lead', NOW())
    ON CONFLICT (email) DO UPDATE SET password_hash = v_pwd_hash, full_name = 'Technical Team Lead';

    -- 5. employee@example.invalid (Standard Employee / ESS)
    INSERT INTO users (id, email, password_hash, full_name, created_at)
    VALUES ('e80200c2-0000-0000-0000-000000000008', 'employee@example.invalid', v_pwd_hash, 'Standard Staff Member', NOW())
    ON CONFLICT (email) DO UPDATE SET password_hash = v_pwd_hash, full_name = 'Standard Staff Member';

    -- =========================================================================
    -- 2. System Roles Assignment (user_system_roles)
    -- =========================================================================
    -- admin -> admin (System Administrator)
    INSERT INTO user_system_roles (user_id, role_id)
    SELECT 'e80200c2-eec3-4fc1-8135-de9f8f264396', id FROM system_roles WHERE name = 'admin'
    ON CONFLICT DO NOTHING;

    -- =========================================================================
    -- 3. Workspace Roles Assignment (workspace_members)
    -- =========================================================================
    -- admin -> admin (Administrator with full access)
    INSERT INTO workspace_members (workspace_id, user_id, role_id)
    SELECT v_ws_id, 'e80200c2-eec3-4fc1-8135-de9f8f264396', id FROM workspace_roles WHERE workspace_id IS NULL AND name = 'admin'
    ON CONFLICT (workspace_id, user_id) DO UPDATE SET role_id = EXCLUDED.role_id;

    -- hr.manager -> hr_manager
    INSERT INTO workspace_members (workspace_id, user_id, role_id)
    SELECT v_ws_id, 'e80200c2-0000-0000-0000-000000000005', id FROM workspace_roles WHERE workspace_id IS NULL AND name = 'hr_manager'
    ON CONFLICT (workspace_id, user_id) DO UPDATE SET role_id = EXCLUDED.role_id;

    -- shift.manager -> shift_manager
    INSERT INTO workspace_members (workspace_id, user_id, role_id)
    SELECT v_ws_id, 'e80200c2-0000-0000-0000-000000000006', id FROM workspace_roles WHERE workspace_id IS NULL AND name = 'shift_manager'
    ON CONFLICT (workspace_id, user_id) DO UPDATE SET role_id = EXCLUDED.role_id;

    -- team.lead -> team_lead
    INSERT INTO workspace_members (workspace_id, user_id, role_id)
    SELECT v_ws_id, 'e80200c2-0000-0000-0000-000000000007', id FROM workspace_roles WHERE workspace_id IS NULL AND name = 'team_lead'
    ON CONFLICT (workspace_id, user_id) DO UPDATE SET role_id = EXCLUDED.role_id;

    -- employee -> employee
    INSERT INTO workspace_members (workspace_id, user_id, role_id)
    SELECT v_ws_id, 'e80200c2-0000-0000-0000-000000000008', id FROM workspace_roles WHERE workspace_id IS NULL AND name = 'employee'
    ON CONFLICT (workspace_id, user_id) DO UPDATE SET role_id = EXCLUDED.role_id;

    -- =========================================================================
    -- 4. Employee Records (employees)
    -- =========================================================================
    INSERT INTO employees (id, workspace_id, user_id, employee_id_number, full_name, hire_date, original_hire_date, is_active)
    VALUES
        ('f0000000-0000-0000-0000-000000000001', v_ws_id, 'e80200c2-eec3-4fc1-8135-de9f8f264396', 'EMP001', 'System Administrator', CURRENT_DATE - INTERVAL '365 days', CURRENT_DATE - INTERVAL '365 days', true),
        ('f0000000-0000-0000-0000-000000000005', v_ws_id, 'e80200c2-0000-0000-0000-000000000005', 'EMP005', 'HR Manager', CURRENT_DATE - INTERVAL '200 days', CURRENT_DATE - INTERVAL '200 days', true),
        ('f0000000-0000-0000-0000-000000000006', v_ws_id, 'e80200c2-0000-0000-0000-000000000006', 'EMP006', 'Shift Operations Manager', CURRENT_DATE - INTERVAL '180 days', CURRENT_DATE - INTERVAL '180 days', true),
        ('f0000000-0000-0000-0000-000000000007', v_ws_id, 'e80200c2-0000-0000-0000-000000000007', 'EMP007', 'Technical Team Lead', CURRENT_DATE - INTERVAL '150 days', CURRENT_DATE - INTERVAL '150 days', true),
        ('f0000000-0000-0000-0000-000000000008', v_ws_id, 'e80200c2-0000-0000-0000-000000000008', 'EMP008', 'Standard Staff Member', CURRENT_DATE - INTERVAL '120 days', CURRENT_DATE - INTERVAL '120 days', true)
    ON CONFLICT (id) DO UPDATE SET full_name = EXCLUDED.full_name;

    -- =========================================================================
    -- 5. Employments (active employment records)
    -- =========================================================================
    INSERT INTO employments (id, workspace_id, employee_id, employer_id, department_id, status, start_date, job_title, employment_type)
    VALUES
        ('b0000000-0000-0000-0000-000000000001', v_ws_id, 'f0000000-0000-0000-0000-000000000001', v_employer_id, v_dept_exec, 'active', CURRENT_DATE - INTERVAL '365 days', 'Chief Executive Officer', 'full_time'),
        ('b0000000-0000-0000-0000-000000000005', v_ws_id, 'f0000000-0000-0000-0000-000000000005', v_employer_id, v_dept_hr, 'active', CURRENT_DATE - INTERVAL '200 days', 'HR Director', 'full_time'),
        ('b0000000-0000-0000-0000-000000000006', v_ws_id, 'f0000000-0000-0000-0000-000000000006', v_employer_id, v_dept_tech, 'active', CURRENT_DATE - INTERVAL '180 days', 'Shift Planning Lead', 'full_time'),
        ('b0000000-0000-0000-0000-000000000007', v_ws_id, 'f0000000-0000-0000-0000-000000000007', v_employer_id, v_dept_tech, 'active', CURRENT_DATE - INTERVAL '150 days', 'Tech Lead', 'full_time'),
        ('b0000000-0000-0000-0000-000000000008', v_ws_id, 'f0000000-0000-0000-0000-000000000008', v_employer_id, v_dept_tech, 'active', CURRENT_DATE - INTERVAL '120 days', 'Software Engineer', 'full_time')
    ON CONFLICT (id) DO NOTHING;

    -- Update department managers
    UPDATE departments SET manager_id = 'f0000000-0000-0000-0000-000000000005' WHERE id = v_dept_hr;
    UPDATE departments SET manager_id = 'f0000000-0000-0000-0000-000000000007' WHERE id = v_dept_tech;
END $$;
