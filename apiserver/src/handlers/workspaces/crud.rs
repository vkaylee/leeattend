use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::{NaiveDate, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    AttendanceTrendPoint, CreateWorkspaceRequest, UpdateWorkspaceRequest, Workspace, WorkspaceStats,
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
