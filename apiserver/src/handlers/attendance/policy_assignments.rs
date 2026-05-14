use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::permissions::Permission, errors::AppResult, handlers::common::MemberContext,
    models::AttendanceRules, AppState,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PolicyAssignmentResponse {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub rules: AttendanceRules,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpsertPolicyAssignmentRequest {
    pub target_type: String, // "USER" or "GROUP"
    pub target_id: Uuid,
    pub work_unit_minutes: i32,
    pub rounding_policy: String,
    pub early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy,
    pub valid_from: Option<String>,
}

pub async fn list_policy_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<Vec<PolicyAssignmentResponse>>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;

    let assignments = state
        .attendance
        .policy_repo
        .list_policy_assignments(ctx.workspace_id)
        .await?;

    Ok(Json(assignments))
}

pub async fn upsert_policy_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(req): Json<UpsertPolicyAssignmentRequest>,
) -> AppResult<Json<PolicyAssignmentResponse>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    let res = state
        .attendance
        .policy_repo
        .upsert_policy_assignment(ctx.workspace_id, ctx.user_id, req)
        .await?;

    Ok(Json(res))
}

pub async fn delete_policy_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> AppResult<axum::http::StatusCode> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    state
        .attendance
        .policy_repo
        .delete_policy_assignment(ctx.workspace_id, assignment_id)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
