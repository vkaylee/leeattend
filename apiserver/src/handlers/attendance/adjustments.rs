use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{
    AdjustmentStatus, ApproveAttendanceAdjustmentRequest, AttendanceAdjustmentRequest,
    CreateAttendanceAdjustmentRequest,
};
use crate::AppState;
use crate::UserId;
use serde::Deserialize;

pub async fn submit_attendance_adjustment(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<CreateAttendanceAdjustmentRequest>,
) -> AppResult<(StatusCode, Json<AttendanceAdjustmentRequest>)> {
    let adjustment = state
        .attendance
        .adjustment_service
        .submit_request(workspace_id, user_id, req)
        .await?;

    Ok((StatusCode::CREATED, Json(adjustment)))
}

pub async fn approve_attendance_adjustment(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveAttendanceAdjustmentRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::AttendanceAdjustmentsApprove)?;
    state
        .attendance
        .adjustment_service
        .approve_request(ctx.workspace_id, user_id, id, req.status)
        .await?;

    Ok(StatusCode::OK)
}

pub async fn get_my_attendance_adjustments(
    Path(id): Path<Uuid>,
    UserId(_user_id): UserId,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AttendanceAdjustmentRequest>>> {
    let requests = state
        .attendance
        .adjustment_service
        .get_my_requests(ctx.workspace_id, id)
        .await?;

    Ok(Json(requests))
}

#[derive(Deserialize)]
pub struct AdjustmentStatusQuery {
    pub status: Option<AdjustmentStatus>,
}

pub async fn get_workspace_attendance_adjustments(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AdjustmentStatusQuery>,
) -> AppResult<Json<Vec<AttendanceAdjustmentRequest>>> {
    let requests = state
        .attendance
        .adjustment_service
        .get_workspace_requests(workspace_id, query.status)
        .await?;

    Ok(Json(requests))
}
