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

#[derive(Deserialize, utoipa::ToSchema)]
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreatePayrollRuleTierRequest {
    pub tier_order: i32,
    pub min_amount: Decimal,
    pub max_amount: Option<Decimal>,
    pub rate: Decimal,
    pub fixed_amount: Option<Decimal>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AssignPayrollRuleRequest {
    pub assignee_type: AssigneeType,
    pub assignee_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub override_config: Option<serde_json::Value>,
}

/// POST /workspaces/{ws_id}/payroll-rules
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/payroll-rules",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreatePayrollRuleDefinitionRequest,
    responses(
        (status = 200, description = "Created payroll rule definition", body = PayrollRuleDefinition)
    )
)]
pub async fn create_payroll_rule_definition(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<CreatePayrollRuleDefinitionRequest>,
) -> Result<Json<PayrollRuleDefinition>, AppError> {
    let def = state
        .payroll()
        .rule_repo()
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

/// GET /workspaces/{ws_id}/payroll-rules
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/payroll-rules",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Payroll rule definitions list", body = [PayrollRuleDefinition])
    )
)]
pub async fn get_payroll_rule_definitions(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<Json<Vec<PayrollRuleDefinition>>, AppError> {
    let defs = state
        .payroll()
        .rule_repo()
        .get_definitions(ctx.workspace_id)
        .await?;
    Ok(Json(defs))
}

/// POST /workspaces/{ws_id}/payroll-rules/{rule_id}/tiers
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("rule_id" = Uuid, Path, description = "Rule Definition ID")
    ),
    request_body = CreatePayrollRuleTierRequest,
    responses(
        (status = 200, description = "Created payroll rule tier", body = PayrollRuleTier)
    )
)]
pub async fn create_payroll_rule_tier(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreatePayrollRuleTierRequest>,
) -> Result<Json<PayrollRuleTier>, AppError> {
    let (_ws_id, rule_id) = params;
    let tier = state
        .payroll()
        .rule_repo()
        .create_tier(
            ctx.workspace_id,
            rule_id,
            payload.tier_order,
            payload.min_amount,
            payload.max_amount,
            payload.rate,
            payload.fixed_amount.unwrap_or(Decimal::ZERO),
        )
        .await?;

    Ok(Json(tier))
}

/// GET /workspaces/{ws_id}/payroll-rules/{rule_id}/tiers
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("rule_id" = Uuid, Path, description = "Rule Definition ID")
    ),
    responses(
        (status = 200, description = "Payroll rule tiers list", body = [PayrollRuleTier])
    )
)]
pub async fn get_payroll_rule_tiers(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<PayrollRuleTier>>, AppError> {
    let (_ws_id, rule_id) = params;
    let tiers = state
        .payroll()
        .rule_repo()
        .get_tiers_for_rule(ctx.workspace_id, rule_id)
        .await?;
    Ok(Json(tiers))
}

/// POST /workspaces/{ws_id}/payroll-rules/{rule_id}/assignments
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("rule_id" = Uuid, Path, description = "Rule Definition ID")
    ),
    request_body = AssignPayrollRuleRequest,
    responses(
        (status = 200, description = "Assigned payroll rule", body = PayrollRuleAssignment)
    )
)]
pub async fn assign_payroll_rule(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignPayrollRuleRequest>,
) -> Result<Json<PayrollRuleAssignment>, AppError> {
    let (_ws_id, rule_id) = params;
    let assignment = state
        .payroll()
        .rule_repo()
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

/// GET /workspaces/{ws_id}/payroll-rules/{rule_id}/assignments
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("rule_id" = Uuid, Path, description = "Rule Definition ID")
    ),
    responses(
        (status = 200, description = "Payroll rule assignments list", body = [PayrollRuleAssignment])
    )
)]
pub async fn get_payroll_rule_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<PayrollRuleAssignment>>, AppError> {
    let (_ws_id, rule_id) = params;
    let assignments = state
        .payroll()
        .rule_repo()
        .get_assignments_for_rule(ctx.workspace_id, rule_id)
        .await?;
    Ok(Json(assignments))
}

/// DELETE /workspaces/{ws_id}/payroll-rules/{rule_id}/assignments/{assignment_id}
#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments/{assignment_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("rule_id" = Uuid, Path, description = "Rule Definition ID"),
        ("assignment_id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 200, description = "Deleted payroll rule assignment")
    )
)]
pub async fn delete_payroll_rule_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<()>, AppError> {
    let (_ws_id, rule_id, assignment_id) = params;
    state
        .payroll()
        .rule_repo()
        .delete_assignment(ctx.workspace_id, rule_id, assignment_id)
        .await?;
    Ok(Json(()))
}

/// DELETE /workspaces/{ws_id}/payroll-rules/{rule_id}/tiers/{tier_id}
#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers/{tier_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("rule_id" = Uuid, Path, description = "Rule Definition ID"),
        ("tier_id" = Uuid, Path, description = "Tier ID")
    ),
    responses(
        (status = 200, description = "Deleted payroll rule tier")
    )
)]
pub async fn delete_payroll_rule_tier(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<()>, AppError> {
    let (_ws_id, rule_id, tier_id) = params;
    state
        .payroll()
        .rule_repo()
        .delete_tier(ctx.workspace_id, rule_id, tier_id)
        .await?;
    Ok(Json(()))
}
