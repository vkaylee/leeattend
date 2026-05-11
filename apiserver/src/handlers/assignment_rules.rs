use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{AssignmentRule, CreateAssignmentRuleRequest, UpdateAssignmentRuleRequest};
use crate::AppState;

pub async fn get_assignment_rules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AssignmentRule>>> {
    ctx.require_permission(Permission::AssignmentRulesManage)?;

    let rules = state.shifts.assignment_rule_repo.get_rules(ctx.workspace_id).await?;
    Ok(Json(rules))
}

pub async fn create_assignment_rule(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateAssignmentRuleRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::AssignmentRulesManage)?;

    let rule = state
        .shifts
        .assignment_rule_repo
        .create_rule(ctx.workspace_id, &req)
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(rule)))
}

pub async fn update_assignment_rule(
    Path((_, rule_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateAssignmentRuleRequest>,
) -> AppResult<Json<AssignmentRule>> {
    ctx.require_permission(Permission::AssignmentRulesManage)?;

    let rule = state
        .shifts
        .assignment_rule_repo
        .update_rule(ctx.workspace_id, rule_id, &req)
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(rule))
}

pub async fn delete_assignment_rule(
    Path((_, rule_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::AssignmentRulesManage)?;

    state
        .shifts
        .assignment_rule_repo
        .delete_rule(ctx.workspace_id, rule_id)
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}
