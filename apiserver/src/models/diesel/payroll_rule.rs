use crate::models::enums::*;
use crate::models::payroll_rule::*;
use crate::schema::*;
use crate::utils::bigdecimal_helpers::bd_to_dec;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = payroll_rule_definitions)]
pub struct PayrollRuleDefinitionEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub category: String,
    pub applies_to: String,
    pub calculation_method: String,
    pub rule_config: serde_json::Value,
    pub is_mandatory: bool,
    pub is_active: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PayrollRuleDefinitionEntity> for PayrollRuleDefinition {
    fn from(entity: PayrollRuleDefinitionEntity) -> Self {
        use std::str::FromStr;
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            description: entity.description,
            code: entity.code,
            category: PayrollRuleCategory::from_str(&entity.category)
                .unwrap_or(PayrollRuleCategory::StatutoryDeduction),
            applies_to: RuleAppliesTo::from_str(&entity.applies_to)
                .unwrap_or(RuleAppliesTo::Employee),
            calculation_method: PayrollCalculationMethod::from_str(&entity.calculation_method)
                .unwrap_or(PayrollCalculationMethod::Percentage),
            rule_config: entity.rule_config,
            is_mandatory: entity.is_mandatory,
            is_active: entity.is_active,
            display_order: entity.display_order,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = payroll_rule_assignments)]
pub struct PayrollRuleAssignmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub rule_definition_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub override_config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<PayrollRuleAssignmentEntity> for PayrollRuleAssignment {
    fn from(entity: PayrollRuleAssignmentEntity) -> Self {
        use std::str::FromStr;
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            rule_definition_id: entity.rule_definition_id,
            assignee_type: AssigneeType::from_str(&entity.assignee_type)
                .unwrap_or(AssigneeType::Workspace),
            assignee_id: entity.assignee_id,
            effective_from: entity.effective_from,
            effective_to: entity.effective_to,
            override_config: entity.override_config,
            created_at: entity.created_at,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = payroll_rule_tiers)]
pub struct PayrollRuleTierEntity {
    pub id: Uuid,
    pub rule_definition_id: Uuid,
    pub tier_order: i32,
    pub min_amount: BigDecimal,
    pub max_amount: Option<BigDecimal>,
    pub rate: BigDecimal,
    pub fixed_amount: BigDecimal,
    pub created_at: DateTime<Utc>,
}

impl From<PayrollRuleTierEntity> for PayrollRuleTier {
    fn from(entity: PayrollRuleTierEntity) -> Self {
        Self {
            id: entity.id,
            rule_definition_id: entity.rule_definition_id,
            tier_order: entity.tier_order,
            min_amount: bd_to_dec(entity.min_amount),
            max_amount: entity.max_amount.map(bd_to_dec),
            rate: bd_to_dec(entity.rate),
            fixed_amount: bd_to_dec(entity.fixed_amount),
            created_at: entity.created_at,
        }
    }
}
