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
    AdjustBalanceRequest, ApproveLeaveRequest, AssignLeavePolicyRequest, CreateLeavePolicyRequest,
    EmployeeLeaveBalance, LeaveHistoryItem, LeavePolicyAssignment, LeavePolicyDefinition,
    LeaveRequest, LeaveStatus, PaginatedResponse, PaginationParams, SubmitLeaveRequest,
    UpdateLeavePolicyRequest, WorkspaceLeaveBalance,
};

use crate::{AppState, UserId};

// --- Leave Handlers ---

pub async fn submit_leave_request(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<SubmitLeaveRequest>,
) -> AppResult<(StatusCode, Json<LeaveRequest>)> {
    // Look up employee's department for scoped permission check
    let target_dept_id =
        crate::handlers::common::current_department_id(&state, ctx.workspace_id, req.employee_id)
            .await;

    // Require submit permission scoped to the employee's department
    ctx.require_scoped_permission(Permission::LeaveRequestsSubmit, target_dept_id)?;

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
        .leaves
        .service
        .submit_leave_request(ctx.workspace_id, user_id, req, can_auto_approve)
        .await?;

    Ok((StatusCode::CREATED, Json(request)))
}

pub async fn approve_leave_request(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveLeaveRequest>,
) -> AppResult<StatusCode> {
    let target_dept_id = state
        .leaves
        .request_repo
        .get_employee_department_for_request(id, ctx.workspace_id)
        .await
        .map_err(AppError::Sqlx)?;

    ctx.require_any_scoped_permission(
        &[
            Permission::LeaveRequestsApprove,
            Permission::LeaveApproveL1,
            Permission::LeaveApproveL2,
        ],
        target_dept_id,
    )?;
    state
        .leaves
        .service
        .approve_leave_request(ctx.workspace_id, user_id, id, req)
        .await?;

    Ok(StatusCode::OK)
}

pub async fn get_my_leave_requests(
    Path(id): Path<Uuid>,
    UserId(_user_id): UserId,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeaveRequest>>> {
    let requests = state
        .leaves
        .request_repo
        .get_employee_leave_requests(ctx.workspace_id, id)
        .await?;
    Ok(Json(requests))
}

pub async fn get_workspace_leave_balances(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<WorkspaceLeaveBalance>>> {
    ctx.require_permission(Permission::LeaveBalancesRead)?;
    let year = params.year.unwrap_or_else(|| Utc::now().year());
    let balances = state
        .leaves
        .service
        .get_workspace_balances(ctx.workspace_id, year, params)
        .await?;

    Ok(Json(balances))
}

pub async fn get_employee_leave_balances(
    Path((_, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeeLeaveBalance>>> {
    let year = Utc::now().year();
    let balances = state
        .leaves
        .service
        .get_employee_balances(ctx.workspace_id, emp_id, year)
        .await?;

    Ok(Json(balances))
}

#[derive(Deserialize)]
pub struct LeaveStatusQuery {
    pub status: Option<LeaveStatus>,
}

pub async fn get_workspace_leave_requests(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<LeaveStatusQuery>,
) -> AppResult<Json<Vec<LeaveRequest>>> {
    ctx.require_permission(Permission::LeaveRequestsRead)?;
    let mut requests = state
        .leaves
        .service
        .get_workspace_leave_requests(ctx.workspace_id, query.status)
        .await?;

    // When filtering by Approved, also include PreApproved requests
    if query.status == Some(LeaveStatus::Approved) {
        let pre_approved = state
            .leaves
            .service
            .get_workspace_leave_requests(ctx.workspace_id, Some(LeaveStatus::PreApproved))
            .await?;
        requests.extend(pre_approved);
        requests.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    }

    Ok(Json(requests))
}

#[derive(Deserialize)]
pub struct LeaveHistoryQuery {
    pub leave_type_id: Option<Uuid>,
}

pub async fn get_employee_leave_history(
    Path((_, emp_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<LeaveHistoryQuery>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeaveHistoryItem>>> {
    // Allow admins to see anyone's history; non-admins can only access via employee self-service
    ctx.require_permission(Permission::LeaveBalancesRead)?;
    let year = Utc::now().year();
    let history = state
        .leaves
        .service
        .get_employee_leave_history(ctx.workspace_id, emp_id, year, query.leave_type_id)
        .await?;

    Ok(Json(history))
}

pub async fn adjust_leave_balance(
    Path((_, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AdjustBalanceRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::LeaveBalancesAdjust)?;
    state
        .leaves
        .service
        .adjust_balance(ctx.workspace_id, emp_id, req)
        .await?;

    Ok(StatusCode::OK)
}

pub async fn create_leave_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateLeavePolicyRequest>,
) -> AppResult<(StatusCode, Json<LeavePolicyDefinition>)> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let policy = state
        .leaves
        .policy_repo
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

pub async fn update_leave_policy(
    State(state): State<Arc<AppState>>,
    Path((_, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateLeavePolicyRequest>,
) -> AppResult<Json<LeavePolicyDefinition>> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let policy = state
        .leaves
        .policy_repo
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

pub async fn delete_leave_policy(
    State(state): State<Arc<AppState>>,
    Path((_, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let rows = state
        .leaves
        .policy_repo
        .delete_policy_definition(policy_id, ctx.workspace_id)
        .await?;

    if rows > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Policy not found".to_string()))
    }
}

pub async fn get_leave_policies(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeavePolicyDefinition>>> {
    let policies = state
        .leaves
        .policy_repo
        .get_policy_definitions(ctx.workspace_id)
        .await?;
    Ok(Json(policies))
}

pub async fn assign_leave_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignLeavePolicyRequest>,
) -> AppResult<(StatusCode, Json<LeavePolicyAssignment>)> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;

    let assignment = LeavePolicyAssignment {
        id: Uuid::new_v4(),
        workspace_id: ctx.workspace_id,
        assignee_type: req.assignee_type,
        assignee_id: req.assignee_id,
        policy_id: req.policy_id,
        effective_from: req.effective_from,
        effective_to: req.effective_to,
        created_at: chrono::Utc::now(),
    };

    let saved = state
        .leaves
        .policy_repo
        .create_policy_assignment(&assignment)
        .await?;
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
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<TriggerAccrualRequest>,
) -> AppResult<(StatusCode, Json<AccrualJobResponse>)> {
    ctx.require_permission(Permission::LeaveAccrualRun)?;

    let date = NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Invalid date format YYYY-MM-DD".to_string()))?;

    // Require Redis for background processing
    let redis = state.infra.redis.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailable(
            "Leave accrual requires Redis for background processing".to_string(),
        )
    })?;

    use crate::queue::JobQueue;
    use crate::queue::{AccrualPayload, Job, JobMetadata, JobPayload, TaskType};

    let queue = JobQueue::new(redis.clone());
    let parent_id = Uuid::new_v4();
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
        None, // No report type for leave accrual
    );

    queue
        .create_job_metadata(&meta)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create job metadata: {e}")))?;

    queue
        .enqueue_jobs(vec![job])
        .await
        .map_err(|e| AppError::Internal(format!("Failed to enqueue accrual job: {e}")))?;

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
#[derive(Debug, serde::Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

pub async fn get_job_status(
    Path((_, job_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    _ctx: MemberContext,
) -> AppResult<Json<JobStatusResponse>> {
    let redis = state
        .infra
        .redis
        .as_ref()
        .ok_or_else(|| AppError::Internal("Redis not available".to_string()))?;

    use crate::queue::JobQueue;
    let queue = JobQueue::new(redis.clone());

    let meta = queue
        .get_job_metadata(job_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get job metadata: {e}")))?;

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
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<crate::models::LeavePolicyAssignmentDetail>>> {
    ctx.require_permission(Permission::LeavePoliciesManage)?;
    let assignments = state
        .leaves
        .policy_repo
        .get_policy_assignments(ctx.workspace_id)
        .await?;
    Ok(Json(assignments))
}

// ========================
// Leave Import Handlers
// ========================

/// POST /workspaces/:ws_id/leave/upload-excel
/// Upload an Excel file for bulk leave request import.
pub async fn upload_leave_excel(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_permission(Permission::LeaveRequestsSubmit)?;

    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

    let mut file_path = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("Failed to read file bytes: {e}")))?;

            if data.len() > MAX_FILE_SIZE {
                return Err(AppError::Validation(
                    "File too large (max 10MB)".to_string(),
                ));
            }

            let dir = "storage/imports";
            tokio::fs::create_dir_all(dir).await?;
            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);
            tokio::fs::write(&path, data).await?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;

    // Check auto-approve permission
    let auto_approve = ctx.has_permission(Permission::LeaveRequestsApprove)
        || ctx.has_permission(Permission::LeaveApproveL1)
        || ctx.has_permission(Permission::LeaveApproveL2);

    // Create import job record in DB
    let import_job_id = Uuid::new_v4();
    let import_job = crate::models::AttendanceImportJob {
        id: import_job_id,
        workspace_id: ctx.workspace_id,
        user_id,
        status: crate::models::ImportJobStatus::Pending,
        total_rows: 0,
        processed_rows: 0,
        file_path: Some(path.clone()),
        error_log: None,
        target_type: "leave_import".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    {
        use crate::repositories::import_job_repository::{
            ImportJobRepository, SqlxImportJobRepository,
        };
        let import_repo = SqlxImportJobRepository::new(state.infra.db.api().clone());
        import_repo
            .create_job(&import_job)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create import job: {e}")))?;
    }

    // Require Redis for background processing
    let redis = state.infra.redis.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailable(
            "Leave import requires Redis for background processing".to_string(),
        )
    })?;

    use crate::queue::{Job, JobPayload, JobQueue, LeaveImportPayload, TaskType};

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

    let queue = JobQueue::new(redis.clone());
    queue
        .enqueue_jobs(vec![job])
        .await
        .map_err(|e| AppError::Internal(format!("Failed to enqueue leave import job: {e}")))?;

    Ok(Json(serde_json::json!({
        "job_id": import_job_id.to_string(),
        "status": "enqueued",
    })))
}

/// GET /workspaces/:ws_id/leave/import/latest
/// Get the latest leave import job for the workspace (for page resume).
pub async fn get_latest_leave_import(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_permission(Permission::LeaveRequestsSubmit)?;

    use crate::repositories::import_job_repository::{
        ImportJobRepository, SqlxImportJobRepository,
    };
    let repo = SqlxImportJobRepository::new(state.infra.db.api().clone());

    match repo
        .get_latest_job(ctx.workspace_id, Some("leave_import".to_string()))
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
pub async fn download_leave_import_errors(
    Path((_, job_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<axum::response::Response> {
    ctx.require_permission(Permission::LeaveRequestsSubmit)?;

    use crate::repositories::import_job_repository::{
        ImportJobRepository, SqlxImportJobRepository,
    };
    let repo = SqlxImportJobRepository::new(state.infra.db.api().clone());

    let job = repo
        .get_job_by_id(job_id)
        .await
        .map_err(|e| AppError::Internal(format!("DB error: {e}")))?
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
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

#[derive(serde::Deserialize)]
pub struct DownloadTemplateQuery {
    pub lang: Option<String>,
}

/// GET /workspaces/:ws_id/leave/upload-template
/// Download a dynamic template Excel with leave type codes for this workspace.
pub async fn download_leave_import_template(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<DownloadTemplateQuery>,
    ctx: MemberContext,
) -> AppResult<axum::response::Response> {
    ctx.require_permission(Permission::LeaveRequestsSubmit)?;

    let leave_types = state
        .leaves
        .request_repo
        .get_leave_types(ctx.workspace_id)
        .await
        .map_err(AppError::Sqlx)?;

    let lang = query.lang.as_deref().unwrap_or("en");
    let bytes = crate::services::leave_import_service::LeaveImportService::generate_template(
        &leave_types,
        lang,
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate template: {e}")))?;

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
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}
