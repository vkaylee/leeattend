use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{CreateWorkspaceRequest, UpdateWorkspaceRequest, Workspace, WorkspaceStats};
use crate::services::WorkspaceService;
use crate::{AppState, UserId};

pub async fn get_workspace_stats(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> Result<impl IntoResponse, (StatusCode, String)> {
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
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid date calculation".to_string(),
        ))?;

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
        "#
    )
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(rust_decimal::Decimal::ZERO);

    Ok(Json(WorkspaceStats {
        total_employees,
        present_today,
        pending_leave_requests: pending_leave,
        estimated_monthly_payroll: payroll,
    }))
}

pub async fn get_workspaces(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> Result<Json<Vec<Workspace>>, (StatusCode, String)> {
    let workspaces = sqlx::query_as::<_, Workspace>(
        "SELECT w.* FROM workspaces w
         JOIN workspace_members wm ON w.id = wm.workspace_id
         WHERE wm.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(workspaces))
}

pub async fn get_workspace_by_id(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    UserId(_user_id): UserId,
) -> Result<Json<Workspace>, (StatusCode, String)> {
    let workspace = state
        .workspace_repo
        .get_workspace_by_id(ws_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or((StatusCode::NOT_FOUND, "Workspace not found".to_string()))?;

    Ok(Json(workspace))
}

pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>, (StatusCode, String)> {
    match WorkspaceService::create_workspace(&state.db, &req.name, user_id).await {
        Ok(ws) => Ok(Json(ws)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    UserId(_user_id): UserId,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>, (StatusCode, String)> {
    let ws = crate::services::workspace_service::WorkspaceService::update_workspace(
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
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ws))
}
