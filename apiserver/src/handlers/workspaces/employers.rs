use crate::errors::AppError;
use crate::handlers::common::MemberContext;
use crate::models::{CreateEmployerRequest, Employer, UpdateEmployerRequest};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/employers",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of employers", body = Vec<Employer>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_employers(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, AppError> {
    let employers = state
        .core
        .employer_service
        .list_employers(ctx.workspace_id)
        .await?;
    Ok(Json(employers))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/employers/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employer ID")
    ),
    responses(
        (status = 200, description = "Employer details", body = Employer)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_employer(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, AppError> {
    let employer = state
        .core
        .employer_service
        .get_employer(ctx.workspace_id, id)
        .await?;
    Ok(Json(employer))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/employers",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateEmployerRequest,
    responses(
        (status = 201, description = "Employer created", body = Employer)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_employer(
    State(state): State<Arc<AppState>>,
    Path(_workspace_id): Path<Uuid>,
    ctx: MemberContext,
    Json(payload): Json<CreateEmployerRequest>,
) -> Result<impl IntoResponse, AppError> {
    let actor_id = ctx.user_id;
    let employer = state
        .core
        .employer_service
        .create_employer(ctx.workspace_id, actor_id, payload)
        .await?;
    Ok((StatusCode::CREATED, Json(employer)))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/employers/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employer ID")
    ),
    request_body = UpdateEmployerRequest,
    responses(
        (status = 200, description = "Employer updated", body = Employer)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_employer(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(payload): Json<UpdateEmployerRequest>,
) -> Result<impl IntoResponse, AppError> {
    let actor_id = ctx.user_id;
    let employer = state
        .core
        .employer_service
        .update_employer(ctx.workspace_id, id, actor_id, payload)
        .await?;
    Ok(Json(employer))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/employers/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employer ID")
    ),
    responses(
        (status = 204, description = "Employer deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_employer(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, AppError> {
    let actor_id = ctx.user_id;
    state
        .core
        .employer_service
        .delete_employer(ctx.workspace_id, id, actor_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
