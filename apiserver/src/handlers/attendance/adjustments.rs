use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    AdjustmentStatus, ApproveAttendanceAdjustmentRequest, AttendanceAdjustmentRequest,
    CreateAttendanceAdjustmentRequest,
};
use crate::AppState;
use crate::UserId;
use serde::Deserialize;

pub async fn submit_attendance_adjustment(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<CreateAttendanceAdjustmentRequest>,
) -> AppResult<(StatusCode, Json<AttendanceAdjustmentRequest>)> {
    let adjustment = crate::services::attendance_adjustment_service::AttendanceAdjustmentService::submit_request(
        state.db.api(),
        workspace_id,
        user_id,
        req,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to submit adjustment".to_string())))?;

    Ok((StatusCode::CREATED, Json(adjustment)))
}

pub async fn approve_attendance_adjustment(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveAttendanceAdjustmentRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::AttendanceAdjustmentsApprove)?;
    crate::services::attendance_adjustment_service::AttendanceAdjustmentService::approve_request(
        state.db.api(),
        ctx.workspace_id,
        user_id,
        id,
        req.status,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to approve adjustment".to_string())))?;

    Ok(StatusCode::OK)
}

pub async fn get_my_attendance_adjustments(
    Path(id): Path<Uuid>,
    UserId(_user_id): UserId,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AttendanceAdjustmentRequest>>> {
    let requests = crate::services::attendance_adjustment_service::AttendanceAdjustmentService::get_my_requests(
        state.db.api(), ctx.workspace_id, id,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to get adjustments".to_string())))?;

    Ok(Json(requests))
}

#[derive(Deserialize)]
pub struct AdjustmentStatusQuery {
    pub status: Option<AdjustmentStatus>,
}

pub async fn get_workspace_attendance_adjustments(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AdjustmentStatusQuery>,
) -> AppResult<Json<Vec<AttendanceAdjustmentRequest>>> {
    let requests = crate::services::attendance_adjustment_service::AttendanceAdjustmentService::get_workspace_requests(
        state.db.api(), workspace_id, query.status
    ).await
    .map_err(|s| AppError::from((s, "Failed to get workspace adjustments".to_string())))?;

    Ok(Json(requests))
}
