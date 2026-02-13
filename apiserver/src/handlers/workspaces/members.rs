use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::AddMemberRequest;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: crate::models::MemberRole,
}

pub async fn get_workspace_members(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as::<_, crate::models::WorkspaceMemberDetail>(
        "SELECT wm.workspace_id, wm.user_id, wm.role, u.email, u.full_name, u.avatar_url
         FROM workspace_members wm
         JOIN users u ON wm.user_id = u.id
         WHERE wm.workspace_id = $1",
    )
    .bind(workspace_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(members) => Json(members).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn add_workspace_member(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can invite members").into_response();
    }

    let user_id: Uuid = match sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "User with this email not found").into_response();
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 3. Add to workspace
    match sqlx::query(
        "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(ctx.workspace_id)
    .bind(user_id)
    .bind(req.role)
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_workspace_member_role(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (
            StatusCode::FORBIDDEN,
            "Only admins can change roles".to_string(),
        )
            .into_response();
    }

    match state
        .workspace_repo
        .update_member_role(ctx.workspace_id, user_id, req.role)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn remove_workspace_member(
    Path((_ws_id, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (
            StatusCode::FORBIDDEN,
            "Only admins can remove members".to_string(),
        )
            .into_response();
    }

    // Don't allow removing yourself (prevent lockout if only one admin, though owner should probably remain)
    if user_id == ctx.user_id {
        return (
            StatusCode::BAD_REQUEST,
            "You cannot remove yourself from the workspace".to_string(),
        )
            .into_response();
    }

    match state
        .workspace_repo
        .remove_member(ctx.workspace_id, user_id)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}
