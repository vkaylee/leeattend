use crate::errors::AppError;
use crate::models::enums::PayrollCalculationMethod;
use crate::services::financial_calculator::RuleAssignmentWithTiers;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

pub struct PayrollRuleEvaluator;

#[derive(Debug, Clone)]
pub struct RuleEvaluationResult {
    pub rule_code: String,
    pub amount: Decimal,
}

impl PayrollRuleEvaluator {
    #[allow(clippy::too_many_lines)]
    pub fn evaluate_rules(
        rule_assignments: &[RuleAssignmentWithTiers],
        base_values: &std::collections::HashMap<String, Decimal>,
    ) -> Result<Vec<RuleEvaluationResult>, AppError> {
        let mut results = Vec::new();

        for assignment_with_tiers in rule_assignments {
            let assignment = &assignment_with_tiers.assignment;
            let config = assignment
                .override_config
                .as_ref()
                .unwrap_or(&assignment.default_config);
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
                    let tiers = &assignment_with_tiers.tiers;

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
                    rule_code: assignment.code.clone(),
                    amount,
                });
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::PayrollRuleCategory;
    use crate::repositories::payroll_repository::ResolvedRuleAssignment;
    use serde_json::json;
    use std::collections::HashMap;
    use uuid::Uuid;

    // Direct in-memory test setup

    fn make_assignment_with_tiers(
        method: PayrollCalculationMethod,
        config: serde_json::Value,
        tiers: Vec<crate::models::PayrollRuleTier>,
    ) -> RuleAssignmentWithTiers {
        RuleAssignmentWithTiers {
            assignment: ResolvedRuleAssignment {
                rule_definition_id: Uuid::new_v4(),
                override_config: None,
                code: "TEST_RULE".into(),
                calculation_method: method,
                default_config: config,
                category: PayrollRuleCategory::Allowance,
            },
            tiers,
        }
    }

    fn base_map(gross: f64) -> HashMap<String, Decimal> {
        let mut m = HashMap::new();
        m.insert("gross".into(), Decimal::from_f64(gross).unwrap());
        m
    }

    // Percentage

    #[test]
    fn test_percentage_rule_basic() {
        let assignment = make_assignment_with_tiers(
            PayrollCalculationMethod::Percentage,
            json!({"rate": 10.0, "base": "gross"}),
            vec![],
        );
        let base = base_map(10_000_000.0);
        let results = PayrollRuleEvaluator::evaluate_rules(&[assignment], &base).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_code, "TEST_RULE");
        // 10% of 10_000_000 = 1_000_000
        assert_eq!(results[0].amount, Decimal::from(1_000_000));
    }

    #[test]
    fn test_percentage_rule_with_cap() {
        let assignment = make_assignment_with_tiers(
            PayrollCalculationMethod::Percentage,
            json!({"rate": 50.0, "base": "gross", "cap_amount": 500_000.0}),
            vec![],
        );
        let base = base_map(5_000_000.0);
        let results = PayrollRuleEvaluator::evaluate_rules(&[assignment], &base).unwrap();
        // 50% of 5M = 2.5M but capped at 500K
        assert_eq!(results[0].amount, Decimal::from(500_000));
    }

    #[test]
    fn test_percentage_rule_zero_result_not_included() {
        // rate = 0 -> amount stays 0 -> not pushed to results
        let assignment = make_assignment_with_tiers(
            PayrollCalculationMethod::Percentage,
            json!({"rate": 0.0, "base": "gross"}),
            vec![],
        );
        let base = base_map(10_000_000.0);
        let results = PayrollRuleEvaluator::evaluate_rules(&[assignment], &base).unwrap();
        assert!(
            results.is_empty(),
            "Zero-amount results should be filtered out"
        );
    }

    // Fixed

    #[test]
    fn test_fixed_rule() {
        let assignment = make_assignment_with_tiers(
            PayrollCalculationMethod::Fixed,
            json!({"amount": 2_000_000.0}),
            vec![],
        );
        let results = PayrollRuleEvaluator::evaluate_rules(&[assignment], &HashMap::new()).unwrap();
        assert_eq!(results[0].amount, Decimal::from(2_000_000));
    }

    // Tiered

    #[test]
    fn test_tiered_rule() {
        use crate::models::PayrollRuleTier;
        use chrono::Utc;
        let rule_id = Uuid::new_v4();
        let tiers = vec![
            PayrollRuleTier {
                id: Uuid::new_v4(),
                rule_definition_id: rule_id,
                min_amount: Decimal::ZERO,
                max_amount: Some(Decimal::from(5_000_000)),
                rate: Decimal::from(5),
                fixed_amount: Decimal::ZERO,
                tier_order: 1,
                created_at: Utc::now(),
            },
            PayrollRuleTier {
                id: Uuid::new_v4(),
                rule_definition_id: rule_id,
                min_amount: Decimal::from(5_000_000),
                max_amount: None,
                rate: Decimal::from(10),
                fixed_amount: Decimal::ZERO,
                tier_order: 2,
                created_at: Utc::now(),
            },
        ];
        let assignment = make_assignment_with_tiers(
            PayrollCalculationMethod::Tiered,
            json!({"base": "gross"}),
            tiers,
        );
        let base = base_map(10_000_000.0);
        let results = PayrollRuleEvaluator::evaluate_rules(&[assignment], &base).unwrap();
        assert!(!results.is_empty());
    }

    // Empty assignments

    #[test]
    fn test_no_assignments_returns_empty() {
        let results = PayrollRuleEvaluator::evaluate_rules(&[], &HashMap::new()).unwrap();
        assert!(results.is_empty());
    }
}
