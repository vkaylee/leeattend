use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{
    AssigneeType, PayrollCalculationMethod, PayrollRuleCategory, RuleAppliesTo,
};

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct PayrollRuleDefinition {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub category: PayrollRuleCategory,
    pub applies_to: RuleAppliesTo,
    pub calculation_method: PayrollCalculationMethod,
    pub rule_config: serde_json::Value,
    pub is_mandatory: bool,
    pub is_active: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct PayrollRuleTier {
    pub id: Uuid,
    pub rule_definition_id: Uuid,
    pub tier_order: i32,
    pub min_amount: rust_decimal::Decimal,
    pub max_amount: Option<rust_decimal::Decimal>,
    pub rate: rust_decimal::Decimal,
    pub fixed_amount: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct PayrollRuleAssignment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub rule_definition_id: Uuid,
    pub assignee_type: AssigneeType,
    pub assignee_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub override_config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
