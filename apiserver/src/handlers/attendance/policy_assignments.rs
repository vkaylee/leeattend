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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PolicyAssignmentResponse {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub rules: AttendanceRules,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpsertPolicyAssignmentRequest {
    pub target_type: String, // "USER" or "GROUP"
    pub target_id: Uuid,
    pub work_unit_minutes: i32,
    pub rounding_policy: crate::models::RoundingPolicy,
    pub early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy,
    pub grace_period_minutes: Option<i32>,
    pub valid_from: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/policies/assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Policy assignments", body = Vec<PolicyAssignmentResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn list_policy_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<Vec<PolicyAssignmentResponse>>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;

    let assignments = state
        .attendance()
        .policy_repo()
        .list_policy_assignments(&ctx.auth_token)
        .await?;

    Ok(Json(assignments))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/policies/assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = UpsertPolicyAssignmentRequest,
    responses(
        (status = 200, description = "Policy assignment created/updated", body = PolicyAssignmentResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn upsert_policy_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(req): Json<UpsertPolicyAssignmentRequest>,
) -> AppResult<Json<PolicyAssignmentResponse>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    let res = state
        .attendance()
        .policy_repo()
        .upsert_policy_assignment(&ctx.auth_token, ctx.user_id, req)
        .await?;

    Ok(Json(res))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/attendance/policies/assignments/{assignment_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("assignment_id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 204, description = "Policy assignment deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, assignment_id = %__path_params.1))]
pub async fn delete_policy_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<axum::http::StatusCode> {
    let (_ws_id, assignment_id) = __path_params;
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    state
        .attendance()
        .policy_repo()
        .delete_policy_assignment(&ctx.auth_token, assignment_id)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
