use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::AppState;

// Fake DTOs for now (will connect to DB service in next iter)
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct DocumentFolderDto {
    pub id: Uuid,
    pub name: String,
    pub parent_folder_id: Option<Uuid>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_folder_id: Option<Uuid>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RenameFolderRequest {
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/documents/folders",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List folders", body = Vec<DocumentFolderDto>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_folders_handler(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<Vec<DocumentFolderDto>>> {
    Ok(Json(vec![]))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/documents/folders",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateFolderRequest,
    responses(
        (status = 200, description = "Folder created", body = DocumentFolderDto)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_folder_handler(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(payload): Json<CreateFolderRequest>,
) -> AppResult<Json<DocumentFolderDto>> {
    Ok(Json(DocumentFolderDto {
        id: Uuid::new_v4(),
        name: payload.name,
        parent_folder_id: payload.parent_folder_id,
    }))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/documents/folders/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Folder ID")
    ),
    request_body = RenameFolderRequest,
    responses(
        (status = 200, description = "Folder renamed", body = DocumentFolderDto)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn rename_folder_handler(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
    Json(payload): Json<RenameFolderRequest>,
) -> AppResult<Json<DocumentFolderDto>> {
    let (_, folder_id) = params;
    Ok(Json(DocumentFolderDto {
        id: folder_id,
        name: payload.name,
        parent_folder_id: None,
    }))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/documents/folders/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Folder ID")
    ),
    responses(
        (status = 200, description = "Folder deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_folder_handler(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path(_params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "status": "success" })))
}
