use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    AdjustBalanceRequest, ApproveLeaveRequest, AssignLeavePolicyRequest, CreateLeavePolicyRequest,
    EmployeeLeaveBalance, LeaveHistoryItem, LeavePolicyAssignment, LeavePolicyDefinition,
    LeaveRequest, LeaveStatus, PaginatedResponse, PaginationParams, SubmitLeaveRequest,
    UpdateLeavePolicyRequest, WorkspaceLeaveBalance,
};


use crate::{AppState, UserId};

// --- Leave Handlers ---

pub async fn submit_leave_request(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<SubmitLeaveRequest>,
) -> AppResult<(StatusCode, Json<LeaveRequest>)> {
    let request = state.leave_service.submit_leave_request(
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
    let target_dept_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT edh.department_id 
        FROM leave_requests lr
        JOIN employee_department_history edh ON lr.employee_id = edh.employee_id
        WHERE lr.id = $1 
        ORDER BY edh.effective_date DESC 
        LIMIT 1
        "#
    ).bind(id).fetch_optional(state.db.api()).await.unwrap_or(None);

    ctx.require_scoped_permission(Permission::LeaveRequestsApprove, target_dept_id)?;
    state.leave_service.approve_leave_request(
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
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeaveRequest>>> {
    let requests =
        state.leave_request_repo.get_employee_leave_requests(ctx.workspace_id, id)
            .await?;
    Ok(Json(requests))
}

pub async fn get_workspace_leave_balances(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<WorkspaceLeaveBalance>>> {
    ctx.require_permission(Permission::LeaveBalancesRead)?;
    let year = params.year.unwrap_or_else(|| Utc::now().year());
    let balances = state.leave_service.get_workspace_balances(
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
    let balances = state.leave_service.get_employee_balances(
        ws_id, emp_id, year,
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
    ctx.require_permission(Permission::LeaveRequestsRead)?;
    let requests = state.leave_service.get_workspace_leave_requests(
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
    ctx.require_permission(Permission::LeaveBalancesRead)?;
    let year = Utc::now().year();
    let history = state.leave_service.get_employee_leave_history(
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
    ctx.require_permission(Permission::LeaveBalancesAdjust)?;
    state.leave_service.adjust_balance(ws_id, emp_id, req)
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
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let policy = state.leave_policy_repo.create_policy_definition(workspace_id,
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
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let policy = state.leave_policy_repo.update_policy_definition(policy_id,
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
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let rows =
        state.leave_policy_repo.delete_policy_definition(policy_id, workspace_id)
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
    let policies = state.leave_policy_repo.get_policy_definitions(workspace_id).await?;
    Ok(Json(policies))
}

pub async fn assign_leave_policy(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignLeavePolicyRequest>,
) -> AppResult<(StatusCode, Json<LeavePolicyAssignment>)> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;

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

    let saved = state.leave_policy_repo.create_policy_assignment(&assignment).await?;
    Ok((StatusCode::CREATED, Json(saved)))
}

#[derive(Debug, Deserialize)]
pub struct TriggerAccrualRequest {
    pub date: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AccrualJobResponse {
    pub job_id: String,
    pub status: String,
    pub message: String,
}

pub async fn trigger_accrual_run(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<TriggerAccrualRequest>,
) -> AppResult<(StatusCode, Json<AccrualJobResponse>)> {
    ctx.require_permission(Permission::LeaveAccrualRun)?;

    let date = NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Invalid date format YYYY-MM-DD".to_string()))?;

    // Try to enqueue as background job if Redis is available
    if let Some(ref redis) = state.redis {
        use crate::queue::{AccrualPayload, Job, JobMetadata, JobPayload, TaskType, ReportType};
        use crate::queue::JobQueue;

        let queue = JobQueue::new(redis.clone());
        let parent_id = Uuid::new_v4();
        let job = Job::new(
            parent_id,
            0,
            TaskType::LeaveAccrual,
            workspace_id,
            JobPayload::LeaveAccrual(AccrualPayload { date, force: true }),
        );

        // Create job metadata for status polling
        let meta = JobMetadata::new(
            parent_id,
            1, // Single job (not batched)
            workspace_id,
            ctx.user_id,
            ReportType::Summary, // Metadata requires a report type; using Summary as placeholder
        );

        queue.create_job_metadata(&meta).await.map_err(|e| {
            AppError::Internal(format!("Failed to create job metadata: {e}"))
        })?;

        queue.enqueue_jobs(vec![job]).await.map_err(|e| {
            AppError::Internal(format!("Failed to enqueue accrual job: {e}"))
        })?;

        Ok((
            StatusCode::ACCEPTED,
            Json(AccrualJobResponse {
                job_id: parent_id.to_string(),
                status: "queued".to_string(),
                message: "Accrual job enqueued for background processing".to_string(),
            }),
        ))
    } else {
        // Fallback: synchronous processing (no Redis)
        let logs = state.leave_accrual_service.process_workspace_accruals(workspace_id, date, true)
                .await?;

        Ok((
            StatusCode::OK,
            Json(AccrualJobResponse {
                job_id: "".to_string(),
                status: "completed".to_string(),
                message: format!("Processed {} accrual entries synchronously", logs.len()),
            }),
        ))
    }
}

/// GET /workspaces/:ws_id/jobs/:job_id/status
/// Poll the status of a background job.
#[derive(Debug, serde::Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

pub async fn get_job_status(
    Path((_ws_id, job_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _ctx: MemberContext,
) -> AppResult<Json<JobStatusResponse>> {
    let redis = state.redis.as_ref().ok_or_else(|| {
        AppError::Internal("Redis not available".to_string())
    })?;

    use crate::queue::JobQueue;
    let queue = JobQueue::new(redis.clone());

    let meta = queue.get_job_metadata(job_id).await.map_err(|e| {
        AppError::Internal(format!("Failed to get job metadata: {e}"))
    })?;

    match meta {
        Some(m) => Ok(Json(JobStatusResponse {
            job_id: m.job_id.to_string(),
            status: serde_json::to_value(&m.status)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "unknown".to_string()),
            completed_at: m.completed_at.map(|t| t.to_rfc3339()),
            error_message: m.error_message,
        })),
        None => Err(AppError::NotFound(format!("Job {job_id} not found"))),
    }
}

pub async fn get_leave_policy_assignments(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<crate::models::LeavePolicyAssignmentDetail>>> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;
    let assignments = state.leave_policy_repo.get_policy_assignments(ctx.workspace_id,
    )
    .await?;
    Ok(Json(assignments))
}
