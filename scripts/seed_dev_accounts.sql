-- =============================================================================
-- Seed documented development accounts in the final single-organization schema.
-- Operational account reference: docs/dev-accounts.md
-- Development fixtures only; do not execute against customer or live databases.
-- =============================================================================

DO $$
DECLARE
    v_pwd_hash text := '$argon2id$v=19$m=65536,t=1,p=4$Q71XRJT8SsPnqVciqiphLA$i37H/ySyg61+aIj0BzIV3RAPSRP09elhC0wocBSTv5Y';
    v_employer_id uuid := 'e0000000-0000-0000-0000-000000000001';
    v_dept_exec uuid := 'd0000000-0000-0000-0000-000000000001';
    v_dept_hr uuid := 'd0000000-0000-0000-0000-000000000002';
    v_dept_tech uuid := 'd0000000-0000-0000-0000-000000000003';
BEGIN
    IF NOT EXISTS (SELECT 1 FROM organization_settings) THEN
        INSERT INTO organization_settings (name) VALUES ('Development Organization');
    END IF;

    INSERT INTO employers (id, name, tax_code)
    VALUES (v_employer_id, 'LeeAttend Enterprise Corp', '0101234567')
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO departments (id, name, description)
    VALUES
        (v_dept_exec, 'Ban Giám Đốc', 'Executive & Platform Management'),
        (v_dept_hr, 'Phòng Nhân Sự', 'Human Resources Management'),
        (v_dept_tech, 'Phòng Kỹ Thuật', 'Engineering & Operations')
    ON CONFLICT (id) DO NOTHING;

    -- Seed role catalog before memberships; standalone bootstrap may not have run yet.
    INSERT INTO roles (id, name, description, color, is_system, scope_type)
    VALUES
        ('a0000000-0000-0000-0000-000000000002', 'admin', 'Administrator with full access', 'rose', true, 'global'),
        ('a0000000-0000-0000-0000-000000000003', 'hr_manager', 'HR Manager with employee and attendance control', 'purple', true, 'global'),
        ('a0000000-0000-0000-0000-000000000004', 'shift_manager', 'Shift Manager for schedule planning', 'blue', true, 'global'),
        ('a0000000-0000-0000-0000-000000000005', 'team_lead', 'Team Lead for daily attendance monitoring', 'green', true, 'scoped'),
        ('a0000000-0000-0000-0000-000000000006', 'employee', 'Standard employee with self-service access', 'gray', true, 'scoped')
    ON CONFLICT (name) DO UPDATE SET description = EXCLUDED.description, color = EXCLUDED.color, is_system = EXCLUDED.is_system, scope_type = EXCLUDED.scope_type;

    -- Reconcile grants left by older development seeds. Role upserts do not
    -- remove permissions that were valid in a previous catalog.
    DELETE FROM role_permissions rp
    USING roles r
    WHERE rp.role_id = r.id
      AND (
          (r.name = 'team_lead' AND rp.permission_name IN (
              'attendance:adjustments:approve',
              'attendance:adjustments:submit',
              'attendance:logs:read',
              'attendance:summaries:read',
              'dashboard:read',
              'employees:read',
              'leave:balances:read',
              'leave:requests:read',
              'leave:requests:submit',
              'overtime:approve',
              'overtime:request:read',
              'rosters:read',
              'shift_swaps:read'
          ))
          OR (r.name IN ('shift_manager', 'team_lead', 'employee') AND rp.permission_name = 'employees:read')
          OR (r.name = 'employee' AND rp.permission_name = 'attendance:adjustments:submit')
      );

    INSERT INTO role_permissions (role_id, permission_name)
    SELECT id, permission_name
    FROM roles
    CROSS JOIN (VALUES ('departments:transfer'), ('employees:rehire')) AS grants(permission_name)
    WHERE name = 'hr_manager'
    ON CONFLICT (role_id, permission_name) DO NOTHING;

    -- Upsert only documented development identities; never delete other users.
    INSERT INTO users (id, email, password_hash, full_name, created_at)
    VALUES
        ('e80200c2-eec3-4fc1-8135-de9f8f264396', 'admin@example.invalid', v_pwd_hash, 'System Administrator', NOW()),
        ('e80200c2-0000-0000-0000-000000000005', 'hr.manager@example.invalid', v_pwd_hash, 'HR Manager', NOW()),
        ('e80200c2-0000-0000-0000-000000000006', 'shift.manager@example.invalid', v_pwd_hash, 'Shift Operations Manager', NOW()),
        ('e80200c2-0000-0000-0000-000000000007', 'team.lead@example.invalid', v_pwd_hash, 'Technical Team Lead', NOW()),
        ('e80200c2-0000-0000-0000-000000000008', 'employee@example.invalid', v_pwd_hash, 'Standard Staff Member', NOW())
    ON CONFLICT (email) DO UPDATE SET password_hash = EXCLUDED.password_hash, full_name = EXCLUDED.full_name;

    INSERT INTO user_roles (user_id, role_id)
    SELECT users.id, roles.id
    FROM (VALUES
        ('admin@example.invalid', 'admin'),
        ('hr.manager@example.invalid', 'hr_manager'),
        ('shift.manager@example.invalid', 'shift_manager'),
        ('team.lead@example.invalid', 'team_lead'),
        ('employee@example.invalid', 'employee')
    ) AS expected(email, role_name)
    JOIN users ON users.email = expected.email
    JOIN roles ON roles.name = expected.role_name
    ON CONFLICT (user_id) DO UPDATE SET role_id = EXCLUDED.role_id;

    INSERT INTO employees (id, user_id, employee_id_number, full_name, hire_date, original_hire_date, is_active)
    VALUES
        ('f0000000-0000-0000-0000-000000000001', 'e80200c2-eec3-4fc1-8135-de9f8f264396', 'EMP001', 'System Administrator', CURRENT_DATE - INTERVAL '365 days', CURRENT_DATE - INTERVAL '365 days', true),
        ('f0000000-0000-0000-0000-000000000005', 'e80200c2-0000-0000-0000-000000000005', 'EMP005', 'HR Manager', CURRENT_DATE - INTERVAL '200 days', CURRENT_DATE - INTERVAL '200 days', true),
        ('f0000000-0000-0000-0000-000000000006', 'e80200c2-0000-0000-0000-000000000006', 'EMP006', 'Shift Operations Manager', CURRENT_DATE - INTERVAL '180 days', CURRENT_DATE - INTERVAL '180 days', true),
        ('f0000000-0000-0000-0000-000000000007', 'e80200c2-0000-0000-0000-000000000007', 'EMP007', 'Technical Team Lead', CURRENT_DATE - INTERVAL '150 days', CURRENT_DATE - INTERVAL '150 days', true),
        ('f0000000-0000-0000-0000-000000000008', 'e80200c2-0000-0000-0000-000000000008', 'EMP008', 'Standard Staff Member', CURRENT_DATE - INTERVAL '120 days', CURRENT_DATE - INTERVAL '120 days', true)
    ON CONFLICT (id) DO UPDATE SET full_name = EXCLUDED.full_name, employee_id_number = EXCLUDED.employee_id_number;

    INSERT INTO employments (id, employee_id, employer_id, department_id, status, start_date, job_title, employment_type)
    VALUES
        ('b0000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', v_employer_id, v_dept_exec, 'active', CURRENT_DATE - INTERVAL '365 days', 'Chief Executive Officer', 'full_time'),
        ('b0000000-0000-0000-0000-000000000005', 'f0000000-0000-0000-0000-000000000005', v_employer_id, v_dept_hr, 'active', CURRENT_DATE - INTERVAL '200 days', 'HR Director', 'full_time'),
        ('b0000000-0000-0000-0000-000000000006', 'f0000000-0000-0000-0000-000000000006', v_employer_id, v_dept_tech, 'active', CURRENT_DATE - INTERVAL '180 days', 'Shift Planning Lead', 'full_time'),
        ('b0000000-0000-0000-0000-000000000007', 'f0000000-0000-0000-0000-000000000007', v_employer_id, v_dept_tech, 'active', CURRENT_DATE - INTERVAL '150 days', 'Tech Lead', 'full_time'),
        ('b0000000-0000-0000-0000-000000000008', 'f0000000-0000-0000-0000-000000000008', v_employer_id, v_dept_tech, 'active', CURRENT_DATE - INTERVAL '120 days', 'Software Engineer', 'full_time')
    ON CONFLICT (id) DO UPDATE SET department_id = EXCLUDED.department_id, job_title = EXCLUDED.job_title;

    -- ESS account fixtures need current tenure rows for tenure-aware profile resolution.
    INSERT INTO employee_tenures (employee_id, employee_number, user_id, start_date)
    SELECT e.id, e.employee_id_number, e.user_id, e.hire_date
    FROM employees e
    WHERE e.user_id IS NOT NULL
      AND e.employee_id_number IN ('EMP001', 'EMP005', 'EMP006', 'EMP007', 'EMP008')
      AND NOT EXISTS (
          SELECT 1
          FROM employee_tenures t
          WHERE t.employee_id = e.id AND t.deleted_at IS NULL AND t.end_date IS NULL
      );

    UPDATE departments SET manager_id = 'f0000000-0000-0000-0000-000000000005' WHERE id = v_dept_hr;
    UPDATE departments SET manager_id = 'f0000000-0000-0000-0000-000000000007' WHERE id = v_dept_tech;

    INSERT INTO attendance_daily_summaries (
        id, employment_id, date,
        actual_work_minutes, standard_minutes, approved_ot_minutes,
        is_weekend, is_holiday, ot_status
    )
    SELECT
        gen_random_uuid(),
        emp.id,
        d.dt::date,
        CASE
            WHEN EXTRACT(DOW FROM d.dt) IN (0, 6) THEN 0
            WHEN EXTRACT(DAY FROM d.dt) IN (5, 20) AND emp.id IN ('b0000000-0000-0000-0000-000000000007', 'b0000000-0000-0000-0000-000000000008') THEN 240
            WHEN EXTRACT(DAY FROM d.dt) = 12 AND emp.id = 'b0000000-0000-0000-0000-000000000005' THEN 0
            WHEN EXTRACT(DAY FROM d.dt) = 18 THEN 600
            ELSE 480
        END AS actual_work_minutes,
        CASE
            WHEN EXTRACT(DOW FROM d.dt) IN (0, 6) THEN 0
            WHEN EXTRACT(DAY FROM d.dt) IN (5, 20) AND emp.id IN ('b0000000-0000-0000-0000-000000000007', 'b0000000-0000-0000-0000-000000000008') THEN 240
            WHEN EXTRACT(DAY FROM d.dt) = 12 AND emp.id = 'b0000000-0000-0000-0000-000000000005' THEN 0
            ELSE 480
        END AS standard_minutes,
        CASE
            WHEN EXTRACT(DOW FROM d.dt) NOT IN (0, 6) AND EXTRACT(DAY FROM d.dt) = 18 THEN 120
            ELSE 0
        END AS approved_ot_minutes,
        EXTRACT(DOW FROM d.dt) IN (0, 6) AS is_weekend,
        false AS is_holiday,
        CASE
            WHEN EXTRACT(DOW FROM d.dt) NOT IN (0, 6) AND EXTRACT(DAY FROM d.dt) = 18 THEN 'approved'::public.ot_status_type
            ELSE 'none'::public.ot_status_type
        END AS ot_status
    FROM employments emp
    CROSS JOIN generate_series(CURRENT_DATE - INTERVAL '60 days', CURRENT_DATE, INTERVAL '1 day') AS d(dt)
    ON CONFLICT (employment_id, date) DO UPDATE SET
        actual_work_minutes = EXCLUDED.actual_work_minutes,
        standard_minutes = EXCLUDED.standard_minutes,
        approved_ot_minutes = EXCLUDED.approved_ot_minutes,
        is_weekend = EXCLUDED.is_weekend,
        ot_status = EXCLUDED.ot_status;
END $$;
