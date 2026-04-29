use axum::{
    extract::{Path, State},
    Json,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    handlers::common::MemberContext,
    models::{
        enums::{AssigneeType, PayrollCalculationMethod, PayrollRuleCategory, RuleAppliesTo},
        PayrollRuleAssignment, PayrollRuleDefinition, PayrollRuleTier,
    },
    repositories::PayrollRuleRepository,
    AppState,
};

#[derive(Deserialize)]
pub struct CreatePayrollRuleDefinitionRequest {
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub category: PayrollRuleCategory,
    pub applies_to: RuleAppliesTo,
    pub calculation_method: PayrollCalculationMethod,
    pub rule_config: serde_json::Value,
    pub is_mandatory: Option<bool>,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreatePayrollRuleTierRequest {
    pub tier_order: i32,
    pub min_amount: Decimal,
    pub max_amount: Option<Decimal>,
    pub rate: Decimal,
    pub fixed_amount: Option<Decimal>,
}

#[derive(Deserialize)]
pub struct AssignPayrollRuleRequest {
    pub assignee_type: AssigneeType,
    pub assignee_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub override_config: Option<serde_json::Value>,
}

pub async fn create_payroll_rule_definition(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<CreatePayrollRuleDefinitionRequest>,
) -> Result<Json<PayrollRuleDefinition>, AppError> {
    let def = PayrollRuleRepository::create_definition(
        state.infra.db.api(),
        ctx.workspace_id,
        &payload.name,
        payload.description.as_deref(),
        &payload.code,
        payload.category,
        payload.applies_to,
        payload.calculation_method,
        &payload.rule_config,
        payload.is_mandatory.unwrap_or(false),
        payload.display_order.unwrap_or(0),
    )
    .await?;

    Ok(Json(def))
}

pub async fn get_payroll_rule_definitions(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<Json<Vec<PayrollRuleDefinition>>, AppError> {
    let defs =
        PayrollRuleRepository::get_definitions(state.infra.db.api(), ctx.workspace_id).await?;
    Ok(Json(defs))
}

pub async fn create_payroll_rule_tier(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreatePayrollRuleTierRequest>,
) -> Result<Json<PayrollRuleTier>, AppError> {
    // Note: A robust system should verify that `rule_id` belongs to `ws_id`.
    // In our repository pattern we might either check that first, or pass ws_id down.

    let tier = PayrollRuleRepository::create_tier(
        state.infra.db.api(),
        rule_id,
        payload.tier_order,
        payload.min_amount,
        payload.max_amount,
        payload.rate,
        payload.fixed_amount.unwrap_or(Decimal::new(0, 0)),
    )
    .await?;

    Ok(Json(tier))
}

pub async fn get_payroll_rule_tiers(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<PayrollRuleTier>>, AppError> {
    let tiers = PayrollRuleRepository::get_tiers_for_rule(state.infra.db.api(), rule_id).await?;
    Ok(Json(tiers))
}

pub async fn assign_payroll_rule(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignPayrollRuleRequest>,
) -> Result<Json<PayrollRuleAssignment>, AppError> {
    let assignment = PayrollRuleRepository::assign_rule(
        state.infra.db.api(),
        ctx.workspace_id,
        rule_id,
        payload.assignee_type,
        payload.assignee_id,
        payload.effective_from,
        payload.effective_to,
        payload.override_config.as_ref(),
    )
    .await?;

    Ok(Json(assignment))
}

pub async fn get_payroll_rule_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<PayrollRuleAssignment>>, AppError> {
    let assignments = PayrollRuleRepository::get_assignments_for_rule(
        state.infra.db.api(),
        ctx.workspace_id,
        rule_id,
    )
    .await?;
    Ok(Json(assignments))
}

pub async fn delete_payroll_rule_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, rule_id, assignment_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<()>, AppError> {
    PayrollRuleRepository::delete_assignment(
        state.infra.db.api(),
        ctx.workspace_id,
        rule_id,
        assignment_id,
    )
    .await?;
    Ok(Json(()))
}

pub async fn delete_payroll_rule_tier(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, rule_id, tier_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<()>, AppError> {
    PayrollRuleRepository::delete_tier(state.infra.db.api(), rule_id, tier_id).await?;
    Ok(Json(()))
}
