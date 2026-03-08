use axum::{
    extract::{Multipart, Path, State},
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::AttendanceImportJob;
use crate::AppState;

pub async fn upload_attendance_excel(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::AttendanceLogsImport)?;
    if ctx.workspace_id != ws_id {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

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

    let job_id = state
        .attendance.import_service
        .batch_import_from_file(ws_id, ctx.user_id, path, "adjustment".to_string())
        .await
        ?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_attendance_logs(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::AttendanceLogsImport)?;
    if ctx.workspace_id != ws_id {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

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

    let job_id = state
        .attendance.import_service
        .batch_import_from_file(ws_id, ctx.user_id, path, "log".to_string())
        .await
        ?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_employee_excel(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::EmployeesUpload)?;
    if ctx.workspace_id != ws_id {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

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
    let job_id = state
        .attendance.import_service
        .batch_import_from_file(ws_id, ctx.user_id, path, "employee".to_string())
        .await
        ?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

#[derive(serde::Deserialize)]
pub struct ImportJobQuery {
    pub target_type: Option<String>,
}

pub async fn get_latest_import_job(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ImportJobQuery>,
) -> AppResult<Json<Option<AttendanceImportJob>>> {
    let job = state.attendance.import_repo.get_latest_job(workspace_id,
        query.target_type,
    )
    .await?;
    Ok(Json(job))
}

pub async fn get_import_job_status(
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Path((_ws_id, job_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<AttendanceImportJob>> {
    let job = state.attendance.import_repo.get_job_by_id(job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    Ok(Json(job))
}
