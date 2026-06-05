use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{AddMemberRequest, WorkspaceMemberDetailResponse};
use crate::services::audit_service::AuditHelper;
use crate::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateMemberRoleRequest {
    pub role_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/members",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of workspace members", body = Vec<WorkspaceMemberDetailResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_workspace_members(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<WorkspaceMemberDetailResponse>>> {
    let workspace_id = ctx.workspace_id;

    let members = state
        .core()
        .workspace_repo()
        .get_workspace_members(workspace_id)
        .await?;

    let extra_roles = state
        .core()
        .workspace_repo()
        .get_all_extra_roles(workspace_id)
        .await?;

    let mut response = Vec::with_capacity(members.len());
    for member in members {
        let extra_role_ids = extra_roles
            .iter()
            .filter(|r| r.user_id == member.user_id)
            .map(|r| r.role_id)
            .collect();

        response.push(WorkspaceMemberDetailResponse {
            detail: member,
            extra_role_ids,
        });
    }

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/members",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = AddMemberRequest,
    responses(
        (status = 201, description = "Member added successfully")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_workspace_member(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
    Json(req): Json<AddMemberRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersInvite)?;

    let user_id = state
        .core()
        .user_repo()
        .get_user_by_email(&req.email)
        .await?
        .map(|u| u.id())
        .ok_or_else(|| AppError::NotFound("User with this email not found".to_string()))?;

    // 3. Add to workspace
    state
        .core()
        .workspace_repo()
        .add_member(ctx.workspace_id, user_id, req.role_id)
        .await?;

    AuditHelper::log(crate::services::audit_service::AuditLogArgs {
        repo: state.core().audit_repo(),
        workspace_id: ctx.workspace_id,
        actor_id: Some(ctx.user_id),
        actor_type: Some("user".to_string()),
        actor_name: ctx.user_name.clone(),
        actor_email: Some(crate::utils::masking::mask_email(&ctx.user_email)),
        target_user_id: Some(user_id),
        target_resource_id: None,
        action: "add_member",
        details: Some(serde_json::json!({ "role_id": req.role_id })),
        headers: &headers,
    })
    .await;

    if let Some(cache) = state.infra().cache() {
        let _ = cache
            .delete(&format!(
                "cache:auth:permissions:{}:{}",
                ctx.auth_token.workspace_id(),
                user_id
            ))
            .await;
    }

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    patch,
    path = "/v1/workspaces/{ws_id}/members/{user_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("user_id" = Uuid, Path, description = "User ID to update")
    ),
    request_body = UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "Member role updated successfully")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_workspace_member_role(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> AppResult<StatusCode> {
    let (_, user_id) = __path_params;
    ctx.require_permission(Permission::MembersRolesManage)?;

    let old_role_id = state
        .core()
        .workspace_repo()
        .find_primary_role(ctx.workspace_id, user_id)
        .await
        .unwrap_or(None);

    state
        .core()
        .workspace_repo()
        .update_member_role(ctx.workspace_id, user_id, req.role_id)
        .await?;

    AuditHelper::log(crate::services::audit_service::AuditLogArgs {
        repo: state.core().audit_repo(),
        workspace_id: ctx.workspace_id,
        actor_id: Some(ctx.user_id),
        actor_type: Some("user".to_string()),
        actor_name: ctx.user_name.clone(),
        actor_email: Some(crate::utils::masking::mask_email(&ctx.user_email)),
        target_user_id: Some(user_id),
        target_resource_id: None,
        action: "update_member_role",
        details: Some(serde_json::json!({
            "old_role_id": old_role_id,
            "new_role_id": req.role_id
        })),
        headers: &headers,
    })
    .await;

    if let Some(cache) = state.infra().cache() {
        let _ = cache
            .delete(&format!(
                "cache:auth:permissions:{}:{}",
                ctx.auth_token.workspace_id(),
                user_id
            ))
            .await;
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/members/{user_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("user_id" = Uuid, Path, description = "User ID to remove")
    ),
    responses(
        (status = 204, description = "Member removed successfully")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn remove_workspace_member(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_, user_id) = __path_params;
    ctx.require_permission(Permission::MembersRemove)?;

    // Don't allow removing yourself (prevent lockout if only one admin, though owner should probably remain)
    if user_id == ctx.user_id {
        return Err(AppError::Validation(
            "You cannot remove yourself from the workspace".to_string(),
        ));
    }

    state
        .core()
        .workspace_repo()
        .remove_member(ctx.workspace_id, user_id)
        .await?;

    AuditHelper::log(crate::services::audit_service::AuditLogArgs {
        repo: state.core().audit_repo(),
        workspace_id: ctx.workspace_id,
        actor_id: Some(ctx.user_id),
        actor_type: Some("user".to_string()),
        actor_name: ctx.user_name.clone(),
        actor_email: Some(crate::utils::masking::mask_email(&ctx.user_email)),
        target_user_id: Some(user_id),
        target_resource_id: None,
        action: "remove_member",
        details: None,
        headers: &headers,
    })
    .await;

    if let Some(cache) = state.infra().cache() {
        let _ = cache
            .delete(&format!(
                "cache:auth:permissions:{}:{}",
                ctx.auth_token.workspace_id(),
                user_id
            ))
            .await;
    }

    Ok(StatusCode::NO_CONTENT)
}
