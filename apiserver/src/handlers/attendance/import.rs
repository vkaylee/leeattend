use axum::{
    extract::{Multipart, Path, State},
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{enums::ImportTargetType, AttendanceImportJob};
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/upload",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Attendance adjustment uploaded", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn upload_attendance_excel(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    let path = crate::utils::upload::extract_uploaded_file(
        &state,
        Some(&ctx.auth_token),
        &mut multipart,
        "file",
        crate::utils::upload::MAX_UPLOAD_FILE_SIZE,
        "imports",
    )
    .await?;
    let path = path.to_string_lossy().to_string();

    let job_id = state
        .attendance()
        .import_service()
        .batch_import_from_file(
            ctx.workspace_id,
            ctx.user_id,
            path,
            ImportTargetType::Adjustment,
        )
        .await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/logs/upload",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Attendance logs uploaded", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn upload_attendance_logs(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    let path = crate::utils::upload::extract_uploaded_file(
        &state,
        Some(&ctx.auth_token),
        &mut multipart,
        "file",
        crate::utils::upload::MAX_UPLOAD_FILE_SIZE,
        "imports",
    )
    .await?;
    let path = path.to_string_lossy().to_string();

    let job_id = state
        .attendance()
        .import_service()
        .batch_import_from_file(ctx.workspace_id, ctx.user_id, path, ImportTargetType::Log)
        .await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/employees/upload",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Employees uploaded", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn upload_employee_excel(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::EmployeesUpload)?;

    let path = crate::utils::upload::extract_uploaded_file(
        &state,
        Some(&ctx.auth_token),
        &mut multipart,
        "file",
        crate::utils::upload::MAX_UPLOAD_FILE_SIZE,
        "imports",
    )
    .await?;
    let path = path.to_string_lossy().to_string();

    // Create import job record in DB
    let import_job_id = crate::utils::id::new_id();
    let import_job = crate::models::AttendanceImportJob {
        id: import_job_id,
        workspace_id: ctx.workspace_id,
        user_id: ctx.user_id,
        status: crate::models::ImportJobStatus::Pending,
        total_rows: 0,
        processed_rows: 0,
        file_path: Some(path.clone()),
        error_log: None,
        target_type: ImportTargetType::Employee,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state
        .attendance()
        .import_repo()
        .create_job(&ctx.auth_token, &import_job)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create import job: {e}")))?;

    use crate::queue::{EmployeeImportPayload, Job, JobPayload, TaskType};

    let payload = EmployeeImportPayload {
        import_job_id,
        user_id: ctx.user_id,
    };

    let job = Job::new(
        import_job_id,
        0,
        TaskType::EmployeeImport,
        ctx.workspace_id,
        JobPayload::EmployeeImport(payload),
    );

    let queue = state.infra().job_queue()?;
    queue
        .enqueue_jobs(vec![job])
        .await
        .map_err(|e| AppError::Internal(format!("Failed to enqueue employee import job: {e}")))?;

    Ok(Json(serde_json::json!({ "job_id": import_job_id })))
}

#[derive(serde::Deserialize)]
pub struct ImportJobQuery {
    pub target_type: Option<ImportTargetType>,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/import-jobs/latest",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("target_type" = Option<ImportTargetType>, Query, description = "Target type filter")
    ),
    responses(
        (status = 200, description = "Latest import job", body = Option<AttendanceImportJob>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn get_latest_import_job(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ImportJobQuery>,
) -> AppResult<Json<Option<AttendanceImportJob>>> {
    let _workspace_id = ctx.workspace_id;

    let mut job_opt = state
        .attendance()
        .import_repo()
        .get_latest_job(&ctx.auth_token, query.target_type)
        .await?;

    if let Some(ref mut job) = job_opt {
        if job.status == crate::models::ImportJobStatus::Processing {
            if let Some(redis) = state.infra().redis() {
                let mut conn = redis.conn();
                let key = format!("import_progress:{}", job.id);
                if let Ok(Some(count)) = redis::cmd("GET")
                    .arg(&key)
                    .query_async::<Option<i32>>(&mut conn)
                    .await
                {
                    job.processed_rows = count;
                }
            }
        }
    }

    Ok(Json(job_opt))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/import-jobs/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Import job status", body = AttendanceImportJob)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, job_id = %__path_params.1))]
pub async fn get_import_job_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<AttendanceImportJob>> {
    let (_, job_id) = __path_params;
    let _workspace_id = ctx.workspace_id;

    let mut job = state
        .attendance()
        .import_repo()
        .get_job_by_id(&ctx.auth_token, job_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::JobNotFound))?;

    if job.status == crate::models::ImportJobStatus::Processing {
        if let Some(redis) = state.infra().redis() {
            let mut conn = redis.conn();
            let key = format!("import_progress:{}", job.id);
            if let Ok(Some(count)) = redis::cmd("GET")
                .arg(&key)
                .query_async::<Option<i32>>(&mut conn)
                .await
            {
                job.processed_rows = count;
            }
        }
    }

    Ok(Json(job))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/import-jobs/{id}/error-file",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Error file downloaded", body = Vec<u8>, content_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, job_id = %__path_params.1))]
pub async fn download_import_error_file(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let (_, job_id) = __path_params;
    let workspace_id = ctx.workspace_id;

    let job = state
        .attendance()
        .import_repo()
        .get_job_by_id(&ctx.auth_token, job_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::JobNotFound))?;

    if job.workspace_id != workspace_id {
        return Err(AppError::forbidden(I18nKey::Forbidden));
    }

    let base_dir = crate::utils::get_storage_dir();
    let file_path = format!("{base_dir}/imports/error_{job_id}.xlsx");

    let data = tokio::fs::read(&file_path)
        .await
        .map_err(|_e| AppError::not_found(I18nKey::ErrorFileNotFound))?;

    use axum::http::header;
    let headers = [
        (
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        ),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"error_{job_id}.xlsx\""),
        ),
    ];

    Ok((headers, data))
}
