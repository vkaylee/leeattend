#![allow(unused_variables)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{AssignPermissionRequest, MemberExtraPermission};
use crate::services::audit_service::AuditHelper;
use crate::AppState;

/// GET /workspaces/:ws_id/members/:user_id/permissions
/// List all extra permissions for a member.
pub async fn get_member_extra_permissions(
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<MemberExtraPermission>>> {
    ctx.require_permission(Permission::MembersRead)?;

    let perms = state
        .core
        .workspace_repo
        .get_member_extra_permissions(ctx.workspace_id, user_id)
        .await?;

    Ok(Json(perms))
}

/// POST /workspaces/:ws_id/members/:user_id/permissions
/// Grant an extra permission to a member.
pub async fn assign_extra_permission(
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    ctx: MemberContext,
    Json(req): Json<AssignPermissionRequest>,
) -> AppResult<(StatusCode, Json<MemberExtraPermission>)> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    // Validate the permission string
    if Permission::from_str_permission(&req.permission).is_none() {
        return Err(AppError::Validation(format!(
            "Unknown permission: '{}'. Use GET /permissions/available for valid values.",
            req.permission
        )));
    }

    if req.expires_at <= chrono::Utc::now() {
        return Err(AppError::Validation(
            "Expiration time must be in the future".to_string(),
        ));
    }

    let perm = state
        .core
        .workspace_repo
        .assign_extra_permission(
            ctx.workspace_id,
            user_id,
            req.permission.clone(),
            req.expires_at,
            ctx.user_id,
        )
        .await?;

    AuditHelper::log(crate::services::audit_service::AuditLogArgs {
        repo: &state.core.audit_repo,
        workspace_id: ctx.workspace_id,
        actor_id: Some(ctx.user_id),
        actor_type: None,
        actor_name: None,
        actor_email: None,
        target_user_id: Some(user_id),
        target_resource_id: None,
        action: "assign_permission",
        details: Some(
            serde_json::json!({ "permission": req.permission, "expires_at": req.expires_at }),
        ),
        headers: &headers,
    })
    .await;

    if let Some(cache) = &state.infra.cache {
        let _ = cache
            .delete(&format!(
                "cache:auth:permissions:{}:{}",
                ctx.auth_token.workspace_id(),
                user_id
            ))
            .await;
    }

    Ok((StatusCode::CREATED, Json(perm)))
}

/// DELETE /workspaces/:ws_id/members/:user_id/permissions/:perm_id
/// Revoke an extra permission from a member.
pub async fn revoke_extra_permission(
    Path((_, user_id, perm_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let affected = state
        .core
        .workspace_repo
        .revoke_extra_permission(ctx.workspace_id, user_id, perm_id)
        .await?;

    if affected > 0 {
        AuditHelper::log(crate::services::audit_service::AuditLogArgs {
            repo: &state.core.audit_repo,
            workspace_id: ctx.workspace_id,
            actor_id: Some(ctx.user_id),
            actor_type: None,
            actor_name: None,
            actor_email: None,
            target_user_id: Some(user_id),
            target_resource_id: None,
            action: "revoke_permission",
            details: Some(serde_json::json!({ "permission_id": perm_id })),
            headers: &headers,
        })
        .await;

        if let Some(cache) = &state.infra.cache {
            let _ = cache
                .delete(&format!(
                    "cache:auth:permissions:{}:{}",
                    ctx.auth_token.workspace_id(),
                    user_id
                ))
                .await;
        }

        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Permission not found".to_string()))
    }
}

/// GET /workspaces/:ws_id/permissions/available
/// List all available permission strings.
pub async fn get_available_permissions(ctx: MemberContext) -> AppResult<Json<Vec<&'static str>>> {
    ctx.require_permission(Permission::MembersRolesManage)?;
    Ok(Json(Permission::all_permission_strings()))
}
