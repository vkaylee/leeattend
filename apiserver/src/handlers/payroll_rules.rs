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
    let def = state
        .payroll
        .rule_repo
        .create_definition(
            crate::repositories::payroll_rule_repository::CreatePayrollRuleDefinitionArgs {
                workspace_id: ctx.workspace_id,
                name: payload.name,
                description: payload.description,
                code: payload.code,
                category: payload.category,
                applies_to: payload.applies_to,
                calculation_method: payload.calculation_method,
                rule_config: payload.rule_config,
                is_mandatory: payload.is_mandatory.unwrap_or(false),
                display_order: payload.display_order.unwrap_or(0),
            },
        )
        .await?;

    Ok(Json(def))
}

pub async fn get_payroll_rule_definitions(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<Json<Vec<PayrollRuleDefinition>>, AppError> {
    let defs = state
        .payroll
        .rule_repo
        .get_definitions(ctx.workspace_id)
        .await?;
    Ok(Json(defs))
}

pub async fn create_payroll_rule_tier(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreatePayrollRuleTierRequest>,
) -> Result<Json<PayrollRuleTier>, AppError> {
    let tier = state
        .payroll
        .rule_repo
        .create_tier(
            ctx.workspace_id,
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
    ctx: MemberContext,
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<PayrollRuleTier>>, AppError> {
    let tiers = state
        .payroll
        .rule_repo
        .get_tiers_for_rule(ctx.workspace_id, rule_id)
        .await?;
    Ok(Json(tiers))
}

pub async fn assign_payroll_rule(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignPayrollRuleRequest>,
) -> Result<Json<PayrollRuleAssignment>, AppError> {
    let assignment = state
        .payroll
        .rule_repo
        .assign_rule(
            crate::repositories::payroll_rule_repository::AssignPayrollRuleArgs {
                workspace_id: ctx.workspace_id,
                rule_definition_id: rule_id,
                assignee_type: payload.assignee_type,
                assignee_id: payload.assignee_id,
                effective_from: payload.effective_from,
                effective_to: payload.effective_to,
                override_config: payload.override_config,
            },
        )
        .await?;

    Ok(Json(assignment))
}

pub async fn get_payroll_rule_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<PayrollRuleAssignment>>, AppError> {
    let assignments = state
        .payroll
        .rule_repo
        .get_assignments_for_rule(ctx.workspace_id, rule_id)
        .await?;
    Ok(Json(assignments))
}

pub async fn delete_payroll_rule_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, rule_id, assignment_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<()>, AppError> {
    state
        .payroll
        .rule_repo
        .delete_assignment(ctx.workspace_id, rule_id, assignment_id)
        .await?;
    Ok(Json(()))
}

pub async fn delete_payroll_rule_tier(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, rule_id, tier_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<()>, AppError> {
    state
        .payroll
        .rule_repo
        .delete_tier(ctx.workspace_id, rule_id, tier_id)
        .await?;
    Ok(Json(()))
}
