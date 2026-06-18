use axum::{
    extract::{Multipart, Path, State},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::file::{FileCategory, FileDownloadResponse, FileOwnerType, WorkspaceFile};
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateUploadUrlRequest {
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub category: FileCategory,
    pub owner_type: FileOwnerType,
    pub owner_id: Uuid,
    pub expires_in_secs: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/files/presigned",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateUploadUrlRequest,
    responses(
        (status = 200, description = "Presigned URL generated successfully")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_upload_url_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<CreateUploadUrlRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let (file, url) = state
        .file_service()
        .create_upload_url(
            &ctx.auth_token,
            payload.owner_type,
            payload.owner_id,
            payload.category,
            &payload.file_name,
            &payload.mime_type,
            payload.file_size,
            Some(ctx.user_id),
            payload.expires_in_secs,
        )
        .await?;

    Ok(Json(serde_json::json!({
        "file": file,
        "upload_url": url,
    })))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/files/upload",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "File uploaded successfully", body = WorkspaceFile)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[allow(clippy::too_many_lines)]
pub async fn upload_file_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<WorkspaceFile>> {
    let mut file_name = None;
    let mut mime_type = None;
    let mut data = None;
    let mut category = None;
    let mut owner_type = None;
    let mut owner_id = None;
    let mut expires_in_secs = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::validation_args(
            I18nKey::Validation,
            vec![("details", format!("Multipart error: {e}"))],
        )
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = Some(field.file_name().unwrap_or("upload").to_string());
                mime_type = Some(
                    field
                        .content_type()
                        .unwrap_or("application/octet-stream")
                        .to_string(),
                );
                data = Some(field.bytes().await.map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Read file failed: {e}"))],
                    )
                })?);
            }
            "category" => {
                let s = field.text().await.map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Read category failed: {e}"))],
                    )
                })?;
                category = Some(s.parse::<FileCategory>().map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Invalid category: {e}"))],
                    )
                })?);
            }
            "owner_type" => {
                let s = field.text().await.map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Read owner_type failed: {e}"))],
                    )
                })?;
                owner_type = Some(s.parse::<FileOwnerType>().map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Invalid owner_type: {e}"))],
                    )
                })?);
            }
            "owner_id" => {
                let s = field.text().await.map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Read owner_id failed: {e}"))],
                    )
                })?;
                owner_id = Some(Uuid::parse_str(&s).map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Invalid owner_id UUID: {e}"))],
                    )
                })?);
            }
            "expires_in_secs" => {
                let s = field.text().await.map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Read expires_in_secs failed: {e}"))],
                    )
                })?;
                expires_in_secs = Some(s.parse::<i64>().map_err(|e| {
                    AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", format!("Invalid expires_in_secs integer: {e}"))],
                    )
                })?);
            }
            _ => {}
        }
    }

    let file_name = file_name.ok_or_else(|| AppError::validation(I18nKey::UploadNoFile))?;
    let mime_type = mime_type.ok_or_else(|| AppError::validation(I18nKey::UploadNoFile))?;
    let data = data.ok_or_else(|| AppError::validation(I18nKey::UploadNoFile))?;
    let category = category.ok_or_else(|| {
        AppError::validation_args(
            I18nKey::Validation,
            vec![("details", "Missing category field".to_string())],
        )
    })?;
    let owner_type = owner_type.ok_or_else(|| {
        AppError::validation_args(
            I18nKey::Validation,
            vec![("details", "Missing owner_type field".to_string())],
        )
    })?;
    let owner_id = owner_id.ok_or_else(|| {
        AppError::validation_args(
            I18nKey::Validation,
            vec![("details", "Missing owner_id field".to_string())],
        )
    })?;

    let file = state
        .file_service()
        .upload_file(
            &ctx.auth_token,
            owner_type,
            owner_id,
            category,
            &file_name,
            &mime_type,
            data,
            Some(ctx.user_id),
            expires_in_secs,
        )
        .await?;

    Ok(Json(file))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/files/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "File metadata retrieved", body = WorkspaceFile)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_file_metadata_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<WorkspaceFile>> {
    let (_, file_id) = params;
    let file = state
        .file_service()
        .get_file_metadata(&ctx.auth_token, file_id)
        .await?;
    Ok(Json(file))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/files/{id}/download",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "File downloaded successfully")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn download_file_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> AppResult<Response> {
    let (_, file_id) = params;
    let resp = state
        .file_service()
        .download_file(&ctx.auth_token, file_id)
        .await?;
    match resp {
        FileDownloadResponse::ProxyBytes(bytes) => {
            let file = state
                .file_service()
                .get_file_metadata(&ctx.auth_token, file_id)
                .await?;
            let headers = [
                (axum::http::header::CONTENT_TYPE, file.mime_type.clone()),
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", file.file_name),
                ),
            ];
            Ok((axum::http::StatusCode::OK, headers, bytes).into_response())
        }
        FileDownloadResponse::PresignedUrl(url) => {
            Ok(axum::response::Redirect::temporary(&url).into_response())
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/files/{id}/url",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "Presigned URL generated successfully")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_file_url_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    let (_, file_id) = params;
    let url = state
        .file_service()
        .get_download_url(&ctx.auth_token, file_id)
        .await?;
    Ok(Json(serde_json::json!({ "url": url })))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/files/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "File ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "File version updated successfully", body = WorkspaceFile)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_file_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
    mut multipart: Multipart,
) -> AppResult<Json<WorkspaceFile>> {
    let (_, file_id) = params;
    let mut file_name = None;
    let mut mime_type = None;
    let mut data = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::validation_args(
            I18nKey::Validation,
            vec![("details", format!("Multipart error: {e}"))],
        )
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            file_name = Some(field.file_name().unwrap_or("upload").to_string());
            mime_type = Some(
                field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string(),
            );
            data = Some(field.bytes().await.map_err(|e| {
                AppError::validation_args(
                    I18nKey::Validation,
                    vec![("details", format!("Read file failed: {e}"))],
                )
            })?);
        }
    }

    let file_name = file_name.ok_or_else(|| AppError::validation(I18nKey::UploadNoFile))?;
    let mime_type = mime_type.ok_or_else(|| AppError::validation(I18nKey::UploadNoFile))?;
    let data = data.ok_or_else(|| AppError::validation(I18nKey::UploadNoFile))?;

    let file = state
        .file_service()
        .update_file(
            &ctx.auth_token,
            file_id,
            &file_name,
            &mime_type,
            data,
            Some(ctx.user_id),
        )
        .await?;

    Ok(Json(file))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/files/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "File soft deleted successfully")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_file_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    let (_, file_id) = params;
    state
        .file_service()
        .delete_file(&ctx.auth_token, file_id)
        .await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/files/{id}/versions",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "File version history retrieved", body = Vec<WorkspaceFile>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_file_versions_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<WorkspaceFile>>> {
    let (_, file_id) = params;
    let versions = state
        .file_service()
        .get_file_versions(&ctx.auth_token, file_id)
        .await?;
    Ok(Json(versions))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("owner_type" = String, Path, description = "Owner Type (e.g. employee, user)"),
        ("owner_id" = Uuid, Path, description = "Owner ID")
    ),
    responses(
        (status = 200, description = "Files retrieved successfully", body = Vec<WorkspaceFile>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_files_by_owner_handler(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, String, Uuid)>,
) -> AppResult<Json<Vec<WorkspaceFile>>> {
    let (_, owner_type_str, owner_id) = params;
    let owner_type = owner_type_str.parse::<FileOwnerType>().map_err(|e| {
        AppError::validation_args(
            I18nKey::Validation,
            vec![("details", format!("Invalid owner_type: {e}"))],
        )
    })?;
    let files = state
        .file_service()
        .list_files_by_owner(&ctx.auth_token, owner_type, owner_id)
        .await?;
    Ok(Json(files))
}
