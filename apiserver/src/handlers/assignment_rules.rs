use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{AssignmentRule, CreateAssignmentRuleRequest, UpdateAssignmentRuleRequest};
use crate::repositories::assignment_rule_repository::AssignmentRuleRepository;
use crate::AppState;

pub async fn get_assignment_rules(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AssignmentRule>>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage rules".to_string(),
        ));
    }

    let rules = AssignmentRuleRepository::get_rules(&state.db, ws_id).await?;
    Ok(Json(rules))
}

pub async fn create_assignment_rule(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateAssignmentRuleRequest>,
) -> AppResult<impl IntoResponse> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage rules".to_string(),
        ));
    }

    let rule = AssignmentRuleRepository::create_rule(&state.db, ws_id, &req).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ws_id).await;
    }

    Ok((StatusCode::CREATED, Json(rule)))
}

pub async fn update_assignment_rule(
    Path((ws_id, rule_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateAssignmentRuleRequest>,
) -> AppResult<Json<AssignmentRule>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage rules".to_string(),
        ));
    }

    let rule = AssignmentRuleRepository::update_rule(&state.db, rule_id, &req).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ws_id).await;
    }

    Ok(Json(rule))
}

pub async fn delete_assignment_rule(
    Path((ws_id, rule_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage rules".to_string(),
        ));
    }

    AssignmentRuleRepository::delete_rule(&state.db, rule_id).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ws_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}
