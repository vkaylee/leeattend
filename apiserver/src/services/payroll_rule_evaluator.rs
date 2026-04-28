use crate::errors::AppError;
use chrono::NaiveDate;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::PayrollRuleTier;
use crate::models::enums::{PayrollCalculationMethod, PayrollRuleCategory};

pub struct PayrollRuleEvaluator;

#[derive(Debug, Clone)]
pub struct RuleEvaluationResult {
    pub rule_code: String,
    pub amount: Decimal,
}

impl PayrollRuleEvaluator {
    pub async fn evaluate_rules(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        base_values: &std::collections::HashMap<String, Decimal>,
    ) -> Result<Vec<RuleEvaluationResult>, AppError> {
        let assignments = sqlx::query!(
            r#"
            WITH RECURSIVE resolved_assignments AS (
                SELECT DISTINCT ON (rule_definition_id)
                    rule_definition_id, override_config, assignee_type
                FROM payroll_rule_assignments
                WHERE workspace_id = $1
                  AND effective_from <= $5
                  AND (effective_to IS NULL OR effective_to >= $4)
                  AND (
                      (assignee_type = 'employee' AND assignee_id = $2)
                      OR (assignee_type = 'department' AND assignee_id = $3)
                      OR (assignee_type = 'workspace')
                  )
                ORDER BY rule_definition_id, 
                         CASE assignee_type 
                            WHEN 'employee' THEN 1 
                            WHEN 'department' THEN 2 
                            WHEN 'workspace' THEN 3 
                         END
            )
            SELECT 
                r.rule_definition_id,
                r.override_config,
                d.code,
                d.calculation_method as "calculation_method: PayrollCalculationMethod",
                d.rule_config as default_config,
                d.category as "category: PayrollRuleCategory"
            FROM resolved_assignments r
            JOIN payroll_rule_definitions d ON d.id = r.rule_definition_id
            WHERE d.is_active = true
            ORDER BY d.display_order
            "#,
            workspace_id,
            employee_id,
            department_id,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to resolve rules: {}", e)))?;

        let mut results = Vec::new();

        for assignment in assignments {
            let config = assignment.override_config.unwrap_or(assignment.default_config);
            let mut amount = Decimal::ZERO;

            match assignment.calculation_method {
                PayrollCalculationMethod::Percentage => {
                    let base_type = config.get("base").or_else(|| config.get("base_type")).and_then(|v| v.as_str()).unwrap_or("gross");
                    let current_base = base_values.get(base_type).copied().unwrap_or(Decimal::ZERO);

                    if let Some(rate_val) = config.get("rate").and_then(|r| r.as_f64()) {
                        let rate = Decimal::from_f64(rate_val).unwrap_or(Decimal::ZERO) / Decimal::from(100);
                        amount = current_base * rate;

                        if let Some(cap_val) = config.get("cap_amount").and_then(|c| c.as_f64()) {
                            let cap = Decimal::from_f64(cap_val).unwrap_or(Decimal::ZERO);
                            if amount > cap {
                                amount = cap;
                            }
                        }
                    }
                }
                PayrollCalculationMethod::Fixed => {
                    if let Some(fixed_val) = config.get("amount").and_then(|a| a.as_f64()) {
                        amount = Decimal::from_f64(fixed_val).unwrap_or(Decimal::ZERO);
                    }
                }
                PayrollCalculationMethod::Tiered => {
                    let tiers = sqlx::query_as!(
                        PayrollRuleTier,
                        "SELECT * FROM payroll_rule_tiers WHERE rule_definition_id = $1 ORDER BY tier_order ASC",
                        assignment.rule_definition_id
                    )
                    .fetch_all(pool)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to fetch tiers: {}", e)))?;

                    let base_type = config.get("base").or_else(|| config.get("base_type")).and_then(|v| v.as_str()).unwrap_or("gross");
                    let current_base = base_values.get(base_type).copied().unwrap_or(Decimal::ZERO);

                    let remaining = current_base;
                    for tier in tiers {
                        let tier_min = tier.min_amount;
                        if remaining <= tier_min {
                            continue;
                        }

                        let taxable_in_tier = if let Some(max) = tier.max_amount {
                            let max_val = std::cmp::min(remaining, max);
                            max_val - tier_min
                        } else {
                            remaining - tier_min
                        };

                        if taxable_in_tier > Decimal::ZERO {
                            amount += (taxable_in_tier * tier.rate) / Decimal::from(100) + tier.fixed_amount;
                        }
                    }
                }
            }

            if amount > Decimal::ZERO {
                results.push(RuleEvaluationResult {
                    rule_code: assignment.code,
                    amount,
                });
            }
        }

        Ok(results)
    }
}
