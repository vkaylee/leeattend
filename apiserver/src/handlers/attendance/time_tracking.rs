use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{PunchInRequest, PunchOutRequest, TimeEntry, UpdateTimeEntryRequest};
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
pub async fn punch_in(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<PunchInRequest>,
) -> AppResult<(StatusCode, Json<TimeEntry>)> {
    let workspace_id = ctx.workspace_id;

    let entry = state
        .attendance
        .time_tracking_service
        .punch(
            state.attendance.log_service.as_ref(),
            workspace_id,
            req.employee_id,
            req.note,
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
pub async fn punch_out(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<PunchOutRequest>,
) -> AppResult<Json<TimeEntry>> {
    let (_workspace_id, employee_id) = params;
    let workspace_id = ctx.workspace_id;

    let entry = state
        .attendance
        .time_tracking_service
        .punch(
            state.attendance.log_service.as_ref(),
            workspace_id,
            employee_id,
            req.note,
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
pub async fn get_time_history(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<HistoryQuery>,
) -> AppResult<Json<Vec<TimeEntry>>> {
    let (_workspace_id, employee_id) = params;
    let workspace_id = ctx.workspace_id;

    let entries = state
        .attendance
        .time_tracking_service
        .get_employee_history(workspace_id, employee_id, query.start, query.end)
        .await?;

    Ok(Json(entries))
}

pub async fn update_time_entry(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateTimeEntryRequest>,
) -> AppResult<Json<TimeEntry>> {
    let _workspace_id = ctx.workspace_id;

    let entry = state
        .attendance
        .time_tracking_service
        .update_time_entry(ctx.workspace_id, id, req)
        .await?;

    Ok(Json(entry))
}

pub async fn delete_time_entry(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let workspace_id = ctx.workspace_id;

    state
        .attendance
        .time_tracking_service
        .delete_time_entry(workspace_id, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
