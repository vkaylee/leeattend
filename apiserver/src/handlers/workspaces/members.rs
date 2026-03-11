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
use crate::models::{AddMemberRequest, WorkspaceMemberDetail};
use crate::services::audit_service::AuditHelper;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: crate::models::MemberRole,
}

pub async fn get_workspace_members(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<WorkspaceMemberDetail>>> {
    let members = sqlx::query_as::<_, WorkspaceMemberDetail>(
        "SELECT wm.workspace_id, wm.user_id, wm.role, u.email, u.full_name, u.avatar_url
         FROM workspace_members wm
         JOIN users u ON wm.user_id = u.id
         WHERE wm.workspace_id = $1",
    )
    .bind(workspace_id)
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(members))
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
        "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(req.role)
    .execute(state.infra.db.api())
    .await?;

    AuditHelper::log(
        &state.core.audit_repo,
        ctx.workspace_id,
        Some(ctx.user_id),
        Some(user_id),
        None,
        "add_member",
        Some(serde_json::json!({ "role": req.role })),
        &headers,
    )
    .await;

    Ok(StatusCode::CREATED)
}

pub async fn update_workspace_member_role(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    state
        .core.workspace_repo
        .update_member_role(ctx.workspace_id, user_id, req.role)
        .await
        .map_err(AppError::from)?;

    AuditHelper::log(
        &state.core.audit_repo,
        ctx.workspace_id,
        Some(ctx.user_id),
        Some(user_id),
        None,
        "update_member_role",
        Some(serde_json::json!({ "new_role": req.role })),
        &headers,
    )
    .await;

    Ok(StatusCode::OK)
}

pub async fn remove_workspace_member(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
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
        .core.workspace_repo
        .remove_member(ctx.workspace_id, user_id)
        .await
        .map_err(AppError::from)?;

    AuditHelper::log(
        &state.core.audit_repo,
        ctx.workspace_id,
        Some(ctx.user_id),
        Some(user_id),
        None,
        "remove_member",
        None,
        &headers,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
