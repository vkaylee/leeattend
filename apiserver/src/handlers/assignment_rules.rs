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

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/assignment-rules",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of assignment rules", body = Vec<AssignmentRule>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_assignment_rules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AssignmentRule>>> {
    ctx.require_permission(Permission::AssignmentRulesManage)?;

    let rules = state
        .shifts()
        .assignment_rule_repo()
        .get_rules(&ctx.auth_token)
        .await?;
    Ok(Json(rules))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/assignment-rules",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateAssignmentRuleRequest,
    responses(
        (status = 201, description = "Assignment rule created", body = AssignmentRule)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_assignment_rule(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateAssignmentRuleRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::AssignmentRulesManage)?;

    let rule = state
        .shifts()
        .assignment_rule_repo()
        .create_rule(&ctx.auth_token, &req)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.evict_local(ctx.workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(rule)))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/assignment-rules/{rule_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("rule_id" = Uuid, Path, description = "Assignment Rule ID")
    ),
    request_body = UpdateAssignmentRuleRequest,
    responses(
        (status = 200, description = "Assignment rule updated", body = AssignmentRule)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_assignment_rule(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateAssignmentRuleRequest>,
) -> AppResult<Json<AssignmentRule>> {
    let (_, rule_id) = __path_params;
    ctx.require_permission(Permission::AssignmentRulesManage)?;

    let rule = state
        .shifts()
        .assignment_rule_repo()
        .update_rule(&ctx.auth_token, rule_id, &req)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.evict_local(ctx.workspace_id).await;
    }

    Ok(Json(rule))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/assignment-rules/{rule_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("rule_id" = Uuid, Path, description = "Assignment Rule ID to delete")
    ),
    responses(
        (status = 204, description = "Assignment rule deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_assignment_rule(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_, rule_id) = __path_params;
    ctx.require_permission(Permission::AssignmentRulesManage)?;

    state
        .shifts()
        .assignment_rule_repo()
        .delete_rule(&ctx.auth_token, rule_id)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.evict_local(ctx.workspace_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}
