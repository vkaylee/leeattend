use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{PayrollRuleAssignment, PayrollRuleDefinition, PayrollRuleTier, enums::{AssigneeType, PayrollCalculationMethod, PayrollRuleCategory, RuleAppliesTo}};

pub struct PayrollRuleRepository;

impl PayrollRuleRepository {
    #[allow(clippy::too_many_arguments)]
    pub async fn create_definition(
        pool: &PgPool,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        code: &str,
        category: PayrollRuleCategory,
        applies_to: RuleAppliesTo,
        calculation_method: PayrollCalculationMethod,
        rule_config: &serde_json::Value,
        is_mandatory: bool,
        display_order: i32,
    ) -> Result<PayrollRuleDefinition, AppError> {
        let row = sqlx::query_as!(
            PayrollRuleDefinition,
            r#"
            INSERT INTO payroll_rule_definitions (
                workspace_id, name, description, code, category, applies_to,
                calculation_method, rule_config, is_mandatory, display_order
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING 
                id, workspace_id, name, description, code, 
                category as "category: PayrollRuleCategory", 
                applies_to as "applies_to: RuleAppliesTo", 
                calculation_method as "calculation_method: PayrollCalculationMethod", 
                rule_config, is_mandatory, is_active, display_order, created_at, updated_at
            "#,
            workspace_id,
            name,
            description,
            code,
            category.to_string(),
            applies_to.to_string(),
            calculation_method.to_string(),
            rule_config,
            is_mandatory,
            display_order
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("payroll_rule_definitions_workspace_id_code_key") {
                    return AppError::Validation(
                        "A payroll rule with this code already exists in the workspace.".to_string(),
                    );
                }
            }
            AppError::Internal(format!("Failed to create payroll rule definition: {}", e))
        })?;

        Ok(row)
    }

    pub async fn get_definitions(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollRuleDefinition>, AppError> {
        let rows = sqlx::query_as!(
            PayrollRuleDefinition,
            r#"
            SELECT 
                id, workspace_id, name, description, code, 
                category as "category: PayrollRuleCategory", 
                applies_to as "applies_to: RuleAppliesTo", 
                calculation_method as "calculation_method: PayrollCalculationMethod", 
                rule_config, is_mandatory, is_active, display_order, created_at, updated_at
            FROM payroll_rule_definitions
            WHERE workspace_id = $1
            ORDER BY display_order ASC, created_at DESC
            "#,
            workspace_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch payroll rule definitions: {}", e)))?;

        Ok(rows)
    }

    pub async fn create_tier(
        pool: &PgPool,
        rule_definition_id: Uuid,
        tier_order: i32,
        min_amount: rust_decimal::Decimal,
        max_amount: Option<rust_decimal::Decimal>,
        rate: rust_decimal::Decimal,
        fixed_amount: rust_decimal::Decimal,
    ) -> Result<PayrollRuleTier, AppError> {
        let row = sqlx::query_as!(
            PayrollRuleTier,
            r#"
            INSERT INTO payroll_rule_tiers (
                rule_definition_id, tier_order, min_amount, max_amount, rate, fixed_amount
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            rule_definition_id,
            tier_order,
            min_amount,
            max_amount,
            rate,
            fixed_amount
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("payroll_rule_tiers_rule_definition_id_tier_order_key") {
                    return AppError::Validation(
                        "A tier with this order already exists for this rule.".to_string(),
                    );
                }
            }
            AppError::Internal(format!("Failed to create payroll rule tier: {}", e))
        })?;

        Ok(row)
    }

    pub async fn get_tiers_for_rule(
        pool: &PgPool,
        rule_definition_id: Uuid,
    ) -> Result<Vec<PayrollRuleTier>, AppError> {
        let rows = sqlx::query_as!(
            PayrollRuleTier,
            r#"
            SELECT * FROM payroll_rule_tiers
            WHERE rule_definition_id = $1
            ORDER BY tier_order ASC
            "#,
            rule_definition_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch payroll rule tiers: {}", e)))?;

        Ok(rows)
    }

    pub async fn delete_tier(
        pool: &PgPool,
        rule_definition_id: Uuid,
        tier_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM payroll_rule_tiers WHERE id = $1 AND rule_definition_id = $2",
            tier_id,
            rule_definition_id
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete tier: {}", e)))?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn assign_rule(
        pool: &PgPool,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
        assignee_type: AssigneeType,
        assignee_id: Option<Uuid>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        override_config: Option<&serde_json::Value>,
    ) -> Result<PayrollRuleAssignment, AppError> {
        let row = sqlx::query_as!(
            PayrollRuleAssignment,
            r#"
            INSERT INTO payroll_rule_assignments (
                workspace_id, rule_definition_id, assignee_type, assignee_id,
                effective_from, effective_to, override_config
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                id, workspace_id, rule_definition_id, 
                assignee_type as "assignee_type: AssigneeType", 
                assignee_id, effective_from, effective_to, 
                override_config, created_at
            "#,
            workspace_id,
            rule_definition_id,
            assignee_type.to_string(),
            assignee_id,
            effective_from,
            effective_to,
            override_config
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("payroll_rule_assignments_workspace_id_rule_definition_id_a_excl") {
                    return AppError::Conflict(
                        "This assignment overlaps with an existing assignment for the same rule and assignee in this time period.".to_string()
                    );
                }
            }
            AppError::Internal(format!("Failed to assign payroll rule: {}", e))
        })?;

        Ok(row)
    }

    pub async fn get_assignments_for_rule(
        pool: &PgPool,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<PayrollRuleAssignment>, AppError> {
        let rows = sqlx::query_as!(
            PayrollRuleAssignment,
            r#"
            SELECT 
                id, workspace_id, rule_definition_id, 
                assignee_type as "assignee_type: AssigneeType", 
                assignee_id, effective_from, effective_to, 
                override_config, created_at
            FROM payroll_rule_assignments
            WHERE workspace_id = $1 AND rule_definition_id = $2
            ORDER BY created_at DESC
            "#,
            workspace_id,
            rule_definition_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch assignments: {}", e)))?;

        Ok(rows)
    }

    pub async fn delete_assignment(
        pool: &PgPool,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM payroll_rule_assignments WHERE id = $1 AND workspace_id = $2 AND rule_definition_id = $3",
            assignment_id,
            workspace_id,
            rule_definition_id
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete assignment: {}", e)))?;

        Ok(())
    }
}
