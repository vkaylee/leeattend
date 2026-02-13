use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{AssignRoleRequest, MemberExtraRole};
use crate::services::WorkspaceService;
use crate::AppState;

pub async fn get_member_roles(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    match WorkspaceService::get_member_roles(
        state.workspace_repo.as_ref(),
        ctx.workspace_id,
        user_id,
    )
    .await
    {
        Ok((primary, extra)) => Json(serde_json::json!({
            "primary": primary,
            "extra": extra
        }))
        .into_response(),
        Err(e) if e == "NOT_FOUND" => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn assign_extra_role(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignRoleRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can assign roles").into_response();
    }

    if req.expires_at <= chrono::Utc::now() {
        return (
            StatusCode::BAD_REQUEST,
            "Expiration time must be in the future".to_string(),
        )
            .into_response();
    }

    match sqlx::query_as::<_, MemberExtraRole>(
        "INSERT INTO member_extra_roles (workspace_id, user_id, role, expires_at) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(req.role)
    .bind(req.expires_at)
    .fetch_one(&state.db)
    .await
    {
        Ok(extra_role) => (StatusCode::CREATED, Json(extra_role)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn revoke_extra_role(
    Path((_ws_id, _user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can revoke roles").into_response();
    }

    match sqlx::query("DELETE FROM member_extra_roles WHERE id = $1 AND workspace_id = $2")
        .bind(role_id)
        .bind(ctx.workspace_id)
        .execute(&state.db)
        .await
    {
        Ok(res) if res.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
