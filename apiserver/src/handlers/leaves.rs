use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    AdjustBalanceRequest, ApproveLeaveRequest, AssignLeavePolicyRequest, CreateLeavePolicyRequest,
    EmployeeLeaveBalance, LeaveHistoryItem, LeavePolicyAssignment, LeavePolicyDefinition,
    LeaveRequest, LeaveStatus, PaginatedResponse, PaginationParams, SubmitLeaveRequest,
    UpdateLeavePolicyRequest, WorkspaceLeaveBalance,
};
use crate::repositories::leave_policy_repository;
use crate::services::leave_accrual_service::LeaveAccrualService;
use crate::{AppState, UserId};

// --- Leave Handlers ---

pub async fn submit_leave_request(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<SubmitLeaveRequest>,
) -> AppResult<(StatusCode, Json<LeaveRequest>)> {
    let request = crate::services::leave_service::LeaveService::submit_leave_request(
        &state.db,
        workspace_id,
        user_id,
        req,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to submit leave request".to_string())))?;

    Ok((StatusCode::CREATED, Json(request)))
}

pub async fn approve_leave_request(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveLeaveRequest>,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can approve leave".to_string(),
        ));
    }
    crate::services::leave_service::LeaveService::approve_leave_request(
        &state.db,
        ctx.workspace_id,
        user_id,
        id,
        req,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to approve leave request".to_string())))?;

    Ok(StatusCode::OK)
}

pub async fn get_my_leave_requests(
    Path(id): Path<Uuid>,
    UserId(_user_id): UserId,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<LeaveRequest>>> {
    let requests =
        crate::repositories::leave_request_repository::get_employee_leave_requests(&state.db, id)
            .await?;
    Ok(Json(requests))
}

pub async fn get_workspace_leave_balances(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<WorkspaceLeaveBalance>>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }
    let year = params.year.unwrap_or_else(|| Utc::now().year());
    let balances = crate::services::leave_service::LeaveService::get_workspace_balances(
        &state.db,
        ctx.workspace_id,
        year,
        params,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to get workspace balances".to_string())))?;

    Ok(Json(balances))
}

pub async fn get_employee_leave_balances(
    Path((ws_id, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<EmployeeLeaveBalance>>> {
    let year = Utc::now().year();
    let balances = crate::services::leave_service::LeaveService::get_employee_balances(
        &state.db, ws_id, emp_id, year,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to get employee balances".to_string())))?;

    Ok(Json(balances))
}

#[derive(Deserialize)]
pub struct LeaveStatusQuery {
    pub status: Option<LeaveStatus>,
}

pub async fn get_workspace_leave_requests(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<LeaveStatusQuery>,
) -> AppResult<Json<Vec<LeaveRequest>>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }
    let requests = crate::services::leave_service::LeaveService::get_workspace_leave_requests(
        &state.db,
        ctx.workspace_id,
        query.status,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to get workspace leave requests".to_string())))?;

    Ok(Json(requests))
}

#[derive(Deserialize)]
pub struct LeaveHistoryQuery {
    pub leave_type_id: Option<Uuid>,
}

pub async fn get_employee_leave_history(
    Path((_ws_id, emp_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<LeaveHistoryQuery>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeaveHistoryItem>>> {
    // Allow admins to see anyone's history; non-admins can only access via employee self-service
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }
    let year = Utc::now().year();
    let history = crate::services::leave_service::LeaveService::get_employee_leave_history(
        &state.db,
        ctx.workspace_id,
        emp_id,
        year,
        query.leave_type_id,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to get employee history".to_string())))?;

    Ok(Json(history))
}

pub async fn adjust_leave_balance(
    Path((ws_id, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AdjustBalanceRequest>,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }
    crate::services::leave_service::LeaveService::adjust_balance(&state.db, ws_id, emp_id, req)
        .await
        .map_err(|s| AppError::from((s, "Failed to adjust balance".to_string())))?;

    Ok(StatusCode::OK)
}

pub async fn create_leave_policy(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateLeavePolicyRequest>,
) -> AppResult<(StatusCode, Json<LeavePolicyDefinition>)> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage policies".to_string(),
        ));
    }

    let policy = leave_policy_repository::create_policy_definition(
        &state.db,
        workspace_id,
        &req.name,
        req.description.as_deref(),
        req.leave_type_id,
        req.rule_config,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(policy)))
}

pub async fn update_leave_policy(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateLeavePolicyRequest>,
) -> AppResult<Json<LeavePolicyDefinition>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage policies".to_string(),
        ));
    }

    let policy = leave_policy_repository::update_policy_definition(
        &state.db,
        policy_id,
        workspace_id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.leave_type_id,
        req.rule_config,
    )
    .await?;

    Ok(Json(policy))
}

pub async fn delete_leave_policy(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage policies".to_string(),
        ));
    }

    let rows =
        leave_policy_repository::delete_policy_definition(&state.db, policy_id, workspace_id)
            .await?;

    if rows > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Policy not found".to_string()))
    }
}

pub async fn get_leave_policies(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _ctx: MemberContext,
) -> AppResult<Json<Vec<LeavePolicyDefinition>>> {
    let policies = leave_policy_repository::get_policy_definitions(&state.db, workspace_id).await?;
    Ok(Json(policies))
}

pub async fn assign_leave_policy(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignLeavePolicyRequest>,
) -> AppResult<(StatusCode, Json<LeavePolicyAssignment>)> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can assign policies".to_string(),
        ));
    }

    let assignment = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id,
        assignee_type: req.assignee_type,
        assignee_id: req.assignee_id,
        policy_id: req.policy_id,
        effective_from: req.effective_from,
        effective_to: req.effective_to,
        created_at: chrono::Utc::now(),
    };

    let saved = leave_policy_repository::create_policy_assignment(&state.db, &assignment).await?;
    Ok((StatusCode::CREATED, Json(saved)))
}

#[derive(Debug, Deserialize)]
pub struct TriggerAccrualRequest {
    pub date: String,
}

pub async fn trigger_accrual_run(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<TriggerAccrualRequest>,
) -> AppResult<Json<Vec<crate::models::LeaveAccrualLog>>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can trigger accrual".to_string(),
        ));
    }

    let date = NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Invalid date format YYYY-MM-DD".to_string()))?;

    let logs = LeaveAccrualService::process_workspace_accruals(&state.db, workspace_id, date, true)
        .await?;
    Ok(Json(logs))
}

pub async fn get_leave_policy_assignments(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<crate::models::LeavePolicyAssignmentDetail>>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }
    let assignments = crate::repositories::leave_policy_repository::get_policy_assignments(
        &state.db,
        ctx.workspace_id,
    )
    .await?;
    Ok(Json(assignments))
}
