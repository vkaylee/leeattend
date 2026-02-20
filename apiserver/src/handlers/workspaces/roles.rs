use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{AssignRoleRequest, MemberExtraRole};
use crate::services::WorkspaceService;
use crate::AppState;

pub async fn get_member_roles(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    match WorkspaceService::get_member_roles(
        state.workspace_repo.as_ref(),
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
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignRoleRequest>,
) -> AppResult<(StatusCode, Json<MemberExtraRole>)> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can assign roles".to_string(),
        ));
    }

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
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(extra_role)))
}

pub async fn revoke_extra_role(
    Path((_ws_id, _user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can revoke roles".to_string(),
        ));
    }

    let res = sqlx::query("DELETE FROM member_extra_roles WHERE id = $1 AND workspace_id = $2")
        .bind(role_id)
        .bind(ctx.workspace_id)
        .execute(&state.db)
        .await?;

    if res.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Role not found".to_string()))
    }
}
