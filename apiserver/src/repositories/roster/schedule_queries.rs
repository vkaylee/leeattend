//! Schedule-related database queries.
//!
//! Contains `get_workspace_schedules` (bulk schedule grid) and
//! `get_effective_shift` (single employee+date DB-backed resolution).

use crate::models::{EffectiveSchedule, EmployeeSchedule, ScheduleTrace, ShiftRoster};
use crate::repositories::holiday_repository::HolidayRepository;
use chrono::NaiveDate;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use super::shift_resolution::resolve_effective_shift_from_data;

/// Build the full schedule grid for a workspace in a date range.
///
/// Uses the roster cache (or builds from DB) plus roster overrides to resolve
/// the effective shift for every (employee, date) pair.
#[allow(clippy::too_many_lines)]
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
    let holidays_map =
        HolidayRepository::get_holidays_in_range(pool, workspace_id, start_date, end_date).await?;

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
            let (effective_shift_ids, source, traces) = resolve_effective_shift_from_data(
                &roster_data,
                &roster_map,
                emp_id,
                &emp_id_number,
                emp_tags.as_ref(),
                curr_date,
            );

            let holiday = holidays_map.get(&curr_date);

            // Emit one EffectiveSchedule per shift (supports split shifts)
            for effective_shift_id in &effective_shift_ids {
                let mut sched = EffectiveSchedule {
                    date: curr_date,
                    shift_id: *effective_shift_id,
                    source: source.clone(),
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
                    traces: traces.clone(),
                    is_holiday: holiday.is_some(),
                    holiday_name: holiday.map(|h| h.name.clone()),
                };

                if let Some(sid) = effective_shift_id {
                    if let Some(shift) = roster_data.shifts.get(sid) {
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
            }

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

/// Resolve the effective shift for a single employee on a single date,
/// querying the database directly (no pre-loaded cache).
///
/// Uses the same 6-tier priority chain as `resolve_effective_shift_from_data`
/// but fetches each tier's data from DB on demand.
#[allow(clippy::too_many_lines)]
pub async fn get_effective_shift(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    date: NaiveDate,
) -> Result<(Option<Uuid>, Vec<ScheduleTrace>), sqlx::Error> {
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
        WHERE e.id = $1 AND e.workspace_id = $3
        ORDER BY edh.effective_date DESC
        LIMIT 1
        "#,
        employee_id,
        date,
        workspace_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(info) = employee_full {
        let rules = crate::repositories::assignment_rule_repository::AssignmentRuleRepository::get_active_rules(pool, info.workspace_id).await?;

        if !rules.is_empty() {
            // Pre-fetch pattern details for all active rules in one batch query
            let raw_rows: Vec<_> = sqlx::query!(
                "SELECT pattern_id, day_of_week, shift_id FROM working_pattern_details WHERE pattern_id IN (SELECT pattern_id FROM assignment_rules WHERE workspace_id = $1 AND is_active = true)",
                info.workspace_id
            )
            .fetch_all(pool)
            .await?;

            let mut rule_patterns_map: HashMap<(Uuid, i32), Vec<Option<Uuid>>> = HashMap::new();
            for d in raw_rows {
                rule_patterns_map
                    .entry((d.pattern_id, d.day_of_week))
                    .or_default()
                    .push(d.shift_id);
            }

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
                // Return the first shift for backward compat in single-query path
                if let Some(shift_opts) = rule_patterns_map.get(&(rule.pattern_id, day_of_week)) {
                    if let Some(&shift_opt) = shift_opts.first() {
                        return Ok((shift_opt, traces));
                    }
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
