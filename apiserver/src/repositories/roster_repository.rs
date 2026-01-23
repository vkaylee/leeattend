use crate::models::{EffectiveSchedule, EmployeeSchedule, ShiftRoster};
use chrono::NaiveDate;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

// ... (existing imports)

pub async fn get_workspace_schedules(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    employee_ids: Option<&Vec<Uuid>>,
) -> Result<Vec<EmployeeSchedule>, sqlx::Error> {
    // 1. Fetch Employees
    let employees = sqlx::query(
        "SELECT id, full_name, employee_id_number, tags FROM employees WHERE workspace_id = $1 AND is_active = true AND ($2::uuid[] IS NULL OR id = ANY($2))",
    )
    .bind(workspace_id)
    .bind(employee_ids)
    .fetch_all(pool)
    .await?;

    let employees: Vec<_> = employees.into_iter().map(|row| {
        use sqlx::Row;
        let id: Uuid = row.get(0);
        let full_name: String = row.get(1);
        let employee_id_number: String = row.get(2);
        let tags: Option<Vec<String>> = row.get(3);
        (id, full_name, employee_id_number, tags)
    }).collect();

    // 2. Fetch all shifts (for details)
    #[allow(clippy::type_complexity)]
    let shifts_map: HashMap<Uuid, (String, chrono::NaiveTime, chrono::NaiveTime, i32, Option<chrono::NaiveTime>, Option<chrono::NaiveTime>, bool, i32, bool)> = sqlx::query!(
        "SELECT id, name, start_time, end_time, break_duration_minutes, break_start, break_end, is_break_auto_deduct, early_punch_in_grace_period_minutes, allow_early_ot FROM shifts WHERE workspace_id = $1",
        workspace_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|s| (s.id, (s.name, s.start_time, s.end_time, s.break_duration_minutes, s.break_start, s.break_end, s.is_break_auto_deduct, s.early_punch_in_grace_period_minutes, s.allow_early_ot)))
    .collect();

    // 3. Fetch Rosters in range
    let rosters = sqlx::query_as::<_, ShiftRoster>(
        r#"
        SELECT sr.* 
        FROM shift_rosters sr
        JOIN employees e ON sr.employee_id = e.id
        WHERE e.workspace_id = $1 
          AND sr.date >= $2 
          AND sr.date <= $3
        "#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    // Group rosters by (employee_id, date)
    let mut roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = HashMap::new();
    for r in rosters {
        roster_map.insert((r.employee_id, r.date), r.shift_id);
    }

    // 4. Fetch Fixed Assignments (overlapping)
    let fixed_assignments = sqlx::query!(
        r#"
        SELECT employee_id, shift_id, start_date, end_date, created_at
        FROM employee_shifts
        WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)
          AND start_date <= $2
          AND (end_date IS NULL OR end_date >= $3)
        ORDER BY created_at DESC
        "#,
        workspace_id,
        end_date,
        start_date
    )
    .fetch_all(pool)
    .await?;

    // 5. Fetch Pattern Assignments (overlapping)
    let pattern_assignments = sqlx::query!(
        r#"
        SELECT ewp.employee_id, ewp.pattern_id, ewp.start_date, ewp.end_date, ewp.created_at,
               wpd.day_of_week, wpd.shift_id as pattern_shift_id
        FROM employee_working_patterns ewp
        JOIN working_pattern_details wpd ON ewp.pattern_id = wpd.pattern_id
        WHERE ewp.workspace_id = $1
          AND ewp.start_date <= $2
          AND (ewp.end_date IS NULL OR ewp.end_date >= $3)
        ORDER BY ewp.created_at DESC
        "#,
        workspace_id,
        end_date,
        start_date
    )
    .fetch_all(pool)
    .await?;

    // 6. Fetch Department Patterns & History
    let dept_patterns = sqlx::query!(
        r#"
        SELECT edh.employee_id, d.default_working_pattern_id, edh.effective_date,
               wpd.day_of_week, wpd.shift_id as pattern_shift_id
        FROM employee_department_history edh
        JOIN departments d ON edh.department_id = d.id
        JOIN working_pattern_details wpd ON d.default_working_pattern_id = wpd.pattern_id
        WHERE d.workspace_id = $1
          AND edh.effective_date <= $2
        ORDER BY edh.effective_date DESC
        "#,
        workspace_id,
        end_date
    )
    .fetch_all(pool)
    .await?;

    // 7. Fetch Department Default Shifts
    let dept_fixed_shifts = sqlx::query!(
        r#"
        SELECT edh.employee_id, dds.shift_id, edh.effective_date
        FROM employee_department_history edh
        JOIN department_default_shifts dds ON edh.department_id = dds.department_id
        WHERE edh.effective_date <= $1
        ORDER BY edh.effective_date DESC
        "#,
        end_date
    )
    .fetch_all(pool)
    .await?;

    // 8. Fetch Dynamic Assignment Rules
    let active_rules = crate::repositories::assignment_rule_repository::AssignmentRuleRepository::get_active_rules(pool, workspace_id).await?;

    // 9. Fetch Pattern Details for Rules
    let rule_patterns_details = sqlx::query!(
        "SELECT pattern_id, day_of_week, shift_id FROM working_pattern_details WHERE pattern_id IN (SELECT pattern_id FROM assignment_rules WHERE workspace_id = $1 AND is_active = true)",
        workspace_id
    )
    .fetch_all(pool)
    .await?;

    // 10. Fetch Employee Department History for context
    let all_dept_history = sqlx::query!(
        "SELECT employee_id, department_id, effective_date FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1) AND effective_date <= $2 ORDER BY effective_date DESC",
        workspace_id,
        end_date
    )
    .fetch_all(pool)
    .await?;

    // Let's iterate employees to build result
    let mut result = Vec::new();

    for (emp_id, emp_full_name, emp_id_number, emp_tags) in employees {
        let mut daily_schedules = Vec::new();
        let mut curr_date = start_date;

        while curr_date <= end_date {
            let mut effective_shift_id = None;
            let source: String;
            let mut traces = Vec::new();

            // P1: Roster
            if let Some(shift_opt) = roster_map.get(&(emp_id, curr_date)) {
                effective_shift_id = *shift_opt; // Can be None (Day Off)
                source = "roster".to_string();
                traces.push(crate::models::ScheduleTrace {
                    rule_id: None,
                    rule_name: "Manual Roster".to_string(),
                    priority: 100,
                    status: "applied".to_string(),
                    reason: Some("Directly assigned via roster".to_string()),
                });
            } else {
                let dow = chrono::Datelike::weekday(&curr_date).num_days_from_sunday() as i32;

                // P2: Individual Pattern
                let pattern = pattern_assignments.iter().find(|a| {
                    a.employee_id == emp_id
                        && a.start_date <= curr_date
                        && a.end_date.is_none_or(|ed| ed >= curr_date)
                        && a.day_of_week == dow
                });

                if let Some(p) = pattern {
                    effective_shift_id = p.pattern_shift_id;
                    source = "pattern".to_string();
                    traces.push(crate::models::ScheduleTrace {
                        rule_id: Some(p.pattern_id),
                        rule_name: "Individual Pattern".to_string(),
                        priority: 80,
                        status: "applied".to_string(),
                        reason: Some("Assigned via individual working pattern".to_string()),
                    });
                } else {
                    // P3: Dynamic Rules
                    let emp_dept_id = all_dept_history
                        .iter()
                        .find(|h| h.employee_id == emp_id && h.effective_date <= curr_date)
                        .map(|h| h.department_id);
                    let match_ctx = crate::services::rule_evaluator::EmployeeMatchContext {
                        department_id: emp_dept_id,
                        employee_id_number: emp_id_number.clone(),
                        tags: emp_tags.clone().unwrap_or_default(),
                    };

                    let (matched_rule, rule_traces) =
                        crate::services::rule_evaluator::RuleEvaluator::resolve_with_trace(
                            &match_ctx,
                            &active_rules,
                        );
                    traces.extend(rule_traces);

                    if let Some(rule) = matched_rule {
                        effective_shift_id = rule_patterns_details
                            .iter()
                            .find(|d| d.pattern_id == rule.pattern_id && d.day_of_week == dow)
                            .and_then(|d| d.shift_id);
                        source = "assignment_rule".to_string();
                    } else {
                        // P4: Department Pattern
                        let dept_p = dept_patterns.iter().find(|a| {
                            a.employee_id == emp_id
                                && a.effective_date <= curr_date
                                && a.day_of_week == dow
                        });

                        if let Some(dp) = dept_p {
                            effective_shift_id = dp.pattern_shift_id;
                            source = "department_pattern".to_string();
                            traces.push(crate::models::ScheduleTrace {
                                rule_id: dp.default_working_pattern_id,
                                rule_name: "Department Pattern".to_string(),
                                priority: 40,
                                status: "applied".to_string(),
                                reason: Some("Inherited from department default pattern".to_string()),
                            });
                        } else {
                            // P5: Fixed (Individual)
                            let fixed = fixed_assignments.iter().find(|a| {
                                a.employee_id == emp_id
                                    && a.start_date <= curr_date
                                    && a.end_date.is_none_or(|ed| ed >= curr_date)
                            });

                            if let Some(f) = fixed {
                                effective_shift_id = Some(f.shift_id);
                                source = "fixed".to_string();
                                traces.push(crate::models::ScheduleTrace {
                                    rule_id: None,
                                    rule_name: "Fixed Assignment".to_string(),
                                    priority: 20,
                                    status: "applied".to_string(),
                                    reason: Some("Directly assigned shift".to_string()),
                                });
                            } else {
                                // P6: Department Fixed
                                let dept_f = dept_fixed_shifts.iter().find(|a| {
                                    a.employee_id == emp_id && a.effective_date <= curr_date
                                });

                                if let Some(df) = dept_f {
                                    effective_shift_id = Some(df.shift_id);
                                    source = "department_fixed".to_string();
                                    traces.push(crate::models::ScheduleTrace {
                                        rule_id: None,
                                        rule_name: "Department Fixed".to_string(),
                                        priority: 10,
                                        status: "applied".to_string(),
                                        reason: Some("Inherited from department default shift".to_string()),
                                    });
                                } else {
                                    source = "none".to_string();
                                }
                            }
                        }
                    }
                }
            }

            let mut sched = EffectiveSchedule {
                date: curr_date,
                shift_id: effective_shift_id,
                source,
                shift_name: None,
                start_time: None,
                end_time: None,
                break_duration_minutes: None,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: None,
                early_punch_in_grace_period_minutes: None,
                allow_early_ot: None,
                traces,
            };

            if let Some(sid) = effective_shift_id {
                if let Some((
                    name,
                    start,
                    end,
                    duration,
                    b_start,
                    b_end,
                    auto_deduct,
                    grace,
                    allow_ot,
                )) = shifts_map.get(&sid)
                {
                    sched.shift_name = Some(name.clone());
                    sched.start_time = Some(*start);
                    sched.end_time = Some(*end);
                    sched.break_duration_minutes = Some(*duration);
                    sched.break_start = *b_start;
                    sched.break_end = *b_end;
                    sched.is_break_auto_deduct = Some(*auto_deduct);
                    sched.early_punch_in_grace_period_minutes = Some(*grace);
                    sched.allow_early_ot = Some(*allow_ot);
                }
            }

            daily_schedules.push(sched);
            if let Some(next_date) = curr_date.succ_opt() {
                curr_date = next_date;
            } else {
                break;
            }
        }

        result.push(EmployeeSchedule {
            employee_id: emp_id,
            full_name: emp_full_name,
            schedules: daily_schedules,
        });
    }

    Ok(result)
}

pub async fn get_assignment_history(
    pool: &PgPool,
    workspace_id: Uuid,
    limit: i64,
) -> Result<Vec<crate::models::AssignmentHistory>, sqlx::Error> {
    sqlx::query_as::<_, crate::models::AssignmentHistory>(
        r#"
        SELECT 
            id,
            employee_id,
            (SELECT full_name FROM employees WHERE id = history.employee_id) as employee_name,
            assignment_type,
            shift_name,
            date_range,
            created_at
        FROM (
            -- Fixed Assignments
            SELECT 
                es.id,
                es.employee_id,
                'fixed'::assignment_history_type as assignment_type,
                s.name as shift_name,
                TO_CHAR(es.start_date, 'YYYY-MM-DD') || 
                CASE WHEN es.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(es.end_date, 'YYYY-MM-DD') END as date_range,
                es.created_at
            FROM employee_shifts es
            JOIN shifts s ON es.shift_id = s.id
            JOIN employees e ON es.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Pattern Assignments
            SELECT 
                ewp.id,
                ewp.employee_id,
                'pattern'::assignment_history_type as assignment_type,
                wp.name as shift_name,
                TO_CHAR(ewp.start_date, 'YYYY-MM-DD') || 
                CASE WHEN ewp.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(ewp.end_date, 'YYYY-MM-DD') END as date_range,
                ewp.created_at
            FROM employee_working_patterns ewp
            JOIN working_patterns wp ON ewp.pattern_id = wp.id
            JOIN employees e ON ewp.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Roster Assignments
            SELECT 
                sr.id,
                sr.employee_id,
                'roster'::assignment_history_type as assignment_type,
                CASE WHEN sr.shift_id IS NULL THEN 'OFF' ELSE s.name END as shift_name,
                TO_CHAR(sr.date, 'YYYY-MM-DD') as date_range,
                sr.created_at
            FROM shift_rosters sr
            LEFT JOIN shifts s ON sr.shift_id = s.id
            JOIN employees e ON sr.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Department Patterns
            SELECT 
                edh.id,
                edh.employee_id,
                'department_pattern'::assignment_history_type as assignment_type,
                wp.name as shift_name,
                TO_CHAR(edh.effective_date, 'YYYY-MM-DD') || ' -> ∞' as date_range,
                edh.created_at
            FROM employee_department_history edh
            JOIN departments d ON edh.department_id = d.id
            JOIN working_patterns wp ON d.default_working_pattern_id = wp.id
            JOIN employees e ON edh.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Department Fixed
            SELECT 
                edh.id,
                edh.employee_id,
                'department_fixed'::assignment_history_type as assignment_type,
                s.name as shift_name,
                TO_CHAR(edh.effective_date, 'YYYY-MM-DD') || ' -> ∞' as date_range,
                edh.created_at
            FROM employee_department_history edh
            JOIN department_default_shifts dds ON edh.department_id = dds.department_id
            JOIN shifts s ON dds.shift_id = s.id
            JOIN employees e ON edh.employee_id = e.id
            WHERE e.workspace_id = $1
        ) as history
        ORDER BY created_at DESC
        LIMIT $2
        "#
    )
    .bind(workspace_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_employee_rosters(
    pool: &PgPool,
    employee_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<ShiftRoster>, sqlx::Error> {
    sqlx::query_as::<_, ShiftRoster>(
        "SELECT * FROM shift_rosters WHERE employee_id = $1 AND date >= $2 AND date <= $3 ORDER BY date ASC"
    )
    .bind(employee_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
}

pub async fn upsert_roster(
    pool: &PgPool,
    roster: &ShiftRoster,
) -> Result<ShiftRoster, sqlx::Error> {
    sqlx::query_as::<_, ShiftRoster>(
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status, notes)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (employee_id, date) 
         DO UPDATE SET shift_id = EXCLUDED.shift_id, status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = NOW()
         RETURNING *",
    )
    .bind(roster.id)
    .bind(roster.employee_id)
    .bind(roster.shift_id)
    .bind(roster.date)
    .bind(roster.status.clone())
    .bind(roster.notes.clone())
    .fetch_one(pool)
    .await
}

pub async fn bulk_upsert_roster(
    pool: &sqlx::PgPool,
    employee_id: Uuid,
    items: Vec<crate::models::RosterItem>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    for item in items {
        sqlx::query(
            "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status, notes)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (employee_id, date) 
             DO UPDATE SET shift_id = EXCLUDED.shift_id, status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = NOW()",
        )
        .bind(Uuid::new_v4())
        .bind(employee_id)
        .bind(item.shift_id)
        .bind(item.date)
        .bind("confirmed")
        .bind(item.notes)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn get_effective_shift(
    pool: &PgPool,
    employee_id: Uuid,
    date: NaiveDate,
) -> Result<(Option<Uuid>, Vec<crate::models::ScheduleTrace>), sqlx::Error> {
    let mut traces = Vec::new();

    // Priority 1: Shift Roster
    let roster_shift = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT shift_id FROM shift_rosters WHERE employee_id = $1 AND date = $2",
    )
    .bind(employee_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    if let Some(shift) = roster_shift {
        traces.push(crate::models::ScheduleTrace {
            rule_id: None,
            rule_name: "Manual Roster".to_string(),
            priority: 100,
            status: "applied".to_string(),
            reason: Some("Directly assigned via roster".to_string()),
        });
        return Ok((shift, traces)); // Returns the shift ID or None (day off)
    }

    // Priority 2: Working Pattern (Resolved based on day of week)
    let day_of_week = chrono::Datelike::weekday(&date).num_days_from_sunday() as i32;
    let pattern_info = sqlx::query!(
        r#"
        SELECT wpd.shift_id, ewp.pattern_id
        FROM working_pattern_details wpd
        JOIN employee_working_patterns ewp ON ewp.pattern_id = wpd.pattern_id
        WHERE ewp.employee_id = $1 
          AND ewp.start_date <= $2 
          AND (ewp.end_date IS NULL OR ewp.end_date >= $2)
          AND wpd.day_of_week = $3
        ORDER BY ewp.created_at DESC
        LIMIT 1
        "#,
        employee_id,
        date,
        day_of_week
    )
    .fetch_optional(pool)
    .await?;

    if let Some(p) = pattern_info {
        traces.push(crate::models::ScheduleTrace {
            rule_id: Some(p.pattern_id),
            rule_name: "Individual Pattern".to_string(),
            priority: 80,
            status: "applied".to_string(),
            reason: Some("Assigned via individual working pattern".to_string()),
        });
        return Ok((p.shift_id, traces));
    }

    // Priority 3: Dynamic Assignment Rules
    let employee_full = sqlx::query!(
        r#"
        SELECT e.workspace_id, e.employee_id_number, e.tags, edh.department_id as "department_id?"
        FROM employees e
        LEFT JOIN employee_department_history edh ON edh.employee_id = e.id AND edh.effective_date <= $2
        WHERE e.id = $1
        ORDER BY edh.effective_date DESC
        LIMIT 1
        "#,
        employee_id,
        date
    )
    .fetch_optional(pool)
    .await?;

    if let Some(info) = employee_full {
        let rules = crate::repositories::assignment_rule_repository::AssignmentRuleRepository::get_active_rules(pool, info.workspace_id).await.unwrap_or_default();

        if !rules.is_empty() {
            let match_ctx = crate::services::rule_evaluator::EmployeeMatchContext {
                department_id: info.department_id,
                employee_id_number: info.employee_id_number,
                tags: info.tags.unwrap_or_default(),
            };

            let (matched_rule, rule_traces) =
                crate::services::rule_evaluator::RuleEvaluator::resolve_with_trace(
                    &match_ctx,
                    &rules,
                );
            traces.extend(rule_traces);

            if let Some(rule) = matched_rule {
                // Resolve pattern for the day of week
                let day_of_week = chrono::Datelike::weekday(&date).num_days_from_sunday() as i32;
                let rule_shift = sqlx::query_scalar!(
                    "SELECT shift_id FROM working_pattern_details WHERE pattern_id = $1 AND day_of_week = $2",
                    rule.pattern_id,
                    day_of_week
                )
                .fetch_optional(pool)
                .await?;

                if let Some(shift_opt) = rule_shift {
                    return Ok((shift_opt, traces));
                }
            }
        }
    }

    // Priority 4: Department Working Pattern (Inheritance)
    let dept_pattern_info = sqlx::query!(
        r#"
        SELECT wpd.shift_id, d.default_working_pattern_id
        FROM working_pattern_details wpd
        JOIN departments d ON d.default_working_pattern_id = wpd.pattern_id
        JOIN employee_department_history edh ON edh.department_id = d.id
        WHERE edh.employee_id = $1 
          AND edh.effective_date <= $2
          AND wpd.day_of_week = $3
        ORDER BY edh.effective_date DESC
        LIMIT 1
        "#,
        employee_id,
        date,
        day_of_week
    )
    .fetch_optional(pool)
    .await?;

    if let Some(dp) = dept_pattern_info {
        traces.push(crate::models::ScheduleTrace {
            rule_id: dp.default_working_pattern_id,
            rule_name: "Department Pattern".to_string(),
            priority: 40,
            status: "applied".to_string(),
            reason: Some("Inherited from department default pattern".to_string()),
        });
        return Ok((dp.shift_id, traces));
    }

    // Priority 4: Employee Default Shift (Legacy Fixed Assignment)
    let employee_shift = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT shift_id 
        FROM employee_shifts 
        WHERE employee_id = $1 
          AND start_date <= $2 
          AND (end_date IS NULL OR end_date >= $2)
        ORDER BY created_at DESC 
        LIMIT 1
        "#,
    )
    .bind(employee_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    if let Some(shift) = employee_shift {
        traces.push(crate::models::ScheduleTrace {
            rule_id: None,
            rule_name: "Fixed Assignment".to_string(),
            priority: 20,
            status: "applied".to_string(),
            reason: Some("Directly assigned shift".to_string()),
        });
        return Ok((Some(shift), traces));
    }

    // Priority 5: Department Default Shift
    let dept_shift = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT dds.shift_id 
        FROM department_default_shifts dds
        JOIN employee_department_history edh ON edh.department_id = dds.department_id
        WHERE edh.employee_id = $1 AND edh.effective_date <= $2
        ORDER BY edh.effective_date DESC LIMIT 1
        "#,
    )
    .bind(employee_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    if let Some(shift) = dept_shift {
        traces.push(crate::models::ScheduleTrace {
            rule_id: None,
            rule_name: "Department Fixed".to_string(),
            priority: 10,
            status: "applied".to_string(),
            reason: Some("Inherited from department default shift".to_string()),
        });
        return Ok((Some(shift), traces));
    }

    Ok((None, traces))
}

pub async fn delete_roster(
    pool: &PgPool,
    employee_id: Uuid,
    date: NaiveDate,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM shift_rosters WHERE employee_id = $1 AND date = $2")
        .bind(employee_id)
        .bind(date)
        .execute(pool)
        .await?;

    Ok(())
}
