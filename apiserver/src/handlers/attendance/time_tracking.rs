use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{PunchInRequest, PunchOutRequest, TimeEntry, UpdateTimeEntryRequest};
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct HistoryQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/time/punch-in",
    request_body = PunchInRequest,
    responses(
        (status = 201, description = "Punch in successful", body = TimeEntry)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn punch_in(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<PunchInRequest>,
) -> AppResult<(StatusCode, Json<TimeEntry>)> {
    // RBAC: self-service punch requires EssClock permission
    ctx.require_permission(Permission::EssClock)?;

    let entry = state
        .attendance()
        .time_tracking_service()
        .punch(
            state.attendance().log_service().as_ref(),
            &ctx.auth_token,
            req.employment_id(),
            req.note().map(String::from),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(entry)))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/time/punch-out/{employee_id}",
    request_body = PunchOutRequest,
    responses(
        (status = 200, description = "Punch out successful", body = TimeEntry)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, employee_id = %params.1))]
pub async fn punch_out(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<PunchOutRequest>,
) -> AppResult<Json<TimeEntry>> {
    let (_workspace_id, employee_id) = params;

    // RBAC: force punch-out for another employee is admin-only
    ctx.require_permission(Permission::AttendanceForceClockOut)?;

    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;

    let active_employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    let entry = state
        .attendance()
        .time_tracking_service()
        .punch(
            state.attendance().log_service().as_ref(),
            &ctx.auth_token,
            active_employment_id,
            req.note().map(String::from),
        )
        .await?;

    Ok(Json(entry))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/time/history/{employee_id}",
    responses(
        (status = 200, description = "History fetched", body = Vec<TimeEntry>)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        HistoryQuery
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, employee_id = %params.1))]
pub async fn get_time_history(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<HistoryQuery>,
) -> AppResult<Json<Vec<TimeEntry>>> {
    let (_workspace_id, employee_id) = params;

    // RBAC: admin can read any employee's history; employee can only read own.
    let has_read_perm = ctx.has_permission(Permission::AttendanceLogsRead);
    if !has_read_perm {
        let my_employee_id = state
            .employees()
            .repo()
            .get_employee_id_by_user_id(&ctx.auth_token, ctx.user_id)
            .await?
            .ok_or_else(|| AppError::forbidden(I18nKey::Forbidden))?;
        if employee_id != my_employee_id {
            return Err(AppError::forbidden(I18nKey::Forbidden));
        }
    }

    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;

    let active_employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    let entries = state
        .attendance()
        .time_tracking_service()
        .get_employee_history(
            &ctx.auth_token,
            active_employment_id,
            query.start,
            query.end,
        )
        .await?;

    Ok(Json(entries))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/attendance/time-tracking/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Time Entry ID")
    ),
    request_body = UpdateTimeEntryRequest,
    responses(
        (status = 200, description = "Time entry updated", body = TimeEntry)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, entry_id = %__path_params.1))]
pub async fn update_time_entry(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateTimeEntryRequest>,
) -> AppResult<Json<TimeEntry>> {
    let (_, id) = __path_params;

    // RBAC: updating time entries is admin-only
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    let entry = state
        .attendance()
        .time_tracking_service()
        .update_time_entry(&ctx.auth_token, id, req)
        .await?;

    Ok(Json(entry))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/attendance/time-tracking/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Time Entry ID")
    ),
    responses(
        (status = 204, description = "Time entry deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, entry_id = %__path_params.1))]
pub async fn delete_time_entry(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_, id) = __path_params;

    // RBAC: deleting time entries is admin-only
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    state
        .attendance()
        .time_tracking_service()
        .delete_time_entry(&ctx.auth_token, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
