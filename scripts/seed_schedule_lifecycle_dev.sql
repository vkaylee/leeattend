-- =============================================================================
-- Idempotent dev fixture for Schedule Lifecycle & Attendance Closure walkthrough
-- Target employer: LeeAttend Enterprise Corp (e0000000-0000-0000-0000-000000000001)
-- Target month:    2026-11 (30 days; 5 active employments = 150 employment-days)
--
-- Leaves EMP008 (b0000000-0000-0000-0000-000000000008) week 2026-11-23..2026-11-29
-- unpublished (with employee_shifts ready) so browser click-through executes:
--   1. /shifts (admin/shift_manager): Preview -> Publish EMP008 (completes 150/150)
--   2. /shifts: Evidence -> Replan -> Confirm replan on 2026-11-25 (r1 superseded, r2 published)
--   3. /reports (hr_manager): Close 2026-11 -> Closed (#1, 150 days, snapshot_hash)
--   4. /reports (hr_manager): Reopen -> 403 error.self_reopen_forbidden
--   5. /reports (admin): Reopen -> 200 OK (independent actor)
--
-- Development fixtures only; do not execute against customer or live databases.
-- =============================================================================

DO $$
DECLARE
    v_employer_id  uuid := 'e0000000-0000-0000-0000-000000000001';
    v_admin_id     uuid := 'e80200c2-eec3-4fc1-8135-de9f8f264396';
    v_emp_walk     uuid := 'b0000000-0000-0000-0000-000000000008';
    v_template_id  uuid := '50000000-0000-0000-0000-000000000001';
    v_version_id   uuid := '51000000-0000-0000-0000-000000000001';
    v_interval_id  uuid := '52000000-0000-0000-0000-000000000001';
    v_batch_id     uuid := '53000000-0000-0000-0000-000000000001';
    v_payroll_id   uuid := '54000000-0000-0000-0000-000000000001';
    v_tz           text := 'Asia/Ho_Chi_Minh';
    v_month_start  date := DATE '2026-11-01';
    v_month_end    date := DATE '2026-11-30';
    v_week_start   date := DATE '2026-11-23';
    v_week_end     date := DATE '2026-11-29';
    v_emp          record;
    v_day          date;
    v_rev_id       uuid;
    v_start_at     timestamptz;
    v_end_at       timestamptz;
    v_breaks_json  jsonb;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM employers WHERE id = v_employer_id AND deleted_at IS NULL) THEN
        RAISE EXCEPTION 'Dev employer % not found; run scripts/seed_dev_accounts.sql first', v_employer_id;
    END IF;

    -- 1. Standard day shift template, version, and default interval (08:00-17:00, 60m break)
    IF NOT EXISTS (SELECT 1 FROM shift_templates WHERE id = v_template_id) THEN
        INSERT INTO shift_templates (id, name)
        VALUES (v_template_id, 'Standard Day Shift (08:00-17:00)');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM shift_versions WHERE id = v_version_id) THEN
        INSERT INTO shift_versions (
            id, template_id, version_number, name, report_symbol, employer_id,
            effective_from, change_type, change_reason, created_by
        )
        VALUES (
            v_version_id, v_template_id, 1, 'Standard Day Shift (08:00-17:00)', 'D', v_employer_id,
            DATE '2026-01-01', 'create', 'dev schedule lifecycle seed', v_admin_id
        );
    END IF;

    IF EXISTS (SELECT 1 FROM shift_templates WHERE id = v_template_id AND current_version_id IS DISTINCT FROM v_version_id) THEN
        UPDATE shift_templates
        SET current_version_id = v_version_id
        WHERE id = v_template_id;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM shift_default_intervals WHERE id = v_interval_id) THEN
        INSERT INTO shift_default_intervals (
            id, version_id, sort_order, interval_name, start_time, end_time,
            break_duration_minutes, break_start, break_end, break_punch_mode
        )
        VALUES (
            v_interval_id, v_version_id, 0, 'Day Interval', TIME '08:00:00', TIME '17:00:00',
            60, TIME '12:00:00', TIME '13:00:00', 'none'
        );
    END IF;

    -- 2. Open payroll period for 2026-11 (required by AttendanceClosure)
    INSERT INTO payroll_periods (id, employer_id, period_year, period_month, status, version)
    VALUES (v_payroll_id, v_employer_id, 2026, 11, 'open', 1)
    ON CONFLICT (employer_id, period_year, period_month) DO NOTHING;

    -- 3. Fixed shift assignment for all active employments across 2026-11 so Preview/Publish resolves shifts
    FOR v_emp IN
        SELECT id
        FROM employment_segments
        WHERE employer_id = v_employer_id
          AND deleted_at IS NULL
          AND start_date <= v_month_end
          AND (end_date IS NULL OR end_date >= v_month_start)
        ORDER BY id
    LOOP
        IF NOT EXISTS (
            SELECT 1
            FROM employee_shifts
            WHERE employment_id = v_emp.id
              AND template_id = v_template_id
              AND start_date <= v_month_start
              AND (end_date IS NULL OR end_date >= v_month_end)
        ) THEN
            INSERT INTO employee_shifts (id, employment_id, template_id, start_date, end_date)
            VALUES (gen_random_uuid(), v_emp.id, v_template_id, v_month_start, v_month_end);
        END IF;
    END LOOP;

    -- 4. Seed publication batch for baseline official coverage
    INSERT INTO schedule_publication_batches (
        id, idempotency_key, request_hash, mode, status, requested_by, operation,
        response_snapshot, actor_type, actor_name, reason, timezone, range_start, range_end,
        published_at, affected_days
    )
    VALUES (
        v_batch_id,
        'dev-seed-schedule-lifecycle-2026-11',
        repeat('1', 64),
        'rotating',
        'published',
        v_admin_id,
        'publish',
        '{"seeded":true}'::jsonb,
        'human',
        NULL,
        'dev schedule lifecycle baseline 2026-11',
        v_tz,
        v_month_start,
        v_month_end,
        CURRENT_TIMESTAMP,
        143
    )
    ON CONFLICT (id) DO NOTHING;

    -- 5. Official published revisions + work segments (skip EMP008 current week 2026-11-23..2026-11-29)
    PERFORM set_config('schedule.lifecycle_publication', 'on', true);

    FOR v_emp IN
        SELECT id, start_date, end_date
        FROM employment_segments
        WHERE employer_id = v_employer_id
          AND deleted_at IS NULL
          AND start_date <= v_month_end
          AND (end_date IS NULL OR end_date >= v_month_start)
        ORDER BY id
    LOOP
        FOR v_day IN
            SELECT d::date
            FROM generate_series(v_month_start, v_month_end, interval '1 day') AS d
        LOOP
            CONTINUE WHEN v_day < v_emp.start_date OR (v_emp.end_date IS NOT NULL AND v_day > v_emp.end_date);
            CONTINUE WHEN v_emp.id = v_emp_walk AND v_day BETWEEN v_week_start AND v_week_end;
            CONTINUE WHEN EXISTS (
                SELECT 1
                FROM schedule_day_revisions r
                WHERE r.employment_id = v_emp.id
                  AND r.business_date = v_day
                  AND r.state IN ('published', 'explicit_off')
            );

            v_rev_id := gen_random_uuid();
            v_start_at := (v_day::timestamp + TIME '08:00:00') AT TIME ZONE v_tz;
            v_end_at := (v_day::timestamp + TIME '17:00:00') AT TIME ZONE v_tz;
            v_breaks_json := jsonb_build_array(
                jsonb_build_object(
                    'start', '12:00:00',
                    'end', '13:00:00',
                    'start_at', ((v_day::timestamp + TIME '12:00:00') AT TIME ZONE v_tz),
                    'end_at', ((v_day::timestamp + TIME '13:00:00') AT TIME ZONE v_tz),
                    'duration_minutes', 60,
                    'auto_deduct', true
                )
            );

            INSERT INTO schedule_day_revisions (
                id, employment_id, employer_id, business_date, timezone, revision_number,
                state, publication_batch_id, source, source_snapshot, resolver_version,
                reason, published_by, published_actor_type, published_actor_name, published_at
            )
            VALUES (
                v_rev_id, v_emp.id, v_employer_id, v_day, v_tz, 1,
                'draft', v_batch_id, 'fixed',
                jsonb_build_object('template_id', v_template_id, 'version_id', v_version_id),
                'schedule-resolver-v1',
                'dev schedule lifecycle baseline 2026-11',
                v_admin_id, 'human', NULL, NULL
            );

            INSERT INTO work_segments (
                employment_id, date, interval_id, sort_order, scheduled_start, scheduled_end,
                segment_type, segment_status, compensation_rate, gap_compensation_before,
                gap_compensation_rate_before, source, source_detail, generation_metadata,
                template_id, schedule_revision_id, selected_shift_version_id,
                scheduled_start_at, scheduled_end_at, breaks_snapshot,
                start_day_offset, end_day_offset
            )
            VALUES (
                v_emp.id, v_day, v_interval_id, 0, TIME '08:00:00', TIME '17:00:00',
                'work', 'scheduled', 1.00, 'unpaid',
                0.00, 'fixed', '{}'::jsonb, '{"seeded":true}'::jsonb,
                v_template_id, v_rev_id, v_version_id,
                v_start_at, v_end_at, v_breaks_json,
                0, 0
            );

            UPDATE schedule_day_revisions
            SET state = 'published', published_at = CURRENT_TIMESTAMP
            WHERE id = v_rev_id;
        END LOOP;
    END LOOP;

    -- 6. Enable schedule_lifecycle flag and activate employer for 2026-11 official reads
    INSERT INTO feature_flags (flag_key, enabled, description)
    VALUES ('schedule_lifecycle', true, 'Enable official schedule lifecycle reads and mutations in dev')
    ON CONFLICT (flag_key) DO UPDATE SET enabled = true, updated_at = CURRENT_TIMESTAMP;

    INSERT INTO config_entries (
        id, scope_type, scope_id, config_key, value_bool,
        effective_from, effective_to, created_by
    )
    VALUES (
        gen_random_uuid(), 'employer', v_employer_id, 'schedule.lifecycle.enabled', true,
        v_month_start, v_month_end, v_admin_id
    )
    ON CONFLICT (scope_type, scope_id, config_key, effective_from)
    DO UPDATE SET value_bool = true, effective_to = EXCLUDED.effective_to, updated_at = CURRENT_TIMESTAMP;
END $$;
