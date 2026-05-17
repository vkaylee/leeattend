use async_trait::async_trait;

use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    enums::{AssigneeType, PayrollCalculationMethod, PayrollRuleCategory, RuleAppliesTo},
    PayrollRuleAssignment, PayrollRuleDefinition, PayrollRuleTier,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PayrollRuleRepository: Send + Sync {
    async fn create_definition(
        &self,
        args: CreatePayrollRuleDefinitionArgs,
    ) -> Result<PayrollRuleDefinition, AppError>;

    async fn get_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollRuleDefinition>, AppError>;

    async fn create_tier(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
        tier_order: i32,
        min_amount: rust_decimal::Decimal,
        max_amount: Option<rust_decimal::Decimal>,
        rate: rust_decimal::Decimal,
        fixed_amount: rust_decimal::Decimal,
    ) -> Result<PayrollRuleTier, AppError>;

    async fn get_tiers_for_rule(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<PayrollRuleTier>, AppError>;

    async fn delete_tier(&self, workspace_id: Uuid, rule_definition_id: Uuid, tier_id: Uuid) -> Result<(), AppError>;

    async fn assign_rule(
        &self,
        args: AssignPayrollRuleArgs,
    ) -> Result<PayrollRuleAssignment, AppError>;

    async fn get_assignments_for_rule(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<PayrollRuleAssignment>, AppError>;

    async fn delete_assignment(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<(), AppError>;
}

pub struct CreatePayrollRuleDefinitionArgs {
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub category: PayrollRuleCategory,
    pub applies_to: RuleAppliesTo,
    pub calculation_method: PayrollCalculationMethod,
    pub rule_config: serde_json::Value,
    pub is_mandatory: bool,
    pub display_order: i32,
}

pub struct AssignPayrollRuleArgs {
    pub workspace_id: Uuid,
    pub rule_definition_id: Uuid,
    pub assignee_type: AssigneeType,
    pub assignee_id: Option<Uuid>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub override_config: Option<serde_json::Value>,
}
