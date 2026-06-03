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
    let target_employee_id = req.employee_id;

    // Behalf scope enforcement: check if caller is submitting for another employee
    let caller_employee_id = state
        .employees
        .repo
        .get_employee_id_by_user_id(&ctx.auth_token, ctx.user_id)
        .await?;

    if let Some(caller_id) = caller_employee_id {
        if caller_id != target_employee_id {
            check_adjustment_behalf_scope(&state, &ctx, caller_id, target_employee_id).await?;
        }
    }

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

/// Check that the caller has permission to submit attendance adjustments on behalf
/// of the target employee. Validates in order:
/// 1. Caller must have `AttendanceAdjustmentsSubmitOnBehalf` permission
/// 2. Caller has a specific behalf_assignment for the target
/// 3. Target belongs to one of the caller's managed departments
async fn check_adjustment_behalf_scope(
    state: &AppState,
    ctx: &MemberContext,
    caller_employee_id: Uuid,
    target_employee_id: Uuid,
) -> AppResult<()> {
    if !ctx.has_permission(Permission::AttendanceAdjustmentsSubmitOnBehalf) {
        return Err(AppError::Forbidden(
            "Missing permission: attendance:adjustments:submit_on_behalf".to_string(),
        ));
    }

    // Check for a specific behalf assignment
    let target_dept_id =
        crate::handlers::common::current_department_id(state, &ctx.auth_token, target_employee_id)
            .await;

    let is_assigned = state
        .attendance
        .behalf_assignment_repo
        .check_behalf_permission(
            ctx.workspace_id,
            caller_employee_id,
            target_employee_id,
            target_dept_id,
            "attendance_adjustment",
        )
        .await?;

    if is_assigned {
        return Ok(());
    }

    // Fall back to managed department check
    if let Some(allowed_depts) = &ctx.managed_department_ids {
        if let Some(dept_id) = target_dept_id {
            if allowed_depts.contains(&dept_id) {
                return Ok(());
            }
        }
    }

    Err(AppError::Forbidden(
        "Permission denied: Target is outside your adjustment on behalf scope.".to_string(),
    ))
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

// ── Behalf Assignments for Attendance Adjustments ────────────────────

use crate::errors::AppError;
use crate::models::{BehalfAssignment, CreateBehalfAssignmentRequest};

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateBehalfAssignmentRequest,
    responses(
        (status = 200, description = "Assignment created", body = BehalfAssignment)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_adjustment_behalf_assignment(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreateBehalfAssignmentRequest>,
) -> AppResult<Json<BehalfAssignment>> {
    ctx.require_permission(Permission::AttendanceAdjustmentsManageBehalf)?;

    let result = state
        .attendance
        .behalf_assignment_repo
        .create_assignment(
            ctx.workspace_id,
            req.assignee_employee_id,
            req.target_employee_id,
            req.target_department_id,
            "attendance_adjustment",
        )
        .await?;

    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 200, description = "Assignment deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_adjustment_behalf_assignment(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_ws_id, id) = params;
    ctx.require_permission(Permission::AttendanceAdjustmentsManageBehalf)?;

    let affected = state
        .attendance
        .behalf_assignment_repo
        .delete_assignment(ctx.workspace_id, id)
        .await?;

    if affected == 0 {
        return Err(AppError::NotFound("Assignment not found".to_string()));
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of assignments", body = Vec<BehalfAssignment>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_adjustment_behalf_assignments(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<BehalfAssignment>>> {
    ctx.require_permission(Permission::AttendanceAdjustmentsManageBehalf)?;

    let result = state
        .attendance
        .behalf_assignment_repo
        .list_all_assignments(ctx.workspace_id, "attendance_adjustment")
        .await?;

    Ok(Json(result))
}

