#![allow(unused_variables)]
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
use crate::AppState;

pub async fn upload_attendance_excel(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::AttendanceLogsImport)?;

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

            let base_dir = crate::utils::get_storage_dir();
            let dir = format!("{base_dir}/imports");
            tokio::fs::create_dir_all(&dir).await?;
            let path = format!("{}/{}_{}", dir, crate::utils::id::new_id(), file_name);
            tokio::fs::write(&path, data).await?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;

    let job_id = state
        .attendance
        .import_service
        .batch_import_from_file(
            ctx.workspace_id,
            ctx.user_id,
            path,
            ImportTargetType::Adjustment,
        )
        .await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_attendance_logs(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::AttendanceLogsImport)?;

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

            let base_dir = crate::utils::get_storage_dir();
            let dir = format!("{base_dir}/imports");
            tokio::fs::create_dir_all(&dir).await?;
            let path = format!("{}/{}_{}", dir, crate::utils::id::new_id(), file_name);
            tokio::fs::write(&path, data).await?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;

    let job_id = state
        .attendance
        .import_service
        .batch_import_from_file(ctx.workspace_id, ctx.user_id, path, ImportTargetType::Log)
        .await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_employee_excel(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::EmployeesUpload)?;

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

            let base_dir = crate::utils::get_storage_dir();
            let dir = format!("{base_dir}/imports");
            tokio::fs::create_dir_all(&dir).await?;
            let path = format!("{}/{}_{}", dir, crate::utils::id::new_id(), file_name);
            tokio::fs::write(&path, data).await?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;

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
        .attendance
        .import_repo
        .create_job(&ctx.auth_token, &import_job)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create import job: {e}")))?;

    // Require Redis for background processing
    let redis = state.infra.redis.as_ref().ok_or_else(|| {
        AppError::ServiceUnavailable(
            "Employee import requires Redis for background processing".to_string(),
        )
    })?;

    use crate::queue::{EmployeeImportPayload, Job, JobPayload, JobQueue, TaskType};

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

    let queue = JobQueue::new(redis.clone());
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

pub async fn get_latest_import_job(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ImportJobQuery>,
) -> AppResult<Json<Option<AttendanceImportJob>>> {
    let workspace_id = ctx.workspace_id;

    let mut job_opt = state
        .attendance
        .import_repo
        .get_latest_job(&ctx.auth_token, query.target_type)
        .await?;

    if let Some(ref mut job) = job_opt {
        if job.status == crate::models::ImportJobStatus::Processing {
            if let Some(ref redis) = state.infra.redis {
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

pub async fn get_import_job_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_, job_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<AttendanceImportJob>> {
    let workspace_id = ctx.workspace_id;

    let mut job = state
        .attendance
        .import_repo
        .get_job_by_id(&ctx.auth_token, job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    if job.status == crate::models::ImportJobStatus::Processing {
        if let Some(ref redis) = state.infra.redis {
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

pub async fn download_import_error_file(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_, job_id)): Path<(Uuid, Uuid)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let workspace_id = ctx.workspace_id;

    let job = state
        .attendance
        .import_repo
        .get_job_by_id(&ctx.auth_token, job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    if job.workspace_id != workspace_id {
        return Err(AppError::Forbidden(
            "Unauthorized access to this job".to_string(),
        ));
    }

    let base_dir = crate::utils::get_storage_dir();
    let file_path = format!("{base_dir}/imports/error_{job_id}.xlsx");

    let data = tokio::fs::read(&file_path)
        .await
        .map_err(|_e| AppError::NotFound("Error file not found or has expired".to_string()))?;

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
