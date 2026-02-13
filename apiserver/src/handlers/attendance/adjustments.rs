use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{
    AdjustmentStatus, ApproveAttendanceAdjustmentRequest, CreateAttendanceAdjustmentRequest,
};
use crate::UserId;
use crate::AppState;

pub async fn submit_attendance_adjustment(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<CreateAttendanceAdjustmentRequest>,
) -> impl IntoResponse {
    println!(
        "SUBMIT_ADJUSTMENT_HANDLER_REACHED: ws_id={workspace_id}, user_id={user_id}"
    );
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::submit_request(
        &state.db,
        workspace_id,
        user_id,
        req,
    )
    .await
    {
        Ok(adjustment) => Ok((StatusCode::CREATED, Json(adjustment))),
        Err(e) => Err(e),
    }
}

pub async fn approve_attendance_adjustment(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveAttendanceAdjustmentRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::approve_request(
        &state.db,
        ctx.workspace_id,
        user_id,
        id,
        req.status,
    )
    .await
    {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(e) => Err(e),
    }
}

pub async fn get_my_attendance_adjustments(
    Path(id): Path<Uuid>,
    UserId(user_id): UserId,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    println!("GET_MY_ADJUSTMENTS_REACHED: id={id}, user_id={user_id}");
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::get_my_requests(
        &state.db, id,
    )
    .await {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
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
) -> impl IntoResponse {
    match crate::services::attendance_adjustment_service::AttendanceAdjustmentService::get_workspace_requests(&state.db, workspace_id, query.status).await {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
}
