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

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/adjustments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateAttendanceAdjustmentRequest,
    responses(
        (status = 201, description = "Attendance adjustment created", body = AttendanceAdjustmentRequest)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn submit_attendance_adjustment(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<CreateAttendanceAdjustmentRequest>,
) -> AppResult<(StatusCode, Json<AttendanceAdjustmentRequest>)> {
    let workspace_id = ctx.workspace_id;

    let adjustment = state
        .attendance
        .adjustment_service
        .submit_request(workspace_id, user_id, req)
        .await?;

    state
        .core
        .approval_workflow_service
        .submit_to_workflow(
            &ctx.auth_token,
            "attendance_adjustment".to_string(),
            adjustment.id,
            adjustment.employee_id,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(adjustment)))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Adjustment ID")
    ),
    request_body = ApproveAttendanceAdjustmentRequest,
    responses(
        (status = 200, description = "Adjustment approved/rejected")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn approve_attendance_adjustment(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveAttendanceAdjustmentRequest>,
) -> AppResult<StatusCode> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::AttendanceAdjustmentsApprove)?;

    let opt_instance = state
        .core
        .approval_workflow_service
        .get_workflow_instance_by_entity(&ctx.auth_token, "attendance_adjustment", id)
        .await?;

    if let Some(instance) = opt_instance {
        let action = match req.status {
            AdjustmentStatus::Approved => "approve".to_string(),
            AdjustmentStatus::Rejected => "reject".to_string(),
            AdjustmentStatus::Pending => {
                return Err(crate::errors::AppError::Validation(
                    "Cannot set status to pending".to_string(),
                ))
            }
        };

        state
            .core
            .approval_workflow_service
            .process_approval_action(&ctx.auth_token, user_id, instance.id, action, None, None)
            .await?;
    } else {
        state
            .attendance
            .adjustment_service
            .approve_request(ctx.workspace_id, user_id, id, req.status)
            .await?;
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/v1/employees/{id}/attendance/adjustments",
    params(
        ("id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "My adjustments", body = Vec<AttendanceAdjustmentRequest>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/adjustments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("status" = Option<AdjustmentStatus>, Query, description = "Status filter")
    ),
    responses(
        (status = 200, description = "Workspace adjustments", body = Vec<AttendanceAdjustmentRequest>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_workspace_attendance_adjustments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<AdjustmentStatusQuery>,
) -> AppResult<Json<Vec<AttendanceAdjustmentRequest>>> {
    let workspace_id = ctx.workspace_id;

    let requests = state
        .attendance
        .adjustment_service
        .get_workspace_requests(workspace_id, query.status)
        .await?;

    Ok(Json(requests))
}
