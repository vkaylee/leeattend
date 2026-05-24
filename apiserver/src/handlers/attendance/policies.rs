#![allow(unused_variables)]
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
pub struct UpdateAttendanceRulesRequest {
    pub work_unit_minutes: Option<i32>,
    pub rounding_policy: Option<String>,
    /// ISO 8601 date string for when this version takes effect.
    pub valid_from: Option<String>,
    pub early_arrival_strategy: Option<crate::models::enums::EarlyArrivalStrategy>,
}

/// Response shape for a single policy version (includes metadata)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AttendancePolicyVersionResponse {
    pub id: String,
    pub work_unit_minutes: i32,
    pub rounding_policy: String,
    pub early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
}

pub async fn get_default_attendance_rules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<AttendanceRules>> {
    let workspace_id = ctx.workspace_id;

    let rules = state
        .attendance
        .policy_repo
        .get_default_policy_rules(workspace_id)
        .await?;

    match rules {
        Some(r) => Ok(Json(r)),
        None => Ok(Json(AttendanceRules {
            work_unit_minutes: 15,
            rounding_policy: "none".to_string(),
            early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
        })),
    }
}

/// Returns all versions of the default attendance policy, ordered newest first.
pub async fn list_default_attendance_policy_versions(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<Vec<AttendancePolicyVersionResponse>>> {
    let workspace_id = ctx.workspace_id;

    let versions = state
        .attendance
        .policy_repo
        .get_default_policy_versions(workspace_id)
        .await?;

    let response: Vec<AttendancePolicyVersionResponse> = versions
        .into_iter()
        .map(|v| AttendancePolicyVersionResponse {
            id: v.id.to_string(),
            work_unit_minutes: v.rules.work_unit_minutes,
            rounding_policy: v.rules.rounding_policy,
            early_arrival_strategy: v.rules.early_arrival_strategy,
            valid_from: v.valid_from.to_rfc3339(),
            valid_to: v.valid_to.map(|dt| dt.to_rfc3339()),
            created_by: v.created_by.map(|u| u.to_string()),
            created_at: v.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

#[allow(clippy::too_many_lines)]
pub async fn update_default_attendance_rules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(req): Json<UpdateAttendanceRulesRequest>,
) -> AppResult<Json<AttendancePolicyVersionResponse>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    let v = state
        .attendance
        .policy_repo
        .update_default_policy_rules(ctx.workspace_id, ctx.user_id, req)
        .await?;

    Ok(Json(AttendancePolicyVersionResponse {
        id: v.id.to_string(),
        work_unit_minutes: v.rules.work_unit_minutes,
        rounding_policy: v.rules.rounding_policy,
        early_arrival_strategy: v.rules.early_arrival_strategy,
        valid_from: v.valid_from.to_rfc3339(),
        valid_to: v.valid_to.map(|dt| dt.to_rfc3339()),
        created_by: v.created_by.map(|u| u.to_string()),
        created_at: v.created_at.to_rfc3339(),
    }))
}
