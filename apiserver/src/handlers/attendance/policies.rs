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
pub struct UpdateAttendanceRulesRequest {
    pub work_unit_minutes: Option<i32>,
    pub rounding_policy: Option<crate::models::RoundingPolicy>,
    /// ISO 8601 date string for when this version takes effect.
    pub valid_from: Option<String>,
    pub early_arrival_strategy: Option<crate::models::enums::EarlyArrivalStrategy>,
    pub grace_period_minutes: Option<i32>,
}

/// Response shape for a single policy version (includes metadata)
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AttendancePolicyVersionResponse {
    pub id: String,
    pub work_unit_minutes: i32,
    pub rounding_policy: crate::models::RoundingPolicy,
    pub early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy,
    pub grace_period_minutes: i32,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/rules",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Default attendance rules", body = AttendanceRules)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn get_default_attendance_rules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<AttendanceRules>> {
    // RBAC: reading workspace attendance rules requires WorkspaceSettingsRead
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;

    let rules = state
        .attendance()
        .policy_repo()
        .get_default_policy_rules(&ctx.auth_token)
        .await?;

    match rules {
        Some(r) => Ok(Json(r)),
        None => Ok(Json(AttendanceRules::default())),
    }
}

/// Returns all versions of the default attendance policy, ordered newest first.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/rules/versions",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Default attendance rules versions", body = Vec<AttendancePolicyVersionResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn list_default_attendance_policy_versions(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<Vec<AttendancePolicyVersionResponse>>> {
    // RBAC: reading policy version history requires WorkspaceSettingsRead
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;

    let versions = state
        .attendance()
        .policy_repo()
        .get_default_policy_versions(&ctx.auth_token)
        .await?;

    let response: Vec<AttendancePolicyVersionResponse> = versions
        .into_iter()
        .map(|v| AttendancePolicyVersionResponse {
            id: v.id.to_string(),
            work_unit_minutes: v.rules.work_unit_minutes,
            rounding_policy: v.rules.rounding_policy,
            early_arrival_strategy: v.rules.early_arrival_strategy,
            grace_period_minutes: v
                .rules
                .grace_period_minutes
                .unwrap_or(crate::domain::policy::defaults::DEFAULT_GRACE_PERIOD_MINUTES),
            valid_from: v.valid_from.to_rfc3339(),
            valid_to: v.valid_to.map(|dt| dt.to_rfc3339()),
            created_by: v.created_by.map(|u| u.to_string()),
            created_at: v.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = "/v1/workspaces/{ws_id}/attendance/rules",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = UpdateAttendanceRulesRequest,
    responses(
        (status = 200, description = "Default attendance rules updated", body = AttendancePolicyVersionResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn update_default_attendance_rules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(req): Json<UpdateAttendanceRulesRequest>,
) -> AppResult<Json<AttendancePolicyVersionResponse>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    let v = state
        .attendance()
        .policy_repo()
        .update_default_policy_rules(&ctx.auth_token, ctx.user_id, req)
        .await?;

    Ok(Json(AttendancePolicyVersionResponse {
        id: v.id.to_string(),
        work_unit_minutes: v.rules.work_unit_minutes,
        rounding_policy: v.rules.rounding_policy,
        early_arrival_strategy: v.rules.early_arrival_strategy,
        grace_period_minutes: v
            .rules
            .grace_period_minutes
            .unwrap_or(crate::domain::policy::defaults::DEFAULT_GRACE_PERIOD_MINUTES),
        valid_from: v.valid_from.to_rfc3339(),
        valid_to: v.valid_to.map(|dt| dt.to_rfc3339()),
        created_by: v.created_by.map(|u| u.to_string()),
        created_at: v.created_at.to_rfc3339(),
    }))
}
