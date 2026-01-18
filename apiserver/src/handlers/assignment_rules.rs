use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{CreateAssignmentRuleRequest, UpdateAssignmentRuleRequest};
use crate::repositories::assignment_rule_repository::AssignmentRuleRepository;
use crate::AppState;

pub async fn get_assignment_rules(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage rules").into_response();
    }

    match AssignmentRuleRepository::get_rules(&state.db, ws_id).await {
        Ok(rules) => Json(rules).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_assignment_rule(
    Path(ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateAssignmentRuleRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage rules").into_response();
    }

    match AssignmentRuleRepository::create_rule(&state.db, ws_id, &req).await {
        Ok(rule) => (StatusCode::CREATED, Json(rule)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_assignment_rule(
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateAssignmentRuleRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage rules").into_response();
    }

    match AssignmentRuleRepository::update_rule(&state.db, rule_id, &req).await {
        Ok(rule) => Json(rule).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_assignment_rule(
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage rules").into_response();
    }

    match AssignmentRuleRepository::delete_rule(&state.db, rule_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
