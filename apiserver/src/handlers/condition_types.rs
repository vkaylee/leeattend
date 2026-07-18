use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{ConditionTypeDefinition, CreateConditionTypeRequest, UpdateConditionTypeRequest};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize, Default)]
pub struct ListConditionTypesQuery {
    pub active_only: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/condition-types",
    responses(
        (status = 200, description = "List condition type definitions", body = Vec<ConditionTypeDefinition>)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("active_only" = Option<bool>, Query, description = "Filter active types only")
    )
)]
pub async fn list_condition_types(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ListConditionTypesQuery>,
) -> AppResult<Json<Vec<ConditionTypeDefinition>>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let types = state
        .core()
        .condition_type_service()
        .list(&ctx.auth_token, query.active_only.unwrap_or(false))
        .await?;
    Ok(Json(types))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/condition-types/{id}",
    responses(
        (status = 200, description = "Get condition type definition", body = ConditionTypeDefinition),
        (status = 404, description = "Not found")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Condition type ID")
    )
)]
pub async fn get_condition_type(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<ConditionTypeDefinition>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let definition = state
        .core()
        .condition_type_service()
        .get_by_id(&ctx.auth_token, id)
        .await?
        .ok_or_else(|| crate::errors::AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;
    Ok(Json(definition))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/condition-types",
    request_body = CreateConditionTypeRequest,
    responses(
        (status = 201, description = "Created condition type definition", body = ConditionTypeDefinition),
        (status = 409, description = "Code already exists")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_condition_type(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateConditionTypeRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let definition = state
        .core()
        .condition_type_service()
        .create(&ctx.auth_token, req)
        .await?;
    Ok((StatusCode::CREATED, Json(definition)))
}

#[utoipa::path(
    patch,
    path = "/workspaces/{workspace_id}/condition-types/{id}",
    request_body = UpdateConditionTypeRequest,
    responses(
        (status = 200, description = "Updated condition type definition", body = ConditionTypeDefinition),
        (status = 404, description = "Not found")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Condition type ID")
    )
)]
pub async fn update_condition_type(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateConditionTypeRequest>,
) -> AppResult<Json<ConditionTypeDefinition>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let definition = state
        .core()
        .condition_type_service()
        .update(&ctx.auth_token, id, req)
        .await?;
    Ok(Json(definition))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{workspace_id}/condition-types/{id}",
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Cannot delete system type"),
        (status = 404, description = "Not found")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Condition type ID")
    )
)]
pub async fn delete_condition_type(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    state
        .core()
        .condition_type_service()
        .delete(&ctx.auth_token, id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
