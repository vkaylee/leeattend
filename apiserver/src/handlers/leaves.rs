use axum::{
    extract::{Multipart, Path, Query, State},
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
    enums::ImportTargetType, AdjustBalanceRequest, ApproveLeaveRequest, AssignLeavePolicyRequest,
    BehalfAssignment, CreateBehalfAssignmentRequest, CreateLeavePolicyRequest, Employee,
    EmployeeLeaveBalance, LeaveHistoryItem, LeavePolicyAssignment, LeavePolicyAssignmentDetail,
    LeavePolicyDefinition, LeaveRequest, LeaveStatus, PaginatedResponse, PaginationParams,
    SubmitLeaveRequest, UpdateLeavePolicyRequest, WorkspaceLeaveBalance,
};

use crate::{AppState, UserId};

// --- Leave Handlers ---

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedWorkspaceLeaveBalances {
    pub data: Vec<WorkspaceLeaveBalance>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/leave/submit",
    request_body = SubmitLeaveRequest,
    responses(
        (status = 201, description = "Leave request submitted", body = LeaveRequest)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn submit_leave_request(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<SubmitLeaveRequest>,
) -> AppResult<(StatusCode, Json<LeaveRequest>)> {
    // Look up employee's department for scoped permission check
    let target_dept_id =
        crate::handlers::common::current_department_id(&state, &ctx.auth_token, req.employee_id)
            .await;

    let caller_employee_id = state
        .employees()
        .repo()
        .get_employee_id_by_user_id(&ctx.auth_token, ctx.user_id)
        .await?;

    let is_on_behalf = match caller_employee_id {
        Some(emp_id) => emp_id != req.employee_id,
        None => true,
    };

    if is_on_behalf {
        check_leave_behalf_scope(&state, &ctx, req.employee_id).await?;
    } else {
        // Require submit permission scoped to the employee's department
        ctx.require_scoped_permission(Permission::LeaveRequestsSubmit, target_dept_id)?;
    }

    // Auto-approve if submitter also has approve permission for this department
    let can_auto_approve = ctx
        .require_any_scoped_permission(
            &[
                Permission::LeaveRequestsApprove,
                Permission::LeaveApproveL1,
                Permission::LeaveApproveL2,
            ],
            target_dept_id,
        )
        .is_ok();

    let request = state
        .leaves()
        .service()
        .submit_leave_request(ctx.workspace_id, user_id, req, can_auto_approve)
        .await?;

    if !can_auto_approve {
        state
            .core()
            .approval_workflow_service()
            .submit_to_workflow(
                &ctx.auth_token,
                "leave_request".to_string(),
                request.id,
                request.employee_id,
            )
            .await?;
    }

    Ok((StatusCode::CREATED, Json(request)))
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/leave/requests/{id}/approve",
    request_body = ApproveLeaveRequest,
    responses(
        (status = 200, description = "Leave request approved")
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Leave request ID")
    )
)]
pub async fn approve_leave_request(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveLeaveRequest>,
) -> AppResult<StatusCode> {
    let (_, id) = params;
    let target_dept_id = state
        .leaves()
        .request_repo()
        .get_employee_department_for_request(id, &ctx.auth_token)
        .await?;

    ctx.require_any_scoped_permission(
        &[
            Permission::LeaveRequestsApprove,
            Permission::LeaveApproveL1,
            Permission::LeaveApproveL2,
        ],
        target_dept_id,
    )?;
    state
        .leaves()
        .service()
        .approve_leave_request(ctx.workspace_id, user_id, id, req)
        .await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/employees/{id}/leave/requests",
    responses(
        (status = 200, description = "List of leave requests", body = [LeaveRequest])
    ),
    params(
        ("id" = Uuid, Path, description = "Employee ID")
    )
)]
pub async fn get_my_leave_requests(
    Path(id): Path<Uuid>,
    UserId(_user_id): UserId,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeaveRequest>>> {
    let requests = state
        .leaves()
        .request_repo()
        .get_employee_leave_requests(&ctx.auth_token, id)
        .await?;
    Ok(Json(requests))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/leave/balances",
    responses(
        (status = 200, description = "Workspace leave balances", body = PaginatedWorkspaceLeaveBalances)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        PaginationParams
    )
)]
pub async fn get_workspace_leave_balances(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<WorkspaceLeaveBalance>>> {
    ctx.require_permission(Permission::LeaveBalancesRead)?;
    let year = params.year.unwrap_or_else(|| Utc::now().year());
    let balances = state
        .leaves()
        .service()
        .get_workspace_balances(ctx.workspace_id, year, params)
        .await?;

    Ok(Json(balances))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/employees/{id}/leave/balances",
    responses(
        (status = 200, description = "Employee leave balances", body = [EmployeeLeaveBalance])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID")
    )
)]
pub async fn get_employee_leave_balances(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeeLeaveBalance>>> {
    let (_, emp_id) = params;
    let year = Utc::now().year();
    let balances = state
        .leaves()
        .service()
        .get_employee_balances(ctx.workspace_id, emp_id, year)
        .await?;

    Ok(Json(balances))
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct LeaveStatusQuery {
    pub status: Option<LeaveStatus>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/leave/requests",
    responses(
        (status = 200, description = "Workspace leave requests", body = [LeaveRequest])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        LeaveStatusQuery
    )
)]
pub async fn get_workspace_leave_requests(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<LeaveStatusQuery>,
) -> AppResult<Json<Vec<LeaveRequest>>> {
    ctx.require_permission(Permission::LeaveRequestsRead)?;
    let mut requests = state
        .leaves()
        .service()
        .get_workspace_leave_requests(ctx.workspace_id, query.status)
        .await?;

    // When filtering by Approved, also include PreApproved requests
    if query.status == Some(LeaveStatus::Approved) {
        let pre_approved = state
            .leaves()
            .service()
            .get_workspace_leave_requests(ctx.workspace_id, Some(LeaveStatus::PreApproved))
            .await?;
        requests.extend(pre_approved);
        requests.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    }

    Ok(Json(requests))
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct LeaveHistoryQuery {
    pub leave_type_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/employees/{id}/leave/history",
    responses(
        (status = 200, description = "Employee leave history", body = [LeaveHistoryItem])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
        LeaveHistoryQuery
    )
)]
pub async fn get_employee_leave_history(
    Path(params): Path<(Uuid, Uuid)>,
    Query(query): Query<LeaveHistoryQuery>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeaveHistoryItem>>> {
    let (_, emp_id) = params;
    // Allow admins to see anyone's history; non-admins can only access via employee self-service
    ctx.require_permission(Permission::LeaveBalancesRead)?;
    let year = Utc::now().year();
    let history = state
        .leaves()
        .service()
        .get_employee_leave_history(ctx.workspace_id, emp_id, year, query.leave_type_id)
        .await?;

    Ok(Json(history))
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/employees/{id}/leave/adjust",
    request_body = AdjustBalanceRequest,
    responses(
        (status = 200, description = "Leave balance adjusted")
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID")
    )
)]
pub async fn adjust_leave_balance(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AdjustBalanceRequest>,
) -> AppResult<StatusCode> {
    let (_, emp_id) = params;
    ctx.require_permission(Permission::LeaveBalancesAdjust)?;
    state
        .leaves()
        .service()
        .adjust_balance(ctx.workspace_id, emp_id, req)
        .await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/leave/policies",
    request_body = CreateLeavePolicyRequest,
    responses(
        (status = 201, description = "Leave policy created", body = LeavePolicyDefinition)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_leave_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateLeavePolicyRequest>,
) -> AppResult<(StatusCode, Json<LeavePolicyDefinition>)> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let policy = state
        .leaves()
        .policy_repo()
        .create_policy_definition(
            ctx.workspace_id,
            &req.name,
            req.description.as_deref(),
            req.leave_type_id,
            req.rule_config,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(policy)))
}

#[utoipa::path(
    patch,
    path = "/workspaces/{ws_id}/leave/policies/{policy_id}",
    request_body = UpdateLeavePolicyRequest,
    responses(
        (status = 200, description = "Leave policy updated", body = LeavePolicyDefinition)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("policy_id" = Uuid, Path, description = "Policy ID")
    )
)]
pub async fn update_leave_policy(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateLeavePolicyRequest>,
) -> AppResult<Json<LeavePolicyDefinition>> {
    let (_, policy_id) = params;
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let policy = state
        .leaves()
        .policy_repo()
        .update_policy_definition(
            policy_id,
            ctx.workspace_id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.leave_type_id,
            req.rule_config,
        )
        .await?;

    Ok(Json(policy))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/leave/policies/{policy_id}",
    responses(
        (status = 204, description = "Leave policy deleted")
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("policy_id" = Uuid, Path, description = "Policy ID")
    )
)]
pub async fn delete_leave_policy(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_, policy_id) = params;
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let rows = state
        .leaves()
        .policy_repo()
        .delete_policy_definition(policy_id, ctx.workspace_id)
        .await?;

    if rows > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Policy not found".to_string()))
    }
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/leave/policies",
    responses(
        (status = 200, description = "Workspace leave policies", body = [LeavePolicyDefinition])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn get_leave_policies(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeavePolicyDefinition>>> {
    let policies = state
        .leaves()
        .policy_repo()
        .get_policy_definitions(ctx.workspace_id)
        .await?;
    Ok(Json(policies))
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/leave/policies/assign",
    request_body = AssignLeavePolicyRequest,
    responses(
        (status = 201, description = "Leave policy assigned", body = LeavePolicyAssignment)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn assign_leave_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignLeavePolicyRequest>,
) -> AppResult<(StatusCode, Json<LeavePolicyAssignment>)> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let assignment = LeavePolicyAssignment {
        id: crate::utils::id::new_id(),
        workspace_id: ctx.workspace_id,
        assignee_type: req.assignee_type,
        assignee_id: req.assignee_id,
        policy_id: req.policy_id,
        effective_from: req.effective_from,
        effective_to: req.effective_to,
        created_at: chrono::Utc::now(),
    };

    let saved = state
        .leaves()
        .policy_repo()
        .create_policy_assignment(ctx.workspace_id, &assignment)
        .await?;
    Ok((StatusCode::CREATED, Json(saved)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TriggerAccrualRequest {
    pub date: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AccrualJobResponse {
    pub job_id: String,
    pub status: String,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/leave/accrual/run",
    request_body = TriggerAccrualRequest,
    responses(
        (status = 202, description = "Accrual run triggered", body = AccrualJobResponse)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn trigger_accrual_run(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<TriggerAccrualRequest>,
) -> AppResult<(StatusCode, Json<AccrualJobResponse>)> {
    ctx.require_permission(Permission::LeaveAccrualRun)?;

    let date = NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d")
        .map_err(|_e| AppError::Validation("Invalid date format YYYY-MM-DD".to_string()))?;

    use crate::queue::{AccrualPayload, Job, JobMetadata, JobPayload, TaskType};

    let queue = state.infra().job_queue()?;
    let parent_id = crate::utils::id::new_id();
    let job = Job::new(
        parent_id,
        0,
        TaskType::LeaveAccrual,
        ctx.workspace_id,
        JobPayload::LeaveAccrual(AccrualPayload { date, force: true }),
    );

    // Create job metadata for status polling
    let meta = JobMetadata::new(
        parent_id,
        1, // Single job (not batched)
        ctx.workspace_id,
        ctx.user_id,
        Some(TaskType::LeaveAccrual),
        None, // No report type for leave accrual
        Some(serde_json::json!({
            "date": payload.date,
            "force": true
        })),
    );

    queue.create_job_metadata(&meta).await.map_err(|e| {
        AppError::BackgroundJobError(format!("Failed to create job job metadata/queue: {e}"))
    })?;

    queue.enqueue_jobs(vec![job]).await.map_err(|e| {
        AppError::BackgroundJobError(format!("Failed to enqueue accrual job metadata/queue: {e}"))
    })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(AccrualJobResponse {
            job_id: parent_id.to_string(),
            status: "queued".to_string(),
            message: "Accrual job enqueued for background processing".to_string(),
        }),
    ))
}

/// GET /workspaces/:ws_id/jobs/:job_id/status
/// Poll the status of a background job.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/jobs/{job_id}/status",
    operation_id = "get_leave_job_status",
    responses(
        (status = 200, description = "Get background job status", body = JobStatusResponse)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("job_id" = Uuid, Path, description = "Job ID")
    )
)]
pub async fn get_job_status(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _ctx: MemberContext,
) -> AppResult<Json<JobStatusResponse>> {
    let (_, job_id) = params;
    let queue = state.infra().job_queue()?;

    let meta = queue.get_job_metadata(job_id).await.map_err(|e| {
        AppError::BackgroundJobError(format!("Failed to get job job metadata/queue: {e}"))
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

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/leave/policies/assignments",
    responses(
        (status = 200, description = "Workspace leave policy assignments", body = [LeavePolicyAssignmentDetail])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn get_leave_policy_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<crate::models::LeavePolicyAssignmentDetail>>> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;
    let assignments = state
        .leaves()
        .policy_repo()
        .get_policy_assignments(ctx.workspace_id)
        .await?;
    Ok(Json(assignments))
}

// ========================
// Leave Import Handlers
// ========================

/// POST /workspaces/:ws_id/leave/upload-excel
/// Upload an Excel file for bulk leave request import.
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/leave/upload-excel",
    responses(
        (status = 200, description = "Leave Excel uploaded", body = serde_json::Value)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn upload_leave_excel(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_permission(Permission::LeaveRequestsSubmit)?;

    let path = crate::utils::upload::extract_uploaded_file(
        &mut multipart,
        "file",
        crate::utils::upload::MAX_UPLOAD_FILE_SIZE,
        "imports",
    )
    .await?;
    let path = path.to_string_lossy().to_string();

    // Check auto-approve permission
    let auto_approve = ctx.has_permission(Permission::LeaveRequestsApprove)
        || ctx.has_permission(Permission::LeaveApproveL1)
        || ctx.has_permission(Permission::LeaveApproveL2);

    // Create import job record in DB
    let import_job_id = crate::utils::id::new_id();
    let import_job = crate::models::AttendanceImportJob {
        id: import_job_id,
        workspace_id: ctx.workspace_id,
        user_id,
        status: crate::models::ImportJobStatus::Pending,
        total_rows: 0,
        processed_rows: 0,
        file_path: Some(path.clone()),
        error_log: None,
        target_type: ImportTargetType::LeaveImport,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state
        .attendance()
        .import_repo()
        .create_job(&ctx.auth_token, &import_job)
        .await
        .map_err(|e| {
            AppError::BackgroundJobError(format!("Failed to create import job metadata/queue: {e}"))
        })?;

    use crate::queue::{Job, JobPayload, LeaveImportPayload, TaskType};

    let payload = LeaveImportPayload {
        file_path: path,
        user_id,
        auto_approve,
        import_job_id,
    };

    let job = Job::new(
        import_job_id,
        0,
        TaskType::LeaveImport,
        ctx.workspace_id,
        JobPayload::LeaveImport(payload),
    );

    let queue = state.infra().job_queue()?;
    queue.enqueue_jobs(vec![job]).await.map_err(|e| {
        AppError::BackgroundJobError(format!("Failed to enqueue leave import job: {e}"))
    })?;

    Ok(Json(serde_json::json!({
        "job_id": import_job_id.to_string(),
        "status": "enqueued",
    })))
}

/// GET /workspaces/:ws_id/leave/import/latest
/// Get the latest leave import job for the workspace (for page resume).
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/leave/import/latest",
    responses(
        (status = 200, description = "Latest leave import job", body = serde_json::Value)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn get_latest_leave_import(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_permission(Permission::LeaveRequestsSubmit)?;

    let repo = state.attendance().import_repo();

    match repo
        .get_latest_job(&ctx.auth_token, Some(ImportTargetType::LeaveImport))
        .await
    {
        Ok(Some(job)) => Ok(Json(serde_json::json!({
            "id": job.id.to_string(),
            "status": serde_json::to_value(job.status)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "unknown".to_string()),
            "total_rows": job.total_rows,
            "processed_rows": job.processed_rows,
            "error_log": job.error_log,
            "created_at": job.created_at.to_rfc3339(),
            "updated_at": job.updated_at.to_rfc3339(),
        }))),
        Ok(None) => Ok(Json(serde_json::json!(null))),
        Err(e) => Err(AppError::Internal(format!(
            "Failed to get latest import job: {e}"
        ))),
    }
}

/// GET /workspaces/:ws_id/leave/import/:job_id/errors
/// Download the error Excel file for a completed import job.
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/leave/import/{job_id}/errors",
    responses(
        (status = 200, description = "Download leave import errors file", body = [u8])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("job_id" = Uuid, Path, description = "Job ID")
    )
)]
pub async fn download_leave_import_errors(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<axum::response::Response> {
    let (_, job_id) = params;
    ctx.require_permission(Permission::LeaveRequestsSubmit)?;

    let repo = state.attendance().import_repo();

    let job = repo
        .get_job_by_id(&ctx.auth_token, job_id)
        .await
        .map_err(|e| AppError::ServiceUnavailable(format!("DB error: {e}")))?
        .ok_or_else(|| AppError::NotFound("Import job not found".to_string()))?;

    // Extract error file path from error_log JSON
    let error_file_path = job
        .error_log
        .as_ref()
        .and_then(|log| serde_json::from_str::<serde_json::Value>(log).ok())
        .and_then(|v| {
            v.get("error_file_path")
                .and_then(|p| p.as_str().map(String::from))
        })
        .ok_or_else(|| AppError::NotFound("No error file available for this import".to_string()))?;

    let bytes = tokio::fs::read(&error_file_path)
        .await
        .map_err(|e| AppError::NotFound(format!("Error file not found: {e}")))?;

    use axum::http::header;

    axum::response::Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"leave_import_{job_id}_errors.xlsx\""),
        )
        .body(axum::body::Body::from(bytes))
        .map_err(|e| AppError::TemplateError(format!("Failed to build response: {e}")))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct DownloadTemplateQuery {
    pub lang: Option<String>,
}

/// GET /workspaces/:ws_id/leave/upload-template
/// Download a dynamic template Excel with leave type codes for this workspace.
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/leave/upload-template",
    responses(
        (status = 200, description = "Download leave import template", body = [u8])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        DownloadTemplateQuery
    )
)]
pub async fn download_leave_import_template(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<DownloadTemplateQuery>,
    ctx: MemberContext,
) -> AppResult<axum::response::Response> {
    ctx.require_permission(Permission::LeaveRequestsSubmit)?;

    let leave_types = state
        .leaves()
        .request_repo()
        .get_leave_types(&ctx.auth_token)
        .await?;

    let lang = query.lang.as_deref().unwrap_or("en");
    let bytes = crate::services::leave_import_service::LeaveImportService::generate_template(
        &leave_types,
        lang,
    )
    .map_err(|e| AppError::TemplateError(format!("Failed to generate template: {e}")))?;

    use axum::http::header;

    axum::response::Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"leave_import_template.xlsx\"",
        )
        .body(axum::body::Body::from(bytes))
        .map_err(|e| AppError::TemplateError(format!("Failed to build response: {e}")))
}

async fn check_leave_behalf_scope(
    state: &AppState,
    ctx: &MemberContext,
    target_employee_id: Uuid,
) -> AppResult<()> {
    crate::handlers::common::check_behalf_scope(
        state,
        ctx,
        &[
            Permission::LeaveRequestsApprove,
            Permission::LeaveApproveL1,
            Permission::LeaveApproveL2,
            Permission::LeaveRequestsSubmitOnBehalf,
        ],
        "leave_request",
        target_employee_id,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/leave/behalf-assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateBehalfAssignmentRequest,
    responses(
        (status = 200, description = "Leave assignment created", body = BehalfAssignment)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_leave_behalf_assignment(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<crate::models::CreateBehalfAssignmentRequest>,
) -> AppResult<Json<crate::models::BehalfAssignment>> {
    let result = crate::handlers::common::create_behalf_assignment_handler(
        &state,
        &ctx,
        &req,
        Permission::LeavePoliciesManage,
        "leave_request",
    )
    .await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/leave/behalf-assignments/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 200, description = "Leave assignment deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_leave_behalf_assignment(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_ws_id, id) = params;
    crate::handlers::common::delete_behalf_assignment_handler(
        &state,
        &ctx,
        id,
        Permission::LeavePoliciesManage,
    )
    .await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/leave/behalf-assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of leave assignments", body = Vec<BehalfAssignment>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_leave_behalf_assignments(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<crate::models::BehalfAssignment>>> {
    let result = crate::handlers::common::list_behalf_assignments_handler(
        &state,
        &ctx,
        Permission::LeavePoliciesManage,
        "leave_request",
    )
    .await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/leave/behalf-assignments/my-targets",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Target employees the current user can act on behalf of", body = Vec<Employee>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_leave_behalf_targets(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<crate::models::Employee>>> {
    let result =
        crate::handlers::common::get_my_behalf_targets_handler(&state, &ctx, "leave_request")
            .await?;
    Ok(Json(result))
}
