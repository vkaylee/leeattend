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
use crate::models::{AssignRoleRequest, MemberExtraRole, WorkspaceRole};
use crate::services::audit_service::AuditHelper;
use crate::services::WorkspaceService;
use crate::AppState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRolePermissionsRequest {
    pub permissions: Vec<String>,
}

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
        "INSERT INTO member_extra_roles (workspace_id, user_id, role_id, expires_at) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(req.role_id)
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
        Some(serde_json::json!({ "role_id": req.role_id, "expires_at": req.expires_at })),
        &headers,
    )
    .await;

    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = crate::cache::RedisCache::new(redis_pool);
        let _ = cache.delete(&format!("cache:auth:permissions:{}:{}", ctx.workspace_id, user_id)).await;
    }

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

        if let Some(redis_pool) = state.infra.redis.clone() {
            let cache = crate::cache::RedisCache::new(redis_pool);
            let _ = cache.delete(&format!("cache:auth:permissions:{}:{}", ctx.workspace_id, user_id)).await;
        }

        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Role not found".to_string()))
    }
}

// --- Workspace Role Management (CRUD) ---

pub async fn get_workspace_roles(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<WorkspaceRole>>> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let roles = sqlx::query_as::<_, WorkspaceRole>(
        "SELECT * FROM workspace_roles WHERE workspace_id = $1 OR is_system = true ORDER BY is_system DESC, created_at ASC"
    )
    .bind(ctx.workspace_id)
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(roles))
}

pub async fn create_workspace_role(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkspaceRoleRequest>,
) -> AppResult<(StatusCode, Json<WorkspaceRole>)> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let role = sqlx::query_as::<_, WorkspaceRole>(
        "INSERT INTO workspace_roles (workspace_id, name, description, color, is_system) VALUES ($1, $2, $3, $4, false) RETURNING *"
    )
    .bind(ctx.workspace_id)
    .bind(req.name)
    .bind(req.description)
    .bind(req.color)
    .fetch_one(state.infra.db.api())
    .await?;

    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn update_workspace_role(
    Path((_, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateWorkspaceRoleRequest>,
) -> AppResult<Json<WorkspaceRole>> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let role = sqlx::query_as::<_, WorkspaceRole>(
        "UPDATE workspace_roles SET name = COALESCE($1, name), description = COALESCE($2, description), color = COALESCE($3, color), updated_at = NOW() WHERE id = $4 AND workspace_id = $5 AND is_system = false RETURNING *"
    )
    .bind(req.name)
    .bind(req.description)
    .bind(req.color)
    .bind(role_id)
    .bind(ctx.workspace_id)
    .fetch_optional(state.infra.db.api())
    .await?
    .ok_or_else(|| AppError::NotFound("Role not found or is a system role".to_string()))?;

    Ok(Json(role))
}

pub async fn delete_workspace_role(
    Path((_, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let res = sqlx::query("DELETE FROM workspace_roles WHERE id = $1 AND workspace_id = $2 AND is_system = false")
        .bind(role_id)
        .bind(ctx.workspace_id)
        .execute(state.infra.db.api())
        .await?;

    if res.rows_affected() > 0 {
        if let Some(redis_pool) = state.infra.redis.clone() {
            let cache = crate::cache::RedisCache::new(redis_pool);
            let _ = cache.delete_pattern(&format!("cache:auth:permissions:{}:*", ctx.workspace_id)).await;
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Role not found or is a system role".to_string()))
    }
}

pub async fn get_role_permissions(
    Path((_, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<String>>> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let permissions: Vec<(String,)> = sqlx::query_as(
        "SELECT permission_name FROM workspace_role_permissions WHERE role_id = $1 AND role_id IN (SELECT id FROM workspace_roles WHERE workspace_id = $2 OR is_system = true)"
    )
    .bind(role_id)
    .bind(ctx.workspace_id)
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(permissions.into_iter().map(|(p,)| p).collect()))
}

pub async fn update_role_permissions(
    Path((_, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateRolePermissionsRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    // Verify role belongs to workspace (can't update system roles)
    let role_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM workspace_roles WHERE id = $1 AND workspace_id = $2 AND is_system = false)"
    )
    .bind(role_id)
    .bind(ctx.workspace_id)
    .fetch_one(state.infra.db.api())
    .await?;

    if !role_exists {
        return Err(AppError::NotFound("Role not found or is a system role".to_string()));
    }

    let mut tx = state.infra.db.api().begin().await?;

    sqlx::query("DELETE FROM workspace_role_permissions WHERE role_id = $1")
        .bind(role_id)
        .execute(&mut *tx)
        .await?;

    for perm in req.permissions {
        sqlx::query("INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, $2)")
            .bind(role_id)
            .bind(perm)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = crate::cache::RedisCache::new(redis_pool);
        let _ = cache.delete_pattern(&format!("cache:auth:permissions:{}:*", ctx.workspace_id)).await;
    }

    Ok(StatusCode::OK)
}

