use axum::{
    extract::{Path, State},
    Json, http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    AppState,
    models::{OvertimePolicy, OvertimeRule},
    repositories::overtime_policy_repository::OvertimePolicyRepository,
    handlers::common::MemberContext,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub priority: i32,
    pub multiplier: rust_decimal::Decimal,
    pub time_start: Option<chrono::NaiveTime>,
    pub time_end: Option<chrono::NaiveTime>,
    pub days_of_week: i32,
    pub is_holiday: bool,
}

pub async fn list_policies(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    _ctx: MemberContext,
) -> Result<Json<Vec<OvertimePolicy>>, StatusCode> {
    let policies = OvertimePolicyRepository::get_policy_definitions(&state.db, workspace_id)
        .await
        .map_err(|e| {
            tracing::error!("Error listing policies: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(policies))
}

pub async fn get_policy(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, policy_id)): Path<(Uuid, Uuid)>,
    _ctx: MemberContext,
) -> Result<Json<(OvertimePolicy, Vec<OvertimeRule>)>, StatusCode> {
    let result = OvertimePolicyRepository::get_policy_with_rules(&state.db, policy_id)
        .await
        .map_err(|e| {
            tracing::error!("Error getting policy: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(result))
}

pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreatePolicyRequest>,
) -> Result<Json<OvertimePolicy>, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    let policy = OvertimePolicyRepository::create_policy_definition(&state.db, workspace_id, &req.name, req.description.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Error creating policy: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(policy))
}

pub async fn add_rule(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateRuleRequest>,
) -> Result<Json<OvertimeRule>, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    let rule = OvertimeRule {
        id: Uuid::new_v4(),
        policy_id,
        priority: req.priority,
        multiplier: req.multiplier,
        time_start: req.time_start,
        time_end: req.time_end,
        days_of_week: req.days_of_week,
        is_holiday: req.is_holiday,
        created_at: None,
        updated_at: None,
    };

    let created = OvertimePolicyRepository::add_rule(&state.db, &rule)
        .await
        .map_err(|e| {
            tracing::error!("Error adding rule: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(created))
}

#[derive(Deserialize)]
pub struct AssignPolicyRequest {
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: chrono::NaiveDate,
}

pub async fn assign_policy(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<AssignPolicyRequest>,
) -> Result<StatusCode, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    OvertimePolicyRepository::create_policy_assignment(
        &state.db,
        workspace_id,
        &req.assignee_type,
        req.assignee_id,
        req.policy_id,
        req.effective_from,
    )
    .await
    .map_err(|e| {
        tracing::error!("Error assigning policy: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::OK)
}
