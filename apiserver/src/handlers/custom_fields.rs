use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{
    BulkSetCustomFieldValuesRequest, CreateCustomFieldDefinitionRequest, CustomFieldDefinition,
    CustomFieldValueResponse, UpdateCustomFieldDefinitionRequest,
};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

// ── Definitions CRUD ───────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/custom-fields/definitions",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateCustomFieldDefinitionRequest,
    responses(
        (status = 201, description = "Definition created", body = CustomFieldDefinition)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_definition(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateCustomFieldDefinitionRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    let definition = state
        .core()
        .custom_field_service()
        .create_definition(&ctx.auth_token, ctx.user_id, req)
        .await?;
    Ok((StatusCode::CREATED, Json(definition)))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/custom-fields/definitions/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Definition ID")
    ),
    responses(
        (status = 200, description = "Definition details", body = CustomFieldDefinition)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_definition(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<CustomFieldDefinition>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let definition = state
        .core()
        .custom_field_service()
        .get_definition(&ctx.auth_token, id)
        .await?;
    Ok(Json(definition))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct ListDefinitionsParams {
    pub entity_type: Option<String>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/custom-fields/definitions",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ListDefinitionsParams
    ),
    responses(
        (status = 200, description = "List of definitions", body = Vec<CustomFieldDefinition>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_definitions(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ListDefinitionsParams>,
) -> AppResult<Json<Vec<CustomFieldDefinition>>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let definitions = state
        .core()
        .custom_field_service()
        .list_definitions(&ctx.auth_token, query.entity_type)
        .await?;
    Ok(Json(definitions))
}

#[utoipa::path(
    patch,
    path = "/workspaces/{ws_id}/custom-fields/definitions/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Definition ID")
    ),
    request_body = UpdateCustomFieldDefinitionRequest,
    responses(
        (status = 200, description = "Definition updated", body = CustomFieldDefinition)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_definition(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateCustomFieldDefinitionRequest>,
) -> AppResult<Json<CustomFieldDefinition>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    let definition = state
        .core()
        .custom_field_service()
        .update_definition(&ctx.auth_token, id, ctx.user_id, req)
        .await?;
    Ok(Json(definition))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/custom-fields/definitions/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Definition ID")
    ),
    responses(
        (status = 204, description = "Definition deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_definition(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    state
        .core()
        .custom_field_service()
        .delete_definition(&ctx.auth_token, id, ctx.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Values GET/PUT ────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("entity_type" = String, Path, description = "Entity type ('employee', 'employment', 'department')"),
        ("entity_id" = Uuid, Path, description = "Entity ID")
    ),
    responses(
        (status = 200, description = "List of values with definitions", body = Vec<CustomFieldValueResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_values(
    Path((_ws_id, entity_type, entity_id)): Path<(Uuid, String, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<CustomFieldValueResponse>>> {
    match entity_type.as_str() {
        "employee" | "employment" => {
            ctx.require_permission(Permission::EmployeesRead)?;
        }
        "department" => {
            ctx.require_permission(Permission::DepartmentsRead)?;
        }
        _ => {
            return Err(crate::errors::AppError::validation_args(
                crate::utils::i18n::I18nKey::Validation,
                vec![("details", format!("Invalid entity type '{entity_type}'."))],
            ));
        }
    }

    let values = state
        .core()
        .custom_field_service()
        .get_values_for_entity(&ctx.auth_token, entity_id)
        .await?;
    Ok(Json(values))
}

#[utoipa::path(
    put,
    path = "/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("entity_type" = String, Path, description = "Entity type ('employee', 'employment', 'department')"),
        ("entity_id" = Uuid, Path, description = "Entity ID")
    ),
    request_body = BulkSetCustomFieldValuesRequest,
    responses(
        (status = 200, description = "Updated list of values with definitions", body = Vec<CustomFieldValueResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn set_values(
    Path((_ws_id, entity_type, entity_id)): Path<(Uuid, String, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<BulkSetCustomFieldValuesRequest>,
) -> AppResult<Json<Vec<CustomFieldValueResponse>>> {
    match entity_type.as_str() {
        "employee" | "employment" => {
            ctx.require_permission(Permission::EmployeesUpdate)?;
        }
        "department" => {
            ctx.require_permission(Permission::DepartmentsUpdate)?;
        }
        _ => {
            return Err(crate::errors::AppError::validation_args(
                crate::utils::i18n::I18nKey::Validation,
                vec![("details", format!("Invalid entity type '{entity_type}'."))],
            ));
        }
    }

    let values = state
        .core()
        .custom_field_service()
        .set_values(
            &ctx.auth_token,
            ctx.user_id,
            &entity_type,
            entity_id,
            req.values,
        )
        .await?;
    Ok(Json(values))
}
