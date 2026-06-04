use crate::errors::AppError;
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateWorkLocationRequest, PaginatedResponse, PaginationParams, UpdateWorkLocationRequest,
    WorkLocation,
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
pub struct PaginatedLocations {
    pub data: Vec<WorkLocation>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/locations",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of locations", body = PaginatedLocations)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_locations(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (locations, total) = state
        .core()
        .location_service()
        .list_locations(ctx.workspace_id, page, limit)
        .await?;
    Ok(Json(PaginatedResponse {
        data: locations,
        total: total as usize,
        page,
        limit,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/locations/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Location ID")
    ),
    responses(
        (status = 200, description = "Work location details", body = WorkLocation)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_location(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, AppError> {
    let location = state
        .core()
        .location_service()
        .get_location(ctx.workspace_id, id)
        .await?;
    Ok(Json(location))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/locations",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateWorkLocationRequest,
    responses(
        (status = 201, description = "Work location created", body = WorkLocation)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_location(
    State(state): State<Arc<AppState>>,
    Path(_workspace_id): Path<Uuid>,
    ctx: MemberContext,
    Json(payload): Json<CreateWorkLocationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Ensure actor ID is available
    let actor_id = ctx.user_id;

    let location = state
        .core()
        .location_service()
        .create_location(ctx.workspace_id, actor_id, payload)
        .await?;

    Ok((StatusCode::CREATED, Json(location)))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/locations/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Location ID")
    ),
    request_body = UpdateWorkLocationRequest,
    responses(
        (status = 200, description = "Work location updated", body = WorkLocation)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_location(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(payload): Json<UpdateWorkLocationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Ensure actor ID is available
    let actor_id = ctx.user_id;

    let location = state
        .core()
        .location_service()
        .update_location(ctx.workspace_id, id, actor_id, payload)
        .await?;

    Ok(Json(location))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/locations/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Location ID")
    ),
    responses(
        (status = 204, description = "Work location deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_location(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, AppError> {
    // Ensure actor ID is available
    let actor_id = ctx.user_id;

    state
        .core()
        .location_service()
        .delete_location(ctx.workspace_id, id, actor_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
