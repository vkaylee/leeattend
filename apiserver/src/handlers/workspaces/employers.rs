use crate::errors::AppError;
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateEmployerRequest, Employer, PaginatedResponse, PaginationParams, UpdateEmployerRequest,
};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedEmployers {
    pub data: Vec<Employer>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/employers",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of employers", body = PaginatedEmployers)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_employers(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (employers, total) = state
        .core()
        .employer_service()
        .list_employers(&ctx.auth_token, page, limit)
        .await?;
    Ok(Json(PaginatedResponse {
        data: employers,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,

        truncated: None,
    }))
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
        .core()
        .employer_service()
        .get_employer(&ctx.auth_token, id)
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
        .core()
        .employer_service()
        .create_employer(&ctx.auth_token, actor_id, payload)
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
        .core()
        .employer_service()
        .update_employer(&ctx.auth_token, id, actor_id, payload)
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
        .core()
        .employer_service()
        .delete_employer(&ctx.auth_token, id, actor_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
