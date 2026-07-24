use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateChecklistRequest, CreateLifecycleChecklistTemplateItemRequest,
    CreateLifecycleChecklistTemplateRequest, LifecycleChecklistResponse,
    LifecycleChecklistTemplate, LifecycleChecklistTemplateItem, LifecycleChecklistType,
    PendingTaskResponse, UpdateChecklistItemStatusRequest,
    UpdateLifecycleChecklistTemplateItemRequest, UpdateLifecycleChecklistTemplateRequest,
};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

// ── Templates Admin CRUD ───────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/lifecycle-templates",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateLifecycleChecklistTemplateRequest,
    responses(
        (status = 201, description = "Template created", body = LifecycleChecklistTemplate)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_template(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateLifecycleChecklistTemplateRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    let template = state
        .core()
        .lifecycle_service()
        .create_template(&ctx.auth_token, req)
        .await?;
    Ok((StatusCode::CREATED, Json(template)))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/lifecycle-templates/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "Template details", body = LifecycleChecklistTemplate)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_template(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<LifecycleChecklistTemplate>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let template = state
        .core()
        .lifecycle_service()
        .get_template(&ctx.auth_token, id)
        .await?;
    Ok(Json(template))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct ListTemplatesQuery {
    pub checklist_type: Option<LifecycleChecklistType>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/lifecycle-templates",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ListTemplatesQuery
    ),
    responses(
        (status = 200, description = "List of active templates", body = Vec<LifecycleChecklistTemplate>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ListTemplatesQuery>,
) -> AppResult<Json<Vec<LifecycleChecklistTemplate>>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let templates = state
        .core()
        .lifecycle_service()
        .list_active_templates(&ctx.auth_token, query.checklist_type)
        .await?;
    Ok(Json(templates))
}

#[utoipa::path(
    patch,
    path = "/workspaces/{ws_id}/lifecycle-templates/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Template ID")
    ),
    request_body = UpdateLifecycleChecklistTemplateRequest,
    responses(
        (status = 200, description = "Template updated", body = LifecycleChecklistTemplate)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_template(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateLifecycleChecklistTemplateRequest>,
) -> AppResult<Json<LifecycleChecklistTemplate>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    let template = state
        .core()
        .lifecycle_service()
        .update_template(&ctx.auth_token, id, req)
        .await?;
    Ok(Json(template))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/lifecycle-templates/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Template ID")
    ),
    responses(
        (status = 204, description = "Template deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_template(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    state
        .core()
        .lifecycle_service()
        .delete_template(&ctx.auth_token, id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Template Items CRUD ───────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/lifecycle-templates/{id}/items",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Template ID")
    ),
    request_body = CreateLifecycleChecklistTemplateItemRequest,
    responses(
        (status = 201, description = "Template item created", body = LifecycleChecklistTemplateItem)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_template_item(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateLifecycleChecklistTemplateItemRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    let item = state
        .core()
        .lifecycle_service()
        .create_template_item(&ctx.auth_token, id, req)
        .await?;
    Ok((StatusCode::CREATED, Json(item)))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("tid" = Uuid, Path, description = "Template ID"),
        ("iid" = Uuid, Path, description = "Template Item ID")
    ),
    responses(
        (status = 200, description = "Template item details", body = LifecycleChecklistTemplateItem)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_template_item(
    Path((_ws_id, _tid, iid)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<LifecycleChecklistTemplateItem>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let item = state
        .core()
        .lifecycle_service()
        .get_template_item(&ctx.auth_token, iid)
        .await?;
    Ok(Json(item))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/lifecycle-templates/{id}/items",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "List of items for template", body = Vec<LifecycleChecklistTemplateItem>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_template_items(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LifecycleChecklistTemplateItem>>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;
    let items = state
        .core()
        .lifecycle_service()
        .list_template_items(&ctx.auth_token, id)
        .await?;
    Ok(Json(items))
}

#[utoipa::path(
    put,
    path = "/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("tid" = Uuid, Path, description = "Template ID"),
        ("iid" = Uuid, Path, description = "Template Item ID")
    ),
    request_body = UpdateLifecycleChecklistTemplateItemRequest,
    responses(
        (status = 200, description = "Template item updated", body = LifecycleChecklistTemplateItem)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_template_item(
    Path((_ws_id, _tid, iid)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateLifecycleChecklistTemplateItemRequest>,
) -> AppResult<Json<LifecycleChecklistTemplateItem>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    let item = state
        .core()
        .lifecycle_service()
        .update_template_item(&ctx.auth_token, iid, req)
        .await?;
    Ok(Json(item))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("tid" = Uuid, Path, description = "Template ID"),
        ("iid" = Uuid, Path, description = "Template Item ID")
    ),
    responses(
        (status = 204, description = "Template item deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_template_item(
    Path((_ws_id, _tid, iid)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    state
        .core()
        .lifecycle_service()
        .delete_template_item(&ctx.auth_token, iid)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Checklist Instances ───────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/employments/{emp_id}/lifecycle",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("emp_id" = Uuid, Path, description = "Employment ID")
    ),
    request_body = CreateChecklistRequest,
    responses(
        (status = 201, description = "Checklist started", body = LifecycleChecklistResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_checklist(
    Path((_ws_id, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateChecklistRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::EmployeesUpdate)?;
    let checklist = state
        .core()
        .lifecycle_service()
        .create_checklist_from_template(
            &ctx.auth_token,
            emp_id,
            req.template_id,
            Some(ctx.user_id),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(checklist)))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/employments/{emp_id}/lifecycle",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("emp_id" = Uuid, Path, description = "Employment ID")
    ),
    responses(
        (status = 200, description = "List of checklists for employment", body = Vec<LifecycleChecklistResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_checklists(
    Path((_ws_id, emp_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LifecycleChecklistResponse>>> {
    ctx.require_permission(Permission::EmployeesRead)?;
    let checklists = state
        .core()
        .lifecycle_service()
        .list_checklists_for_employment(&ctx.auth_token, emp_id)
        .await?;
    Ok(Json(checklists))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/lifecycle/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Checklist ID")
    ),
    responses(
        (status = 200, description = "Checklist details with items", body = LifecycleChecklistResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_checklist(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<LifecycleChecklistResponse>> {
    ctx.require_permission(Permission::EmployeesRead)?;
    let checklist = state
        .core()
        .lifecycle_service()
        .get_checklist(&ctx.auth_token, id)
        .await?;
    Ok(Json(checklist))
}

#[utoipa::path(
    patch,
    path = "/workspaces/{ws_id}/lifecycle/{id}/items/{iid}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Checklist ID"),
        ("iid" = Uuid, Path, description = "Checklist Item ID")
    ),
    request_body = UpdateChecklistItemStatusRequest,
    responses(
        (status = 200, description = "Item status updated, returns parent checklist details", body = LifecycleChecklistResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_item_status(
    Path((_ws_id, id, iid)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateChecklistItemStatusRequest>,
) -> AppResult<Json<LifecycleChecklistResponse>> {
    let checklist = state
        .core()
        .lifecycle_service()
        .get_checklist(&ctx.auth_token, id)
        .await?;

    // Must be either assignee of the item, OR have EmployeesUpdate permission
    let is_assignee = checklist
        .items
        .iter()
        .any(|item| item.id() == iid && item.assignee_user_id() == Some(ctx.user_id));

    if !is_assignee {
        ctx.require_permission(Permission::EmployeesUpdate)?;
    }

    let checklist = state
        .core()
        .lifecycle_service()
        .update_item_status(
            &ctx.auth_token,
            iid,
            req.status,
            req.notes,
            Some(ctx.user_id),
        )
        .await?;
    Ok(Json(checklist))
}

// ── Dashboard Task Feed ───────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/lifecycle/my-tasks",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Pending checklist tasks for logged-in user", body = Vec<PendingTaskResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_tasks(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<PendingTaskResponse>>> {
    // No specific employees:read permission required since it is a self-service feed of pending tasks assigned directly to the current user
    let tasks = state
        .core()
        .lifecycle_service()
        .get_pending_tasks_for_user(&ctx.auth_token, ctx.user_id)
        .await?;
    Ok(Json(tasks))
}
