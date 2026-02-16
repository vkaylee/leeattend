use axum::{
    extract::{Multipart, Path, State},
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{MemberRole, AttendanceImportJob};
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::AppState;

pub async fn upload_attendance_excel(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await?
    .ok_or_else(|| AppError::Forbidden("Forbidden".to_string()))?;

    if role != MemberRole::Admin && role != MemberRole::Owner {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

    let mut file_path = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field.bytes().await
                .map_err(|e| AppError::Validation(format!("Failed to read file bytes: {e}")))?;

            let dir = "storage/imports";
            tokio::fs::create_dir_all(dir)
                .await?;
            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);
            tokio::fs::write(&path, data)
                .await?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;

    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "adjustment".to_string()
    ).await
    .map_err(|s| AppError::from((s, "Import failed".to_string())))?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_attendance_logs(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await?
    .ok_or_else(|| AppError::Forbidden("Forbidden".to_string()))?;

    if role != MemberRole::Admin && role != MemberRole::Owner {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

    let mut file_path = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field.bytes().await
                .map_err(|e| AppError::Validation(format!("Failed to read file bytes: {e}")))?;

            let dir = "storage/imports";
            tokio::fs::create_dir_all(dir)
                .await?;
            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);
            tokio::fs::write(&path, data)
                .await?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;

    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "log".to_string()
    ).await
    .map_err(|s| AppError::from((s, "Import failed".to_string())))?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_employee_excel(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await?
    .ok_or_else(|| AppError::Forbidden("Forbidden".to_string()))?;

    if role != MemberRole::Admin && role != MemberRole::Owner {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

    let mut file_path = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field.bytes().await
                .map_err(|e| AppError::Validation(format!("Failed to read file bytes: {e}")))?;

            let dir = "storage/imports";
            tokio::fs::create_dir_all(dir)
                .await?;
            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);
            tokio::fs::write(&path, data)
                .await?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;
    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "employee".to_string()
    ).await
    .map_err(|s| AppError::from((s, "Import failed".to_string())))?;

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
    let job = crate::repositories::import_job_repository::get_latest_job(
        &state.db,
        workspace_id,
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
    let job = crate::repositories::import_job_repository::get_job_by_id(&state.db, job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    Ok(Json(job))
}
