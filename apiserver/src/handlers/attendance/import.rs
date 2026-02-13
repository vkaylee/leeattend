use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::MemberRole;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::AppState;

pub async fn upload_attendance_excel(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    // RBAC: Only Admin/Owner
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::FORBIDDEN)?;

    if role != MemberRole::Admin && role != MemberRole::Owner {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut file_path = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            let dir = "storage/imports";
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);
            tokio::fs::write(&path, data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or(StatusCode::BAD_REQUEST)?;

    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "adjustment".to_string()
    ).await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_attendance_logs(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, StatusCode> {
    // RBAC: Only Admin/Owner
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::FORBIDDEN)?;

    if role != MemberRole::Admin && role != MemberRole::Owner {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut file_path = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            let dir = "storage/imports";
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);
            tokio::fs::write(&path, data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            file_path = Some(path);
        }
    }

    let path = file_path.ok_or(StatusCode::BAD_REQUEST)?;

    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "log".to_string()
    ).await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

pub async fn upload_employee_excel(

    State(state): State<Arc<AppState>>,

    Path(ws_id): Path<Uuid>,

    MemberContext { user_id, .. }: MemberContext,

    mut multipart: Multipart,

) -> Result<impl IntoResponse, StatusCode> {

    // RBAC: Only Admin/Owner

    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {

        pool: state.db.clone(),

    }

    .find_primary_role(ws_id, user_id)

    .await

    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?

    .ok_or(StatusCode::FORBIDDEN)?;



    if role != MemberRole::Admin && role != MemberRole::Owner {

        return Err(StatusCode::FORBIDDEN);

    }



    let mut file_path = None;

    while let Some(field) = multipart

        .next_field()

        .await

        .map_err(|_| StatusCode::BAD_REQUEST)?

    {

        let name = field.name().unwrap_or("").to_string();

        if name == "file" {

            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();

            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;



            let dir = "storage/imports";

            tokio::fs::create_dir_all(dir)

                .await

                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let path = format!("{}/{}_{}", dir, Uuid::new_v4(), file_name);

            tokio::fs::write(&path, data)

                .await

                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            file_path = Some(path);

        }

    }



    let path = file_path.ok_or(StatusCode::BAD_REQUEST)?;
    let job_id = crate::services::AttendanceImportService::batch_import_from_file(
        state.db.clone(),
        ws_id,
        user_id,
        path,
        "employee".to_string()
    ).await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

#[derive(Deserialize)]
pub struct ImportJobQuery {
    pub target_type: Option<String>,
}

pub async fn get_latest_import_job(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<ImportJobQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    match crate::repositories::import_job_repository::get_latest_job(
        &state.db,
        workspace_id,
        query.target_type,
    )
    .await
    {
        Ok(job) => Ok(Json(job)),
        Err(e) => {
            error!("GET_LATEST_JOB_ERROR: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_import_job_status(
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Path((_ws_id, job_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let job = crate::repositories::import_job_repository::get_job_by_id(&state.db, job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(job))
}
