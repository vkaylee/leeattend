use axum::{
    extract::{State, Path, Query},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Datelike, NaiveDate};
use serde::Deserialize;

use crate::{AppState, UserId};
use crate::handlers::common::MemberContext;
use crate::models::{
    SubmitLeaveRequest, ApproveLeaveRequest, LeaveRequest, 
    WorkspaceLeaveBalance, EmployeeLeaveBalance, AdjustBalanceRequest,
    CreateLeavePolicyRequest, UpdateLeavePolicyRequest, AssignLeavePolicyRequest, LeavePolicyAssignment,
    LeaveHistoryItem
};
use crate::repositories::leave_policy_repository;
use crate::services::leave_accrual_service::LeaveAccrualService;

// --- Leave Handlers ---

pub async fn submit_leave_request(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<SubmitLeaveRequest>,
) -> impl IntoResponse {
    match crate::services::leave_service::LeaveService::submit_leave_request(
        &state.db,
        workspace_id,
        user_id,
        req,
    )
    .await
    {
        Ok(request) => (StatusCode::CREATED, Json(request)).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn approve_leave_request(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveLeaveRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    match crate::services::leave_service::LeaveService::approve_leave_request(
        &state.db,
        ctx.workspace_id,
        user_id,
        id,
        req,
    )
    .await
    {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(status) => Err(status),
    }
}

pub async fn get_my_leave_requests(
    Path(id): Path<Uuid>,
    UserId(_user_id): UserId,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LeaveRequest>>, StatusCode> {
    // For "my" requests without a specific workspace context in URL, 
    // we might need to fetch them differently, but for now let's just 
    // allow fetching by employee_id if we have a valid user.
    // In a real scenario, we should verify the user_id matches the employee's user_id.
    
    // Temporary hack: use a special internal method or just skip workspace check if we trust the employee_id
    crate::repositories::leave_request_repository::get_employee_leave_requests(&state.db, id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_workspace_leave_balances(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<Json<Vec<WorkspaceLeaveBalance>>, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    let year = Utc::now().year();
    match crate::services::leave_service::LeaveService::get_workspace_balances(&state.db, ctx.workspace_id, year).await {
        Ok(balances) => Ok(Json(balances)),
        Err(status) => Err(status),
    }
}

pub async fn get_employee_leave_balances(
    Path((ws_id, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<EmployeeLeaveBalance>>, StatusCode> {
    let year = Utc::now().year();
    match crate::services::leave_service::LeaveService::get_employee_balances(&state.db, ws_id, emp_id, year).await {
        Ok(balances) => Ok(Json(balances)),
        Err(status) => Err(status),
    }
}

#[derive(Deserialize)]
pub struct LeaveStatusQuery {
    pub status: Option<String>,
}

pub async fn get_workspace_leave_requests(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<LeaveStatusQuery>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }
    match crate::services::leave_service::LeaveService::get_workspace_leave_requests(
        &state.db,
        ctx.workspace_id,
        query.status,
    )
    .await
    {
        Ok(requests) => Json(requests).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_employee_leave_history(
    Path((ws_id, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LeaveHistoryItem>>, StatusCode> {
    let year = Utc::now().year();
    match crate::services::leave_service::LeaveService::get_employee_leave_history(
        &state.db,
        ws_id,
        emp_id,
        year,
    )
    .await {
        Ok(history) => Ok(Json(history)),
        Err(status) => Err(status),
    }
}

pub async fn adjust_leave_balance(
    Path((ws_id, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AdjustBalanceRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return StatusCode::FORBIDDEN.into_response();
    }
    match crate::services::leave_service::LeaveService::adjust_balance(
        &state.db,
        ws_id,
        emp_id,
        req,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn create_leave_policy(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateLeavePolicyRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage policies").into_response();
    }

    match leave_policy_repository::create_policy_definition(
        &state.db,
        workspace_id,
        &req.name,
        req.description.as_deref(),
        &req.leave_type,
        req.rule_config,
    )
    .await
    {
        Ok(policy) => (StatusCode::CREATED, Json(policy)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_leave_policy(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateLeavePolicyRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage policies").into_response();
    }

    match leave_policy_repository::update_policy_definition(
        &state.db,
        policy_id,
        workspace_id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.leave_type.as_deref(),
        req.rule_config,
    )
    .await
    {
        Ok(policy) => (StatusCode::OK, Json(policy)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_leave_policy(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage policies").into_response();
    }

    match leave_policy_repository::delete_policy_definition(&state.db, policy_id, workspace_id).await {
        Ok(rows) if rows > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_leave_policies(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    _ctx: MemberContext,
) -> impl IntoResponse {
    match leave_policy_repository::get_policy_definitions(&state.db, workspace_id).await {
        Ok(policies) => Json(policies).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn assign_leave_policy(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignLeavePolicyRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can assign policies").into_response();
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

    match leave_policy_repository::create_policy_assignment(&state.db, &assignment).await {
        Ok(saved) => (StatusCode::CREATED, Json(saved)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn trigger_accrual_run(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can trigger accrual").into_response();
    }

    let date_str = payload["date"].as_str().unwrap_or_default();
    let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid date format YYYY-MM-DD").into_response(),
    };

    match LeaveAccrualService::process_workspace_accruals(&state.db, workspace_id, date).await {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
