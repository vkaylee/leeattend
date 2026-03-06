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
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<MemberExtraPermission>>> {
    ctx.require_permission(Permission::MembersRead)?;

    let perms = sqlx::query_as::<_, MemberExtraPermission>(
        "SELECT * FROM workspace_member_extra_permissions WHERE workspace_id = $1 AND user_id = $2 AND expires_at > NOW() ORDER BY assigned_at DESC",
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .fetch_all(state.db.api())
    .await?;

    Ok(Json(perms))
}

/// POST /workspaces/:ws_id/members/:user_id/permissions
/// Grant an extra permission to a member.
pub async fn assign_extra_permission(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
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

    let perm = sqlx::query_as::<_, MemberExtraPermission>(
        "INSERT INTO workspace_member_extra_permissions (workspace_id, user_id, permission, expires_at, assigned_by) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(&req.permission)
    .bind(req.expires_at)
    .bind(ctx.user_id)
    .fetch_one(state.db.api())
    .await?;

    AuditHelper::log(
        &state.audit_repo,
        ctx.workspace_id,
        Some(ctx.user_id),
        Some(user_id),
        None,
        "assign_permission",
        Some(serde_json::json!({ "permission": req.permission, "expires_at": req.expires_at })),
        &headers,
    )
    .await;

    Ok((StatusCode::CREATED, Json(perm)))
}

/// DELETE /workspaces/:ws_id/members/:user_id/permissions/:perm_id
/// Revoke an extra permission from a member.
pub async fn revoke_extra_permission(
    Path((_ws_id, user_id, perm_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let res = sqlx::query(
        "DELETE FROM workspace_member_extra_permissions WHERE id = $1 AND workspace_id = $2",
    )
    .bind(perm_id)
    .bind(ctx.workspace_id)
    .execute(state.db.api())
    .await?;

    if res.rows_affected() > 0 {
        AuditHelper::log(
            &state.audit_repo,
            ctx.workspace_id,
            Some(ctx.user_id),
            Some(user_id),
            None,
            "revoke_permission",
            Some(serde_json::json!({ "permission_id": perm_id })),
            &headers,
        )
        .await;
        
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Permission not found".to_string()))
    }
}

/// GET /workspaces/:ws_id/permissions/available
/// List all available permission strings.
pub async fn get_available_permissions(
    ctx: MemberContext,
) -> AppResult<Json<Vec<&'static str>>> {
    ctx.require_permission(Permission::MembersRolesManage)?;
    Ok(Json(Permission::all_permission_strings()))
}
