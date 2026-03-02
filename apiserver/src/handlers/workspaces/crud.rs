use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    AttendanceTrendPoint, CreateWorkspaceRequest, DepartmentAttendancePoint, LatenessByDay,
    LeaveTypeBreakdown, OvertimeTrendPoint, PeakTimeSlot, UpdateWorkspaceRequest, Workspace,
    WorkspaceStats,
};
use crate::services::WorkspaceService;
use crate::{AppState, UserId};

pub async fn get_attendance_trend(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<AttendanceTrendPoint>>> {
    let today = Utc::now().date_naive();

    // Main query: present + on_leave per day (uses only guaranteed tables)
    #[derive(Debug, sqlx::FromRow)]
    struct BaseRow {
        date: NaiveDate,
        present: i64,
        on_leave: i64,
    }

    let base_rows = sqlx::query_as::<_, BaseRow>(
        r#"
        WITH days AS (
            SELECT d::date AS date
            FROM generate_series(($1::date - INTERVAL '29 days'), $1::date, '1 day'::interval) AS d
        ),
        active_employees AS (
            SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE
        ),
        daily_present AS (
            SELECT d.date,
                   COUNT(DISTINCT te.employee_id) AS present
            FROM days d
            LEFT JOIN time_entries te
                ON te.punch_in_time::date = d.date
                AND te.employee_id IN (SELECT id FROM active_employees)
            GROUP BY d.date
        ),
        daily_leave AS (
            SELECT d.date,
                   COUNT(DISTINCT lr.employee_id) AS on_leave
            FROM days d
            LEFT JOIN leave_requests lr
                ON lr.status = 'approved'
                AND lr.start_date <= d.date
                AND lr.end_date >= d.date
                AND lr.employee_id IN (SELECT id FROM active_employees)
            GROUP BY d.date
        )
        SELECT dp.date,
               COALESCE(dp.present, 0) AS present,
               COALESCE(dle.on_leave, 0) AS on_leave
        FROM daily_present dp
        LEFT JOIN daily_leave dle ON dp.date = dle.date
        ORDER BY dp.date ASC
        "#,
    )
    .bind(today)
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await?;

    // Late detection: separate query (roster_entries may not exist yet)
    #[derive(Debug, sqlx::FromRow)]
    struct LateRow {
        work_date: NaiveDate,
        late_count: i64,
    }

    let late_map: std::collections::HashMap<NaiveDate, i64> = sqlx::query_as::<_, LateRow>(
        r#"
        WITH active_employees AS (
            SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE
        ),
        first_punches AS (
            SELECT te.employee_id,
                   te.punch_in_time::date AS work_date,
                   MIN(te.punch_in_time)::time AS first_punch
            FROM time_entries te
            WHERE te.punch_in_time::date >= ($1::date - INTERVAL '29 days')
              AND te.punch_in_time::date <= $1::date
              AND te.employee_id IN (SELECT id FROM active_employees)
            GROUP BY te.employee_id, te.punch_in_time::date
        )
        SELECT fp.work_date,
               COUNT(DISTINCT fp.employee_id) AS late_count
        FROM first_punches fp
        JOIN roster_entries re ON re.employee_id = fp.employee_id AND re.date = fp.work_date
        JOIN shifts s ON s.id = re.shift_id
        WHERE fp.first_punch > (s.start_time + INTERVAL '5 minutes')
        GROUP BY fp.work_date
        "#,
    )
    .bind(today)
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| (r.work_date, r.late_count))
    .collect();

    // Total active employees for absent calculation
    let total_active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM employees WHERE workspace_id = $1 AND is_active = TRUE",
    )
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let trend: Vec<AttendanceTrendPoint> = base_rows
        .into_iter()
        .map(|r| {
            let late = *late_map.get(&r.date).unwrap_or(&0);
            let absent = (total_active - r.present - r.on_leave).max(0);
            AttendanceTrendPoint {
                date: r.date,
                present: r.present,
                late,
                absent,
                on_leave: r.on_leave,
            }
        })
        .collect();

    Ok(Json(trend))
}

#[derive(Debug, Deserialize)]
pub struct DeptAttendanceQuery {
    pub range: Option<String>,
}

pub async fn get_department_attendance(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(params): Query<DeptAttendanceQuery>,
) -> AppResult<Json<Vec<DepartmentAttendancePoint>>> {
    let today = Utc::now().date_naive();
    let range = params.range.unwrap_or_else(|| "today".to_string());
    let start_date = match range.as_str() {
        "week" => today - chrono::Duration::days(6),
        _ => today,
    };

    // Departments with employee counts
    #[derive(Debug, sqlx::FromRow)]
    struct DeptRow {
        department_id: Uuid,
        department_name: String,
        total_employees: i64,
    }

    let departments = sqlx::query_as::<_, DeptRow>(
        r#"
        SELECT d.id AS department_id,
               d.name AS department_name,
               COUNT(DISTINCT e.id) AS total_employees
        FROM departments d
        JOIN employee_department_history edh ON edh.department_id = d.id
        JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
        WHERE d.workspace_id = $1
          AND edh.effective_date <= $2
          AND edh.id = (
            SELECT edh2.id FROM employee_department_history edh2
            WHERE edh2.employee_id = edh.employee_id AND edh2.effective_date <= $2
            ORDER BY edh2.effective_date DESC LIMIT 1
          )
        GROUP BY d.id, d.name
        "#,
    )
    .bind(workspace_id)
    .bind(today)
    .fetch_all(&state.db)
    .await?;

    if departments.is_empty() {
        return Ok(Json(vec![]));
    }

    // Present per department in date range
    #[derive(Debug, sqlx::FromRow)]
    struct PresentRow {
        department_id: Uuid,
        present_count: i64,
    }

    let start_dt = start_date
        .and_hms_opt(0, 0, 0)
        .and_then(|t| t.and_local_timezone(Utc).single())
        .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

    let present_rows = sqlx::query_as::<_, PresentRow>(
        r#"
        WITH latest_dept AS (
            SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
            FROM employee_department_history edh
            JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
            WHERE edh.effective_date <= $3
            ORDER BY edh.employee_id, edh.effective_date DESC
        )
        SELECT ld.department_id,
               COUNT(DISTINCT te.employee_id) AS present_count
        FROM time_entries te
        JOIN latest_dept ld ON ld.employee_id = te.employee_id
        WHERE te.punch_in_time >= $2
        GROUP BY ld.department_id
        "#,
    )
    .bind(workspace_id)
    .bind(start_dt)
    .bind(today)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let present_map: std::collections::HashMap<Uuid, i64> = present_rows
        .into_iter()
        .map(|r| (r.department_id, r.present_count))
        .collect();

    // Late per department (only for "today")
    let late_map: std::collections::HashMap<Uuid, i64> = if range == "today" {
        #[derive(Debug, sqlx::FromRow)]
        struct LateRow {
            department_id: Uuid,
            late_count: i64,
        }
        sqlx::query_as::<_, LateRow>(
            r#"
            WITH latest_dept AS (
                SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
                FROM employee_department_history edh
                JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
                WHERE edh.effective_date <= $3
                ORDER BY edh.employee_id, edh.effective_date DESC
            ),
            first_punches AS (
                SELECT te.employee_id, MIN(te.punch_in_time) AS first_punch
                FROM time_entries te
                WHERE te.punch_in_time >= $2
                  AND te.employee_id IN (SELECT employee_id FROM latest_dept)
                GROUP BY te.employee_id
            )
            SELECT ld.department_id,
                   COUNT(DISTINCT fp.employee_id) AS late_count
            FROM first_punches fp
            JOIN latest_dept ld ON ld.employee_id = fp.employee_id
            JOIN roster_entries re ON re.employee_id = fp.employee_id AND re.date = $3
            JOIN shifts s ON s.id = re.shift_id
            WHERE fp.first_punch::time > (s.start_time + INTERVAL '5 minutes')
            GROUP BY ld.department_id
            "#,
        )
        .bind(workspace_id)
        .bind(start_dt)
        .bind(today)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| (r.department_id, r.late_count))
        .collect()
    } else {
        std::collections::HashMap::new()
    };

    // On leave per department
    #[derive(Debug, sqlx::FromRow)]
    struct LeaveRow {
        department_id: Uuid,
        leave_count: i64,
    }

    let leave_rows = sqlx::query_as::<_, LeaveRow>(
        r#"
        WITH latest_dept AS (
            SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
            FROM employee_department_history edh
            JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
            WHERE edh.effective_date <= $3
            ORDER BY edh.employee_id, edh.effective_date DESC
        )
        SELECT ld.department_id,
               COUNT(DISTINCT lr.employee_id) AS leave_count
        FROM leave_requests lr
        JOIN latest_dept ld ON ld.employee_id = lr.employee_id
        WHERE lr.status = 'approved'
          AND lr.start_date <= $3 AND lr.end_date >= $2
        GROUP BY ld.department_id
        "#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(today)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let leave_map: std::collections::HashMap<Uuid, i64> = leave_rows
        .into_iter()
        .map(|r| (r.department_id, r.leave_count))
        .collect();

    // Build result
    let result: Vec<DepartmentAttendancePoint> = departments
        .into_iter()
        .map(|dept| {
            let present = *present_map.get(&dept.department_id).unwrap_or(&0);
            let late = *late_map.get(&dept.department_id).unwrap_or(&0);
            let on_leave = *leave_map.get(&dept.department_id).unwrap_or(&0);
            let on_time = (present - late).max(0);
            let absent = (dept.total_employees - present - on_leave).max(0);
            DepartmentAttendancePoint {
                department_id: dept.department_id,
                department_name: dept.department_name,
                on_time,
                late,
                absent,
                on_leave,
            }
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_overtime_trend(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<OvertimeTrendPoint>>> {
    let today = Utc::now().date_naive();

    #[derive(Debug, sqlx::FromRow)]
    struct OtRow {
        date: NaiveDate,
        system_ot_minutes: i64,
        approved_ot_minutes: i64,
    }

    let rows = sqlx::query_as::<_, OtRow>(
        r#"
        SELECT ads.date,
               COALESCE(SUM(ads.system_detected_ot_minutes), 0) AS system_ot_minutes,
               COALESCE(SUM(ads.approved_ot_minutes), 0) AS approved_ot_minutes
        FROM attendance_daily_summaries ads
        WHERE ads.workspace_id = $1
          AND ads.date >= ($2::date - INTERVAL '29 days')
          AND ads.date <= $2
        GROUP BY ads.date
        ORDER BY ads.date ASC
        "#,
    )
    .bind(workspace_id)
    .bind(today)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let trend: Vec<OvertimeTrendPoint> = rows
        .into_iter()
        .map(|r| OvertimeTrendPoint {
            date: r.date,
            system_ot_hours: r.system_ot_minutes as f64 / 60.0,
            approved_ot_hours: r.approved_ot_minutes as f64 / 60.0,
        })
        .collect();

    Ok(Json(trend))
}

pub async fn get_workspace_stats(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<WorkspaceStats>> {
    // 1. Total Employees
    let total_employees: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM employees WHERE workspace_id = $1 AND is_active = TRUE",
    )
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // 2. Present Today (Employees with at least one punch-in today)
    let today = Utc::now().date_naive();
    let start_of_day = today
        .and_hms_opt(0, 0, 0)
        .and_then(|t| t.and_local_timezone(Utc).single())
        .ok_or_else(|| AppError::Internal("Invalid date calculation".to_string()))?;

    let present_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT employee_id) FROM time_entries
         WHERE punch_in_time >= $1
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2)",
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // 3. Pending Leave Requests
    let pending_leave: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM leave_requests
         WHERE status = 'pending'
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
    )
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // 4. Estimated Monthly Payroll (Sum of latest base salaries)
    // Priority: employee_salary_components -> fallback to salary_history
    let payroll: rust_decimal::Decimal = sqlx::query_scalar(
        r#"
        WITH active_emps AS (
            SELECT id FROM employees WHERE workspace_id = $1 AND is_active = TRUE
        ),
        dynamic_payroll AS (
            SELECT esc.employee_id, SUM(esc.amount) as total_amount
            FROM employee_salary_components esc
            INNER JOIN (
                SELECT employee_id, component_definition_id, MAX(effective_date) as max_date
                FROM employee_salary_components
                WHERE effective_date <= CURRENT_DATE
                GROUP BY employee_id, component_definition_id
            ) latest ON esc.employee_id = latest.employee_id
                     AND esc.component_definition_id = latest.component_definition_id
                     AND esc.effective_date = latest.max_date
            WHERE esc.employee_id IN (SELECT id FROM active_emps)
            GROUP BY esc.employee_id
        )
        SELECT COALESCE(SUM(total_amount), 0) FROM dynamic_payroll
        "#,
    )
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(rust_decimal::Decimal::ZERO);

    // 5. On Leave Today (approved leave requests spanning today)
    let on_leave_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT lr.employee_id) FROM leave_requests lr
         WHERE lr.status = 'approved'
         AND lr.start_date <= $1 AND lr.end_date >= $1
         AND lr.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)",
    )
    .bind(today)
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // 6. Absent Today = total_employees - present_today - on_leave_today
    let absent_today = (total_employees - present_today - on_leave_today).max(0);

    // 7. Late Today (employees whose earliest punch-in is > 5 min after scheduled shift start)
    let late_today: i64 = sqlx::query_scalar(
        r#"
        WITH today_entries AS (
            SELECT te.employee_id, MIN(te.punch_in_time) as first_punch
            FROM time_entries te
            WHERE te.punch_in_time >= $1
            AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
            GROUP BY te.employee_id
        ),
        scheduled AS (
            SELECT re.employee_id, s.start_time
            FROM roster_entries re
            JOIN shifts s ON re.shift_id = s.id
            WHERE re.date = $3
            AND re.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
        )
        SELECT COUNT(*) FROM today_entries te
        JOIN scheduled sc ON te.employee_id = sc.employee_id
        WHERE te.first_punch::time > (sc.start_time + INTERVAL '5 minutes')
        "#,
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .bind(today)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // 8. Early Departure Today (employees who punched out before scheduled end)
    let early_departure_today: i64 = sqlx::query_scalar(
        r#"
        WITH today_completed AS (
            SELECT te.employee_id, MAX(te.punch_out_time) as last_punch_out
            FROM time_entries te
            WHERE te.punch_in_time >= $1
            AND te.punch_out_time IS NOT NULL
            AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
            GROUP BY te.employee_id
        ),
        scheduled AS (
            SELECT re.employee_id, s.end_time
            FROM roster_entries re
            JOIN shifts s ON re.shift_id = s.id
            WHERE re.date = $3
            AND re.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
        )
        SELECT COUNT(*) FROM today_completed tc
        JOIN scheduled sc ON tc.employee_id = sc.employee_id
        WHERE tc.last_punch_out::time < sc.end_time
        "#,
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .bind(today)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // 9. Average work hours today
    let avg_work_minutes: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(duration_minutes)::float8 FROM time_entries
         WHERE punch_in_time >= $1
         AND punch_out_time IS NOT NULL
         AND duration_minutes > 0
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)",
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(None);

    let avg_work_hours_today = avg_work_minutes.unwrap_or(0.0) / 60.0;

    Ok(Json(WorkspaceStats {
        total_employees,
        present_today,
        pending_leave_requests: pending_leave,
        estimated_monthly_payroll: payroll,
        late_today,
        absent_today,
        on_leave_today,
        early_departure_today,
        avg_work_hours_today,
    }))
}

// ── Peak Check-in/Check-out Times ──────────────────────────────────

pub async fn get_peak_times(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<PeakTimeSlot>>> {
    let today = Utc::now().date_naive();
    let start_of_day = today
        .and_hms_opt(0, 0, 0)
        .and_then(|t| t.and_local_timezone(Utc).single())
        .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

    // Generate 30-minute time slots and count check-ins/check-outs per slot
    #[derive(Debug, sqlx::FromRow)]
    struct SlotRow {
        time_slot: String,
        check_in: i64,
        check_out: i64,
    }

    let rows = sqlx::query_as::<_, SlotRow>(
        r#"
        WITH slots AS (
            SELECT generate_series(0, 47) AS slot_index
        ),
        slot_labels AS (
            SELECT slot_index,
                   TO_CHAR((TIME '00:00' + (slot_index * INTERVAL '30 minutes')), 'HH24:MI')
                   || '-' ||
                   TO_CHAR((TIME '00:00' + ((slot_index + 1) * INTERVAL '30 minutes')), 'HH24:MI')
                   AS time_slot,
                   (TIME '00:00' + (slot_index * INTERVAL '30 minutes')) AS slot_start,
                   (TIME '00:00' + ((slot_index + 1) * INTERVAL '30 minutes')) AS slot_end
            FROM slots
        ),
        today_entries AS (
            SELECT punch_in_time, punch_out_time
            FROM time_entries
            WHERE punch_in_time >= $1
              AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
        )
        SELECT sl.time_slot,
               COALESCE(SUM(CASE WHEN te.punch_in_time::time >= sl.slot_start
                                  AND te.punch_in_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0) AS check_in,
               COALESCE(SUM(CASE WHEN te.punch_out_time IS NOT NULL
                                  AND te.punch_out_time::time >= sl.slot_start
                                  AND te.punch_out_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0) AS check_out
        FROM slot_labels sl
        LEFT JOIN today_entries te ON TRUE
        GROUP BY sl.slot_index, sl.time_slot
        HAVING COALESCE(SUM(CASE WHEN te.punch_in_time::time >= sl.slot_start
                                  AND te.punch_in_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0) > 0
            OR COALESCE(SUM(CASE WHEN te.punch_out_time IS NOT NULL
                                  AND te.punch_out_time::time >= sl.slot_start
                                  AND te.punch_out_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0) > 0
        ORDER BY sl.slot_index
        "#,
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let result: Vec<PeakTimeSlot> = rows
        .into_iter()
        .map(|r| PeakTimeSlot {
            time_slot: r.time_slot,
            check_in: r.check_in,
            check_out: r.check_out,
        })
        .collect();

    Ok(Json(result))
}

// ── Leave Types Breakdown ──────────────────────────────────────────

pub async fn get_leave_types_breakdown(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<LeaveTypeBreakdown>>> {
    let today = Utc::now().date_naive();
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

    #[derive(Debug, sqlx::FromRow)]
    struct LtRow {
        leave_type: String,
        count: i64,
    }

    let rows = sqlx::query_as::<_, LtRow>(
        r#"
        SELECT COALESCE(lt.name, 'Other') AS leave_type,
               COUNT(*) AS count
        FROM leave_requests lr
        LEFT JOIN leave_types lt ON lt.id = lr.leave_type_id
        WHERE lr.status = 'approved'
          AND lr.start_date <= $2
          AND lr.end_date >= $1
          AND lr.employee_id IN (
              SELECT id FROM employees WHERE workspace_id = $3 AND is_active = TRUE
          )
        GROUP BY COALESCE(lt.name, 'Other')
        ORDER BY count DESC
        "#,
    )
    .bind(month_start)
    .bind(today)
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let result: Vec<LeaveTypeBreakdown> = rows
        .into_iter()
        .map(|r| LeaveTypeBreakdown {
            leave_type: r.leave_type,
            count: r.count,
        })
        .collect();

    Ok(Json(result))
}

// ── Lateness by Day of Week ────────────────────────────────────────

pub async fn get_lateness_by_day(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<LatenessByDay>>> {
    let today = Utc::now().date_naive();

    #[derive(Debug, sqlx::FromRow)]
    struct DayRow {
        dow: i32,
        late_count: i64,
        avg_late_minutes: f64,
    }

    let rows = sqlx::query_as::<_, DayRow>(
        r#"
        WITH first_punches AS (
            SELECT te.employee_id,
                   te.punch_in_time::date AS work_date,
                   MIN(te.punch_in_time)::time AS first_punch
            FROM time_entries te
            WHERE te.punch_in_time::date >= ($1::date - INTERVAL '29 days')
              AND te.punch_in_time::date <= $1
              AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
            GROUP BY te.employee_id, te.punch_in_time::date
        ),
        resolved_shifts AS (
            SELECT DISTINCT ON (fp.employee_id, fp.work_date)
                   fp.employee_id, fp.work_date, fp.first_punch, COALESCE(
                       -- Priority 1: shift_rosters (manual roster)
                       (SELECT sr.shift_id FROM shift_rosters sr
                        WHERE sr.employee_id = fp.employee_id AND sr.date = fp.work_date
                        LIMIT 1),
                       -- Priority 2: individual working pattern
                       (SELECT wpd.shift_id FROM working_pattern_details wpd
                        JOIN employee_working_patterns ewp ON ewp.pattern_id = wpd.pattern_id
                        WHERE ewp.employee_id = fp.employee_id
                          AND ewp.start_date <= fp.work_date
                          AND (ewp.end_date IS NULL OR ewp.end_date >= fp.work_date)
                          AND wpd.day_of_week = EXTRACT(DOW FROM fp.work_date)::int
                        ORDER BY ewp.created_at DESC LIMIT 1),
                       -- Priority 3: employee_shifts (default assignment)
                       (SELECT es.shift_id FROM employee_shifts es
                        WHERE es.employee_id = fp.employee_id
                          AND es.start_date <= fp.work_date
                          AND (es.end_date IS NULL OR es.end_date >= fp.work_date)
                        LIMIT 1)
                   ) AS shift_id
            FROM first_punches fp
        ),
        late_entries AS (
            SELECT rs.work_date,
                   EXTRACT(ISODOW FROM rs.work_date)::int AS dow,
                   EXTRACT(EPOCH FROM (rs.first_punch - s.start_time)) / 60.0 AS late_minutes
            FROM resolved_shifts rs
            JOIN shifts s ON s.id = rs.shift_id
            WHERE rs.shift_id IS NOT NULL
              AND rs.first_punch > (s.start_time + INTERVAL '5 minutes')
        )
        SELECT dow,
               COUNT(*) AS late_count,
               COALESCE(AVG(late_minutes), 0)::float8 AS avg_late_minutes
        FROM late_entries
        GROUP BY dow
        ORDER BY dow
        "#,
    )
    .bind(today)
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let result: Vec<LatenessByDay> = rows
        .into_iter()
        .map(|r| {
            let day_idx = (r.dow - 1).clamp(0, 6) as usize;
            LatenessByDay {
                day: day_names[day_idx].to_string(),
                late_count: r.late_count,
                avg_late_minutes: (r.avg_late_minutes * 10.0).round() / 10.0,
            }
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_workspaces(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> AppResult<Json<Vec<Workspace>>> {
    let workspaces = sqlx::query_as::<_, Workspace>(
        "SELECT w.* FROM workspaces w
         JOIN workspace_members wm ON w.id = wm.workspace_id
         WHERE wm.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(workspaces))
}

pub async fn get_workspace_by_id(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    UserId(_user_id): UserId,
) -> AppResult<Json<Workspace>> {
    let workspace = state
        .workspace_repo
        .get_workspace_by_id(ws_id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    Ok(Json(workspace))
}

pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<CreateWorkspaceRequest>,
) -> AppResult<Json<Workspace>> {
    WorkspaceService::create_workspace(&state.db, &req.name, user_id)
        .await
        .map(Json)
        .map_err(AppError::from)
}

pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> AppResult<Json<Workspace>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Admin or Owner role required to update workspace".to_string(),
        ));
    }
    crate::services::workspace_service::WorkspaceService::update_workspace(
        &state.db,
        ws_id,
        req.name,
        req.work_unit_minutes,
        req.rounding_policy,
        req.date_format,
        req.time_format,
        req.timezone,
        req.theme_primary_color,
        req.default_language,
        req.pairing_algorithm,
        req.early_arrival_strategy,
        req.holiday_overnight_strategy,
        req.default_holiday_ot_multiplier,
    )
    .await
    .map(Json)
    .map_err(AppError::from)
}
