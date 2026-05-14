use crate::errors::AppError;
use crate::models::enums::PayrollCalculationMethod;
use chrono::NaiveDate;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct PayrollRuleEvaluator;

#[derive(Debug, Clone)]
pub struct RuleEvaluationResult {
    pub rule_code: String,
    pub amount: Decimal,
}

impl PayrollRuleEvaluator {
    #[allow(clippy::too_many_lines)]
    pub async fn evaluate_rules(
        payroll_repo: &dyn crate::repositories::payroll_repository::PayrollRepository,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        base_values: &std::collections::HashMap<String, Decimal>,
    ) -> Result<Vec<RuleEvaluationResult>, AppError> {
        let assignments = payroll_repo
            .get_resolved_rule_assignments(
                workspace_id,
                employee_id,
                department_id,
                start_date,
                end_date,
            )
            .await?;

        let mut results = Vec::new();

        for assignment in assignments {
            let config = assignment
                .override_config
                .unwrap_or(assignment.default_config);
            let mut amount = Decimal::ZERO;

            match assignment.calculation_method {
                PayrollCalculationMethod::Percentage => {
                    let base_type = config
                        .get("base")
                        .or_else(|| config.get("base_type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("gross");
                    let current_base = base_values.get(base_type).copied().unwrap_or(Decimal::ZERO);

                    if let Some(rate_val) = config.get("rate").and_then(serde_json::Value::as_f64) {
                        let rate = Decimal::from_f64(rate_val).unwrap_or(Decimal::ZERO)
                            / Decimal::from(100);
                        amount = current_base * rate;

                        if let Some(cap_val) =
                            config.get("cap_amount").and_then(serde_json::Value::as_f64)
                        {
                            let cap = Decimal::from_f64(cap_val).unwrap_or(Decimal::ZERO);
                            if amount > cap {
                                amount = cap;
                            }
                        }
                    }
                }
                PayrollCalculationMethod::Fixed => {
                    if let Some(fixed_val) =
                        config.get("amount").and_then(serde_json::Value::as_f64)
                    {
                        amount = Decimal::from_f64(fixed_val).unwrap_or(Decimal::ZERO);
                    }
                }
                PayrollCalculationMethod::Tiered => {
                    let tiers = payroll_repo
                        .get_payroll_rule_tiers(assignment.rule_definition_id)
                        .await?;

                    let base_type = config
                        .get("base")
                        .or_else(|| config.get("base_type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("gross");
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
                            amount += (taxable_in_tier * tier.rate) / Decimal::from(100)
                                + tier.fixed_amount;
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
