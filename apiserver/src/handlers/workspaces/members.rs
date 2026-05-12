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
use crate::models::{
    AddMemberRequest, MemberExtraRole, WorkspaceMemberDetail, WorkspaceMemberDetailResponse,
};
use crate::services::audit_service::AuditHelper;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role_id: Uuid,
}

pub async fn get_workspace_members(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<WorkspaceMemberDetailResponse>>> {
    let members = sqlx::query_as::<_, WorkspaceMemberDetail>(
        "SELECT wm.workspace_id, wm.user_id, wm.role_id, u.email, u.full_name, u.avatar_url
         FROM workspace_members wm
         JOIN users u ON wm.user_id = u.id
         WHERE wm.workspace_id = $1",
    )
    .bind(workspace_id)
    .fetch_all(state.infra.db.api())
    .await?;

    let extra_roles = sqlx::query_as::<_, MemberExtraRole>(
        "SELECT * FROM member_extra_roles WHERE workspace_id = $1 AND expires_at > NOW()",
    )
    .bind(workspace_id)
    .fetch_all(state.infra.db.api())
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

pub async fn add_workspace_member(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
    Json(req): Json<AddMemberRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersInvite)?;

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(state.infra.db.api())
        .await?
        .ok_or_else(|| AppError::NotFound("User with this email not found".to_string()))?;

    // 3. Add to workspace
    sqlx::query(
        "INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(req.role_id)
    .execute(state.infra.db.api())
    .await?;

    AuditHelper::log(crate::services::audit_service::AuditLogArgs {
        repo: &state.core.audit_repo,
        workspace_id: ctx.workspace_id,
        actor_id: Some(ctx.user_id),
        target_user_id: Some(user_id),
        target_resource_id: None,
        action: "add_member",
        details: Some(serde_json::json!({ "role_id": req.role_id })),
        headers: &headers,
    })
    .await;

    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = crate::cache::RedisCache::new(redis_pool);
        let _ = cache
            .delete(&format!(
                "cache:auth:permissions:{}:{}",
                ctx.workspace_id, user_id
            ))
            .await;
    }

    Ok(StatusCode::CREATED)
}

pub async fn update_workspace_member_role(
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let old_role_id = state
        .core
        .workspace_repo
        .find_primary_role(ctx.workspace_id, user_id)
        .await
        .unwrap_or(None);

    state
        .core
        .workspace_repo
        .update_member_role(ctx.workspace_id, user_id, req.role_id)
        .await?;

    AuditHelper::log(crate::services::audit_service::AuditLogArgs {
        repo: &state.core.audit_repo,
        workspace_id: ctx.workspace_id,
        actor_id: Some(ctx.user_id),
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

    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = crate::cache::RedisCache::new(redis_pool);
        let _ = cache
            .delete(&format!(
                "cache:auth:permissions:{}:{}",
                ctx.workspace_id, user_id
            ))
            .await;
    }

    Ok(StatusCode::OK)
}

pub async fn remove_workspace_member(
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRemove)?;

    // Don't allow removing yourself (prevent lockout if only one admin, though owner should probably remain)
    if user_id == ctx.user_id {
        return Err(AppError::Validation(
            "You cannot remove yourself from the workspace".to_string(),
        ));
    }

    state
        .core
        .workspace_repo
        .remove_member(ctx.workspace_id, user_id)
        .await?;

    AuditHelper::log(crate::services::audit_service::AuditLogArgs {
        repo: &state.core.audit_repo,
        workspace_id: ctx.workspace_id,
        actor_id: Some(ctx.user_id),
        target_user_id: Some(user_id),
        target_resource_id: None,
        action: "remove_member",
        details: None,
        headers: &headers,
    })
    .await;

    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = crate::cache::RedisCache::new(redis_pool);
        let _ = cache
            .delete(&format!(
                "cache:auth:permissions:{}:{}",
                ctx.workspace_id, user_id
            ))
            .await;
    }

    Ok(StatusCode::NO_CONTENT)
}
