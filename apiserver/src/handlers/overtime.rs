use crate::{
    domain::permissions::Permission,
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::{OvertimePolicy, OvertimeRule},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub priority: i32,
    pub multiplier: rust_decimal::Decimal,
    pub time_start: Option<NaiveTime>,
    pub time_end: Option<NaiveTime>,
    pub days_of_week: i32,
    pub is_holiday: bool,
}

pub async fn list_policies(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<OvertimePolicy>>> {
    let policies = state
        .payroll
        .overtime_policy_repo
        .get_policy_definitions(ctx.workspace_id)
        .await?;
    Ok(Json(policies))
}

pub async fn get_policy(
    State(state): State<Arc<AppState>>,
    Path((_, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<(OvertimePolicy, Vec<OvertimeRule>)>> {
    let result = state
        .payroll
        .overtime_policy_repo
        .get_policy_with_rules(ctx.workspace_id, policy_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Policy not found".to_string()))?;
    Ok(Json(result))
}

pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreatePolicyRequest>,
) -> AppResult<Json<OvertimePolicy>> {
    ctx.require_permission(Permission::OvertimePoliciesManage)?;
    let policy = state
        .payroll
        .overtime_policy_repo
        .create_policy_definition(ctx.workspace_id, &req.name, req.description.as_deref())
        .await?;
    Ok(Json(policy))
}

pub async fn add_rule(
    State(state): State<Arc<AppState>>,
    Path((_workspace_id, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateRuleRequest>,
) -> AppResult<Json<OvertimeRule>> {
    ctx.require_permission(Permission::OvertimePoliciesManage)?;
    let rule = OvertimeRule {
        id: crate::utils::id::new_id(),
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

    let created = state.payroll.overtime_policy_repo.add_rule(ctx.workspace_id, &rule).await?;
    Ok(Json(created))
}

#[derive(Deserialize)]
pub struct AssignPolicyRequest {
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
}

pub async fn assign_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignPolicyRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::OvertimePoliciesManage)?;
    state
        .payroll
        .overtime_policy_repo
        .create_policy_assignment(
            ctx.workspace_id,
            &req.assignee_type,
            req.assignee_id,
            req.policy_id,
            req.effective_from,
        )
        .await?;
    Ok(StatusCode::OK)
}
