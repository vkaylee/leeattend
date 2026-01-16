use axum::{
    extract::{State, Path, Query},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use serde::Deserialize;

use crate::{AppState, UserId};
use crate::handlers::common::MemberContext;
use crate::models::{
    Workspace, CreateWorkspaceRequest, UpdateWorkspaceRequest, WorkspaceStats,
    AddMemberRequest, AssignRoleRequest, MemberExtraRole,
    CreateDepartmentRequest, UpdateDepartmentRequest
};
use crate::services::WorkspaceService;

// --- Local DTOs ---

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct ClearWorkspaceDataQuery {
    pub group: String,
}

// --- Workspace Handlers ---

pub async fn get_workspace_stats(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    // 1. Total Employees
    let total_employees: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM employees WHERE workspace_id = $1 AND is_active = TRUE")
        .bind(workspace_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    // 2. Present Today (Employees with at least one punch-in today)
    let today = Utc::now().date_naive();
    let present_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT employee_id) FROM time_entries 
         WHERE punch_in_time >= $1 
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2)"
    )
    .bind(today.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Utc).unwrap())
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // 3. Pending Leave Requests
    let pending_leave: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM leave_requests 
         WHERE status = 'pending' 
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)"
    )
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // 4. Estimated Monthly Payroll (Sum of latest base salaries)
    let payroll: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(s.base_salary), 0) FROM salary_history s
         INNER JOIN (
            SELECT employee_id, MAX(effective_date) as max_date 
            FROM salary_history 
            GROUP BY employee_id
         ) latest ON s.employee_id = latest.employee_id AND s.effective_date = latest.max_date
         WHERE s.employee_id IN (SELECT id FROM employees WHERE workspace_id = $1 AND is_active = TRUE)"
    )
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(rust_decimal::Decimal::ZERO);

    Json(WorkspaceStats {
        total_employees,
        present_today,
        pending_leave_requests: pending_leave,
        estimated_monthly_payroll: payroll,
    })
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
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ws))
}

pub async fn get_workspace_members(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as::<_, crate::models::WorkspaceMemberDetail>(
        "SELECT wm.workspace_id, wm.user_id, wm.role, u.email, u.full_name, u.avatar_url 
         FROM workspace_members wm
         JOIN users u ON wm.user_id = u.id
         WHERE wm.workspace_id = $1",
    )
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(members) => Json(members).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn add_workspace_member(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can invite members").into_response();
    }

    let user_id: Uuid = match sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "User with this email not found").into_response();
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 3. Add to workspace
    match sqlx::query(
        "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(&req.role)
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_workspace_member_role(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (
            StatusCode::FORBIDDEN,
            "Only admins can change roles".to_string(),
        )
            .into_response();
    }

    match state
        .workspace_repo
        .update_member_role(ctx.workspace_id, user_id, &req.role)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn remove_workspace_member(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (
            StatusCode::FORBIDDEN,
            "Only admins can remove members".to_string(),
        )
            .into_response();
    }

    // Don't allow removing yourself (prevent lockout if only one admin, though owner should probably remain)
    if user_id == ctx.user_id {
        return (
            StatusCode::BAD_REQUEST,
            "You cannot remove yourself from the workspace".to_string(),
        )
            .into_response();
    }

    match state
        .workspace_repo
        .remove_member(ctx.workspace_id, user_id)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

// --- Temporal Roles Handlers ---

pub async fn get_member_roles(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    match WorkspaceService::get_member_roles(
        state.workspace_repo.as_ref(),
        ctx.workspace_id,
        user_id,
    )
    .await
    {
        Ok((primary, extra)) => Json(serde_json::json!({
            "primary": primary,
            "extra": extra
        }))
        .into_response(),
        Err(e) if e == "NOT_FOUND" => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn assign_extra_role(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignRoleRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can assign roles").into_response();
    }

    if req.expires_at <= chrono::Utc::now() {
        return (
            StatusCode::BAD_REQUEST,
            "Expiration time must be in the future".to_string(),
        )
            .into_response();
    }

    match sqlx::query_as::<_, MemberExtraRole>(
        "INSERT INTO member_extra_roles (workspace_id, user_id, role, expires_at) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(&req.role)
    .bind(req.expires_at)
    .fetch_one(&state.db)
    .await
    {
        Ok(extra_role) => (StatusCode::CREATED, Json(extra_role)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn revoke_extra_role(
    Path((_ws_id, _user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can revoke roles").into_response();
    }

    match sqlx::query("DELETE FROM member_extra_roles WHERE id = $1 AND workspace_id = $2")
        .bind(role_id)
        .bind(ctx.workspace_id)
        .execute(&state.db)
        .await
    {
        Ok(res) if res.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn clear_workspace_data(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ClearWorkspaceDataQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut tx = state.db.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match query.group.as_str() {
        "attendance" => {
            sqlx::query("DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM raw_attendance_logs WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM attendance_import_jobs WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        "schedules" => {
            sqlx::query("DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        "leave" => {
            sqlx::query("DELETE FROM leave_accrual_logs WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        "payroll" => {
            sqlx::query("DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM salary_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        "full_reset" => {
            sqlx::query("DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM raw_attendance_logs WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM attendance_import_jobs WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM leave_accrual_logs WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM salary_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM leave_policy_assignments WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM leave_policy_definitions WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM employees WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM shifts WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM departments WHERE workspace_id = $1").bind(ws_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        _ => return Err((StatusCode::BAD_REQUEST, "Invalid group".to_string())),
    };

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn get_departments(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::department_repository::get_departments(&state.db, workspace_id).await
    {
        Ok(departments) => Json(departments).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_department(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateDepartmentRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage departments").into_response();
    }
    match crate::repositories::department_repository::create_department(
        &state.db,
        ctx.workspace_id,
        req.name,
        req.description,
        req.manager_id,
        req.default_working_pattern_id,
    )
    .await
    {
        Ok(dept) => (StatusCode::CREATED, Json(dept)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage departments").into_response();
    }
    match crate::repositories::department_repository::update_department(
        &state.db,
        ctx.workspace_id,
        id,
        req.name,
        req.description,
        req.manager_id,
        req.default_working_pattern_id,
    )
    .await
    {
        Ok(dept) => Json(dept).into_response(),
        Err(e) => {
            tracing::error!("Failed to update department: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn delete_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage departments").into_response();
    }
    match crate::repositories::department_repository::delete_department(&state.db, ctx.workspace_id, id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
