use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{AssignRoleRequest, MemberExtraRole};
use crate::services::audit_service::AuditHelper;
use crate::services::WorkspaceService;
use crate::AppState;

pub async fn get_member_roles(
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    match WorkspaceService::get_member_roles(
        state.core.workspace_repo.as_ref(),
        ctx.workspace_id,
        user_id,
    )
    .await
    {
        Ok((primary, extra)) => Ok(Json(serde_json::json!({
            "primary": primary,
            "extra": extra
        }))),
        Err(e) if e == "NOT_FOUND" => Err(AppError::NotFound("Member not found".to_string())),
        Err(e) => Err(AppError::from(e)),
    }
}

pub async fn assign_extra_role(
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
    Json(req): Json<AssignRoleRequest>,
) -> AppResult<(StatusCode, Json<MemberExtraRole>)> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    if req.expires_at <= chrono::Utc::now() {
        return Err(AppError::Validation(
            "Expiration time must be in the future".to_string(),
        ));
    }

    let extra_role = sqlx::query_as::<_, MemberExtraRole>(
        "INSERT INTO member_extra_roles (workspace_id, user_id, role, expires_at) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(req.role)
    .bind(req.expires_at)
    .fetch_one(state.infra.db.api())
    .await?;

    AuditHelper::log(
        &state.core.audit_repo,
        ctx.workspace_id,
        Some(ctx.user_id),
        Some(user_id),
        Some(extra_role.id),
        "assign_extra_role",
        Some(serde_json::json!({ "role": req.role, "expires_at": req.expires_at })),
        &headers,
    )
    .await;

    Ok((StatusCode::CREATED, Json(extra_role)))
}

pub async fn revoke_extra_role(
    Path((_, user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let res = sqlx::query("DELETE FROM member_extra_roles WHERE id = $1 AND workspace_id = $2")
        .bind(role_id)
        .bind(ctx.workspace_id)
        .execute(state.infra.db.api())
        .await?;

    if res.rows_affected() > 0 {
        AuditHelper::log(
            &state.core.audit_repo,
            ctx.workspace_id,
            Some(ctx.user_id),
            Some(user_id),
            Some(role_id),
            "revoke_extra_role",
            None,
            &headers,
        )
        .await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Role not found".to_string()))
    }
}
