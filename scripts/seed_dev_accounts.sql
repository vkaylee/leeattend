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

    INSERT INTO employees (id, full_name)
    VALUES
        ('f0000000-0000-0000-0000-000000000001', 'System Administrator'),
        ('f0000000-0000-0000-0000-000000000005', 'HR Manager'),
        ('f0000000-0000-0000-0000-000000000006', 'Shift Operations Manager'),
        ('f0000000-0000-0000-0000-000000000007', 'Technical Team Lead'),
        ('f0000000-0000-0000-0000-000000000008', 'Standard Staff Member')
    ON CONFLICT (id) DO UPDATE SET full_name = EXCLUDED.full_name;

    INSERT INTO employee_tenures (id, employee_id, employee_number, user_id, hire_type, start_date)
    VALUES
        ('c0000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', 'EMP001', 'e80200c2-eec3-4fc1-8135-de9f8f264396', 'initial', (CURRENT_DATE - INTERVAL '365 days')::date),
        ('c0000000-0000-0000-0000-000000000005', 'f0000000-0000-0000-0000-000000000005', 'EMP005', 'e80200c2-0000-0000-0000-000000000005', 'initial', (CURRENT_DATE - INTERVAL '200 days')::date),
        ('c0000000-0000-0000-0000-000000000006', 'f0000000-0000-0000-0000-000000000006', 'EMP006', 'e80200c2-0000-0000-0000-000000000006', 'initial', (CURRENT_DATE - INTERVAL '180 days')::date),
        ('c0000000-0000-0000-0000-000000000007', 'f0000000-0000-0000-0000-000000000007', 'EMP007', 'e80200c2-0000-0000-0000-000000000007', 'initial', (CURRENT_DATE - INTERVAL '150 days')::date),
        ('c0000000-0000-0000-0000-000000000008', 'f0000000-0000-0000-0000-000000000008', 'EMP008', 'e80200c2-0000-0000-0000-000000000008', 'initial', (CURRENT_DATE - INTERVAL '120 days')::date)
    ON CONFLICT (id) DO UPDATE SET employee_number = EXCLUDED.employee_number, user_id = EXCLUDED.user_id;

    INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date)
    VALUES
        ('d1000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', v_employer_id, 'c0000000-0000-0000-0000-000000000001', 'active', (CURRENT_DATE - INTERVAL '365 days')::date),
        ('d1000000-0000-0000-0000-000000000005', 'f0000000-0000-0000-0000-000000000005', v_employer_id, 'c0000000-0000-0000-0000-000000000005', 'active', (CURRENT_DATE - INTERVAL '200 days')::date),
        ('d1000000-0000-0000-0000-000000000006', 'f0000000-0000-0000-0000-000000000006', v_employer_id, 'c0000000-0000-0000-0000-000000000006', 'active', (CURRENT_DATE - INTERVAL '180 days')::date),
        ('d1000000-0000-0000-0000-000000000007', 'f0000000-0000-0000-0000-000000000007', v_employer_id, 'c0000000-0000-0000-0000-000000000007', 'active', (CURRENT_DATE - INTERVAL '150 days')::date),
        ('d1000000-0000-0000-0000-000000000008', 'f0000000-0000-0000-0000-000000000008', v_employer_id, 'c0000000-0000-0000-0000-000000000008', 'active', (CURRENT_DATE - INTERVAL '120 days')::date)
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO employments (id, employee_id, employer_id, relationship_id, department_id, status, start_date, job_title, employment_type)
    VALUES
        ('b0000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', v_employer_id, 'd1000000-0000-0000-0000-000000000001', v_dept_exec, 'active', (CURRENT_DATE - INTERVAL '365 days')::date, 'Chief Executive Officer', 'full_time'),
        ('b0000000-0000-0000-0000-000000000005', 'f0000000-0000-0000-0000-000000000005', v_employer_id, 'd1000000-0000-0000-0000-000000000005', v_dept_hr, 'active', (CURRENT_DATE - INTERVAL '200 days')::date, 'HR Director', 'full_time'),
        ('b0000000-0000-0000-0000-000000000006', 'f0000000-0000-0000-0000-000000000006', v_employer_id, 'd1000000-0000-0000-0000-000000000006', v_dept_tech, 'active', (CURRENT_DATE - INTERVAL '180 days')::date, 'Shift Planning Lead', 'full_time'),
        ('b0000000-0000-0000-0000-000000000007', 'f0000000-0000-0000-0000-000000000007', v_employer_id, 'd1000000-0000-0000-0000-000000000007', v_dept_tech, 'active', (CURRENT_DATE - INTERVAL '150 days')::date, 'Tech Lead', 'full_time'),
        ('b0000000-0000-0000-0000-000000000008', 'f0000000-0000-0000-0000-000000000008', v_employer_id, 'd1000000-0000-0000-0000-000000000008', v_dept_tech, 'active', (CURRENT_DATE - INTERVAL '120 days')::date, 'Software Engineer', 'full_time')
    ON CONFLICT (id) DO UPDATE SET department_id = EXCLUDED.department_id, job_title = EXCLUDED.job_title;

    UPDATE departments SET manager_id = 'f0000000-0000-0000-0000-000000000005' WHERE id = v_dept_hr;
    UPDATE departments SET manager_id = 'f0000000-0000-0000-0000-000000000007' WHERE id = v_dept_tech;

    -- Default Work Policy v2 + Published Overtime Policy Version + Memberships
    INSERT INTO overtime_policy_definitions (id, name, description)
    VALUES (
        '60000000-0000-0000-0000-000000000001',
        'Standard Statutory OT Policy',
        'Standard weekday 1.5x, weekend 2.0x, holiday 3.0x, night 22:00-06:00 premium with statutory caps'
    )
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO overtime_policy_versions (
        id, policy_id, version_no, status, row_version,
        daily_cap_minutes, monthly_cap_hours, yearly_cap_hours,
        rules, created_by, published_by, published_at
    )
    VALUES (
        '61000000-0000-0000-0000-000000000001',
        '60000000-0000-0000-0000-000000000001',
        1,
        'published',
        1,
        240,
        40.00,
        200.00,
        '[
            {"priority": 100, "day_type": "holiday", "days_of_week": 127, "time_start": "22:00", "time_end": "06:00", "multiplier": 3.90},
            {"priority": 90,  "day_type": "holiday", "days_of_week": 127, "multiplier": 3.00},
            {"priority": 80,  "day_type": "weekend", "days_of_week": 127, "time_start": "22:00", "time_end": "06:00", "multiplier": 2.70},
            {"priority": 70,  "day_type": "weekend", "days_of_week": 127, "multiplier": 2.00},
            {"priority": 60,  "day_type": "weekday", "days_of_week": 127, "time_start": "22:00", "time_end": "06:00", "multiplier": 2.00},
            {"priority": 50,  "day_type": "weekday", "days_of_week": 127, "multiplier": 1.50}
        ]'::jsonb,
        'e80200c2-eec3-4fc1-8135-de9f8f264396',
        'e80200c2-eec3-4fc1-8135-de9f8f264396',
        NOW()
    )
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO work_policy_groups (id, code, name, description, created_by)
    VALUES (
        '62000000-0000-0000-0000-000000000001',
        'STD-MONTHLY',
        'Standard Office Monthly Policy',
        'Default full-time office work policy (480m/credit, fixed 22 days/month)',
        'e80200c2-eec3-4fc1-8135-de9f8f264396'
    )
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO work_policy_versions (
        id, group_id, version_no, status, row_version,
        effective_from, effective_to,
        created_by, published_by, published_at,
        minutes_per_credit, work_unit_minutes, rounding_policy, early_arrival_strategy,
        late_arrival_grace_minutes, early_ot_enabled, early_ot_qualification_grace_minutes,
        early_ot_max_minutes, break_auto_deduct, break_is_overtime, ot_aggregation_mode,
        working_pattern_id, pay_method, monthly_denominator, monthly_standard_days,
        rate_component_definition_id, paid_leave_credit_mode, overtime_policy_version_id
    )
    VALUES (
        '63000000-0000-0000-0000-000000000001',
        '62000000-0000-0000-0000-000000000001',
        1,
        'published',
        1,
        DATE '2020-01-01',
        NULL,
        'e80200c2-eec3-4fc1-8135-de9f8f264396',
        'e80200c2-eec3-4fc1-8135-de9f8f264396',
        NOW(),
        480, 1, 'none', 'clamp',
        5, false, 0,
        NULL, true, false, 'aggregate',
        NULL, 'monthly', 'fixed', 22.0000,
        NULL, 'standard_day', '61000000-0000-0000-0000-000000000001'
    )
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO work_policy_memberships (
        id, employment_relationship_id, group_id, effective_from, effective_to, created_by
    )
    SELECT
        ('64000000-0000-0000-0000-' || right(replace(er.id::text, '-', ''), 12))::uuid,
        er.id,
        '62000000-0000-0000-0000-000000000001',
        er.start_date,
        CASE WHEN er.end_date IS NOT NULL THEN er.end_date + 1 ELSE NULL END,
        'e80200c2-eec3-4fc1-8135-de9f8f264396'
    FROM employment_relationships er
    WHERE er.id IN (
        'd1000000-0000-0000-0000-000000000001',
        'd1000000-0000-0000-0000-000000000005',
        'd1000000-0000-0000-0000-000000000006',
        'd1000000-0000-0000-0000-000000000007',
        'd1000000-0000-0000-0000-000000000008'
    )
      AND NOT EXISTS (
          SELECT 1
          FROM work_policy_memberships m
          WHERE m.employment_relationship_id = er.id
      )
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO attendance_daily_summaries (
        id, employment_id, date,
        actual_work_minutes, standard_minutes, approved_ot_minutes,
        is_weekend, is_holiday, ot_status, work_policy_version_id
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
        END AS ot_status,
        '63000000-0000-0000-0000-000000000001'::uuid AS work_policy_version_id
    FROM employments emp
    CROSS JOIN generate_series(CURRENT_DATE - INTERVAL '60 days', CURRENT_DATE, INTERVAL '1 day') AS d(dt)
    ON CONFLICT (employment_id, date) DO UPDATE SET
        actual_work_minutes = EXCLUDED.actual_work_minutes,
        standard_minutes = EXCLUDED.standard_minutes,
        approved_ot_minutes = EXCLUDED.approved_ot_minutes,
        is_weekend = EXCLUDED.is_weekend,
        ot_status = EXCLUDED.ot_status,
        work_policy_version_id = EXCLUDED.work_policy_version_id;
END $$;
