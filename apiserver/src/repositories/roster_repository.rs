use crate::cache::RosterData;
use crate::models::{EffectiveSchedule, EmployeeSchedule, ScheduleTrace, ShiftRoster};
use crate::repositories::holiday_repository::HolidayRepository;
use chrono::NaiveDate;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Resolve the effective shift for a single employee on a single date using
/// pre-loaded [`RosterData`] and a roster override map.
///
/// Priority chain (highest wins):
///   P1 Roster → P2 Individual Pattern → P3 Dynamic Rules →
///   P4 Department Pattern → P5 Fixed Assignment → P6 Department Fixed
///
/// Returns `(shift_id, source_label, traces)`.
///
/// This is a **pure function** — no DB access — making it cheap to call in a
/// tight loop and easy to unit-test.
pub fn resolve_effective_shift_from_data(
    roster_data: &RosterData,
    roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>>,
    employee_id: Uuid,
    employee_id_number: &str,
    employee_tags: Option<&Vec<String>>,
    date: NaiveDate,
) -> (Option<Uuid>, String, Vec<ScheduleTrace>) {
    let mut traces = Vec::new();

    // P1: Roster override
    if let Some(shift_opt) = roster_map.get(&(employee_id, date)) {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Manual Roster".to_string(),
            priority: 100,
            status: "applied".to_string(),
            reason: Some("Directly assigned via roster".to_string()),
        });
        return (*shift_opt, "roster".to_string(), traces);
    }

    let dow = chrono::Datelike::weekday(&date).num_days_from_sunday() as i32;

    // P2: Individual Working Pattern
    let pattern = roster_data
        .pattern_assignments
        .get(&employee_id)
        .and_then(|v| {
            v.iter().find(|a| {
                a.start_date <= date
                    && a.end_date.is_none_or(|ed| ed >= date)
                    && a.day_of_week == dow
            })
        });

    if let Some(p) = pattern {
        traces.push(ScheduleTrace {
            rule_id: Some(p.pattern_id),
            rule_name: "Individual Pattern".to_string(),
            priority: 80,
            status: "applied".to_string(),
            reason: Some("Assigned via individual working pattern".to_string()),
        });
        return (p.shift_id, "pattern".to_string(), traces);
    }

    // P3: Dynamic Assignment Rules
    let emp_dept_id = roster_data
        .dept_history
        .get(&employee_id)
        .and_then(|v| v.iter().find(|h| h.effective_date <= date))
        .map(|h| h.department_id);

    let match_ctx = crate::services::rule_evaluator::EmployeeMatchContext {
        department_id: emp_dept_id,
        employee_id_number: employee_id_number.to_string(),
        tags: employee_tags.cloned().unwrap_or_default(),
    };

    let (matched_rule, rule_traces) =
        crate::services::rule_evaluator::RuleEvaluator::resolve_with_trace(
            &match_ctx,
            &roster_data.active_rules,
        );
    traces.extend(rule_traces);

    if let Some(rule) = matched_rule {
        if let Some(&shift_opt) = roster_data.rule_patterns.get(&(rule.pattern_id, dow)) {
            return (shift_opt, "assignment_rule".to_string(), traces);
        }
    }

    // P4: Department Working Pattern
    let dept_p = roster_data
        .dept_patterns
        .get(&employee_id)
        .and_then(|v| v.iter().find(|a| a.effective_date <= date && a.day_of_week == dow));

    if let Some(dp) = dept_p {
        traces.push(ScheduleTrace {
            rule_id: dp.default_working_pattern_id,
            rule_name: "Department Pattern".to_string(),
            priority: 40,
            status: "applied".to_string(),
            reason: Some("Inherited from department default pattern".to_string()),
        });
        return (dp.shift_id, "department_pattern".to_string(), traces);
    }

    // P5: Fixed Assignment (Individual)
    let fixed = roster_data
        .fixed_assignments
        .get(&employee_id)
        .and_then(|v| {
            v.iter()
                .find(|a| a.start_date <= date && a.end_date.is_none_or(|ed| ed >= date))
        });

    if let Some(f) = fixed {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Fixed Assignment".to_string(),
            priority: 20,
            status: "applied".to_string(),
            reason: Some("Directly assigned shift".to_string()),
        });
        return (Some(f.shift_id), "fixed".to_string(), traces);
    }

    // P6: Department Fixed Shift
    let dept_f = roster_data
        .dept_fixed_shifts
        .get(&employee_id)
        .and_then(|v| v.iter().find(|a| a.effective_date <= date));

    if let Some(df) = dept_f {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Department Fixed".to_string(),
            priority: 10,
            status: "applied".to_string(),
            reason: Some("Inherited from department default shift".to_string()),
        });
        return (Some(df.shift_id), "department_fixed".to_string(), traces);
    }

    (None, "none".to_string(), traces)
}

pub async fn get_workspace_schedules(
    pool: &PgPool,
    roster_cache: Option<&crate::cache::RosterCache>,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    employee_ids: Option<&Vec<Uuid>>,
) -> Result<Vec<EmployeeSchedule>, sqlx::Error> {
    // 1. Fetch Employees (per-request — filtered by employee_ids)
    let employees = sqlx::query(
        "SELECT id, full_name, employee_id_number, tags FROM employees WHERE workspace_id = $1 AND is_active = true AND ($2::uuid[] IS NULL OR id = ANY($2))",
    )
    .bind(workspace_id)
    .bind(employee_ids)
    .fetch_all(pool)
    .await?;

    let employees: Vec<_> = employees
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            let id: Uuid = row.get(0);
            let full_name: String = row.get(1);
            let employee_id_number: String = row.get(2);
            let tags: Option<Vec<String>> = row.get(3);
            (id, full_name, employee_id_number, tags)
        })
        .collect();

    // Load roster config from cache (or directly from DB if cache unavailable)
    let roster_data = match roster_cache {
        Some(cache) => cache
            .get_roster_data(workspace_id, pool)
            .await
            .map_err(|e| sqlx::Error::Io(std::io::Error::other(e)))?,
        None => crate::cache::build_roster_data(pool, workspace_id)
            .await
            .map_err(|e| sqlx::Error::Io(std::io::Error::other(e)))?,
    };

    // 2b. Fetch holidays in range for overlay
    let holidays_map = HolidayRepository::get_holidays_in_range(
        pool, workspace_id, start_date, end_date,
    )
    .await
    .unwrap_or_default();

    // 3. Fetch Rosters in range (per-request — date-bounded)
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

    // Iterate employees to build result
    let mut result = Vec::new();

    for (emp_id, emp_full_name, emp_id_number, emp_tags) in employees {
        let mut daily_schedules = Vec::new();
        let mut curr_date = start_date;

        while curr_date <= end_date {
            let (effective_shift_id, source, traces) = resolve_effective_shift_from_data(
                &roster_data,
                &roster_map,
                emp_id,
                &emp_id_number,
                emp_tags.as_ref(),
                curr_date,
            );

            let holiday = holidays_map.get(&curr_date);
            let mut sched = EffectiveSchedule {
                date: curr_date,
                shift_id: effective_shift_id,
                source,
                shift_name: None,
                shift_symbol: None,
                start_time: None,
                end_time: None,
                break_duration_minutes: None,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: None,
                early_punch_in_grace_period_minutes: None,
                allow_early_ot: None,
                traces,
                is_holiday: holiday.is_some(),
                holiday_name: holiday.map(|h| h.name.clone()),
            };

            if let Some(sid) = effective_shift_id {
                if let Some(shift) = roster_data.shifts.get(&sid) {
                    sched.shift_name = Some(shift.name.clone());
                    sched.shift_symbol = shift.report_symbol.clone();
                    sched.start_time = Some(shift.start_time);
                    sched.end_time = Some(shift.end_time);
                    sched.break_duration_minutes = Some(shift.break_duration_minutes);
                    sched.break_start = shift.break_start;
                    sched.break_end = shift.break_end;
                    sched.is_break_auto_deduct = Some(shift.is_break_auto_deduct);
                    sched.early_punch_in_grace_period_minutes =
                        Some(shift.early_punch_in_grace_period_minutes);
                    sched.allow_early_ot = Some(shift.allow_early_ot);
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
            employee_id_number: emp_id_number,
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
    workspace_id: Uuid,
    employee_id: Uuid,
    date: NaiveDate,
) -> Result<(Option<Uuid>, Vec<crate::models::ScheduleTrace>), sqlx::Error> {
    let mut traces = Vec::new();

    // Priority 1: Shift Roster
    let roster_shift = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT sr.shift_id FROM shift_rosters sr JOIN employees e ON e.id = sr.employee_id WHERE sr.employee_id = $1 AND sr.date = $2 AND e.workspace_id = $3",
    )
    .bind(employee_id)
    .bind(date)
    .bind(workspace_id)
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
        JOIN employees e ON e.id = ewp.employee_id
        WHERE ewp.employee_id = $1 
          AND ewp.start_date <= $2 
          AND (ewp.end_date IS NULL OR ewp.end_date >= $2)
          AND wpd.day_of_week = $3
          AND e.workspace_id = $4
        ORDER BY ewp.created_at DESC
        LIMIT 1
        "#,
        employee_id,
        date,
        day_of_week,
        workspace_id
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
            // Pre-fetch pattern details for all active rules in one batch query
            let raw_rows: Vec<_> = sqlx::query!(
                "SELECT pattern_id, day_of_week, shift_id FROM working_pattern_details WHERE pattern_id IN (SELECT pattern_id FROM assignment_rules WHERE workspace_id = $1 AND is_active = true)",
                info.workspace_id
            )
            .fetch_all(pool)
            .await?;

            let rule_patterns_map: HashMap<(Uuid, i32), Option<Uuid>> = raw_rows
                .into_iter()
                .map(|d| ((d.pattern_id, d.day_of_week), d.shift_id))
                .collect();

            let match_ctx = crate::services::rule_evaluator::EmployeeMatchContext {
                department_id: info.department_id,
                employee_id_number: info.employee_id_number,
                tags: info.tags.unwrap_or_default(),
            };

            let (matched_rule, rule_traces) =
                crate::services::rule_evaluator::RuleEvaluator::resolve_with_trace(
                    &match_ctx, &rules,
                );
            traces.extend(rule_traces);

            if let Some(rule) = matched_rule {
                // Key exists = pattern covers this day; value None = day off, Some(id) = shift
                if let Some(&shift_opt) = rule_patterns_map.get(&(rule.pattern_id, day_of_week)) {
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
        JOIN employees e ON e.id = edh.employee_id
        WHERE edh.employee_id = $1 
          AND edh.effective_date <= $2
          AND wpd.day_of_week = $3
          AND e.workspace_id = $4
        ORDER BY edh.effective_date DESC
        LIMIT 1
        "#,
        employee_id,
        date,
        day_of_week,
        workspace_id
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

    // Priority 5: Employee Default Shift (Legacy Fixed Assignment)
    let employee_shift = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT es.shift_id 
        FROM employee_shifts es
        JOIN employees e ON e.id = es.employee_id
        WHERE es.employee_id = $1 
          AND es.start_date <= $2 
          AND (es.end_date IS NULL OR es.end_date >= $2)
          AND e.workspace_id = $3
        ORDER BY es.created_at DESC 
        LIMIT 1
        "#,
    )
    .bind(employee_id)
    .bind(date)
    .bind(workspace_id)
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

    // Priority 6: Department Default Shift
    let dept_shift = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT dds.shift_id 
        FROM department_default_shifts dds
        JOIN employee_department_history edh ON edh.department_id = dds.department_id
        JOIN employees e ON e.id = edh.employee_id
        WHERE edh.employee_id = $1 AND edh.effective_date <= $2
          AND e.workspace_id = $3
        ORDER BY edh.effective_date DESC LIMIT 1
        "#,
    )
    .bind(employee_id)
    .bind(date)
    .bind(workspace_id)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{
        RosterData, RosterDeptFixedShift, RosterDeptPattern,
        RosterFixedAssignment, RosterPatternEntry,
    };
    use chrono::NaiveDate;
    use std::sync::Arc;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    fn empty_roster_data() -> Arc<RosterData> {
        Arc::new(RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),
            dept_patterns: HashMap::new(),
            dept_fixed_shifts: HashMap::new(),
            active_rules: Vec::new(),
            rule_patterns: HashMap::new(),
            dept_history: HashMap::new(),
        })
    }

    #[test]
    fn test_no_match_returns_none() {
        let data = empty_roster_data();
        let roster_map = HashMap::new();
        let emp = Uuid::new_v4();

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &data,
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert!(shift.is_none());
        assert_eq!(source, "none");
        assert!(traces.is_empty());
    }

    #[test]
    fn test_p1_roster_wins() {
        let data = empty_roster_data();
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();

        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), Some(shift_id));

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &data,
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert_eq!(shift, Some(shift_id));
        assert_eq!(source, "roster");
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].priority, 100);
    }

    #[test]
    fn test_p1_roster_day_off() {
        let data = empty_roster_data();
        let emp = Uuid::new_v4();

        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), None); // Day off

        let (shift, source, _) = resolve_effective_shift_from_data(
            &data,
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert!(shift.is_none());
        assert_eq!(source, "roster");
    }

    #[test]
    fn test_p2_individual_pattern_wins() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        // 2026-03-03 is a Tuesday = num_days_from_sunday = 2
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        data.pattern_assignments.insert(
            emp,
            vec![RosterPatternEntry {
                pattern_id,
                start_date: date(2026, 1, 1),
                end_date: None,
                day_of_week: dow,
                shift_id: Some(shift_id),
            }],
        );

        let roster_map = HashMap::new();

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        assert_eq!(shift, Some(shift_id));
        assert_eq!(source, "pattern");
        assert_eq!(traces[0].priority, 80);
    }

    #[test]
    fn test_p4_department_pattern_wins() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        data.dept_patterns.insert(
            emp,
            vec![RosterDeptPattern {
                default_working_pattern_id: Some(pattern_id),
                effective_date: date(2026, 1, 1),
                day_of_week: dow,
                shift_id: Some(shift_id),
            }],
        );

        let roster_map = HashMap::new();

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        assert_eq!(shift, Some(shift_id));
        assert_eq!(source, "department_pattern");
        assert_eq!(traces[0].priority, 40);
    }

    #[test]
    fn test_p5_fixed_assignment_wins() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();

        let mut data = (*empty_roster_data()).clone();
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );

        let roster_map = HashMap::new();

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert_eq!(shift, Some(shift_id));
        assert_eq!(source, "fixed");
        assert_eq!(traces[0].priority, 20);
    }

    #[test]
    fn test_p6_department_fixed_wins() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();

        let mut data = (*empty_roster_data()).clone();
        data.dept_fixed_shifts.insert(
            emp,
            vec![RosterDeptFixedShift {
                shift_id,
                effective_date: date(2026, 1, 1),
            }],
        );

        let roster_map = HashMap::new();

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert_eq!(shift, Some(shift_id));
        assert_eq!(source, "department_fixed");
        assert_eq!(traces[0].priority, 10);
    }

    #[test]
    fn test_p1_beats_all_lower_priorities() {
        let emp = Uuid::new_v4();
        let roster_shift = Uuid::new_v4();
        let fixed_shift = Uuid::new_v4();
        let dept_fixed_shift = Uuid::new_v4();
        let target_date = date(2026, 3, 3);

        let mut data = (*empty_roster_data()).clone();
        // P5
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id: fixed_shift,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );
        // P6
        data.dept_fixed_shifts.insert(
            emp,
            vec![RosterDeptFixedShift {
                shift_id: dept_fixed_shift,
                effective_date: date(2026, 1, 1),
            }],
        );

        // P1 roster override
        let mut roster_map = HashMap::new();
        roster_map.insert((emp, target_date), Some(roster_shift));

        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        assert_eq!(shift, Some(roster_shift), "P1 roster should beat P5 and P6");
        assert_eq!(source, "roster");
    }

    #[test]
    fn test_expired_pattern_falls_through() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let fixed_shift = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        // P2: expired pattern (ended before target date)
        data.pattern_assignments.insert(
            emp,
            vec![RosterPatternEntry {
                pattern_id,
                start_date: date(2026, 1, 1),
                end_date: Some(date(2026, 2, 28)), // expired
                day_of_week: dow,
                shift_id: Some(shift_id),
            }],
        );
        // P5: active fixed assignment
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id: fixed_shift,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );

        let roster_map = HashMap::new();

        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        assert_eq!(
            shift,
            Some(fixed_shift),
            "expired P2 should fall through to P5"
        );
        assert_eq!(source, "fixed");
    }

    #[test]
    fn test_future_dept_history_ignored() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);

        let mut data = (*empty_roster_data()).clone();
        // Department fixed with future effective date
        data.dept_fixed_shifts.insert(
            emp,
            vec![RosterDeptFixedShift {
                shift_id,
                effective_date: date(2026, 4, 1), // future
            }],
        );

        let roster_map = HashMap::new();

        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        assert!(shift.is_none(), "future effective_date should be ignored");
        assert_eq!(source, "none");
    }
}
