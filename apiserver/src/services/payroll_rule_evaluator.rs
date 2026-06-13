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
                        .get_payroll_rule_tiers(workspace_id, assignment.rule_definition_id)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::models::enums::{PayrollCalculationMethod, PayrollRuleCategory};
    use crate::repositories::payroll_repository::{PayrollRepository, ResolvedRuleAssignment};
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use serde_json::json;
    use std::collections::HashMap;
    use uuid::Uuid;

    // ── Minimal mock — only the two methods used by evaluate_rules ───────────

    struct MockPayrollRepo {
        // Store configs so we can reconstruct assignments without Clone on ResolvedRuleAssignment
        assignment_specs: Vec<(PayrollCalculationMethod, serde_json::Value, Uuid)>,
        tiers: Vec<crate::models::PayrollRuleTier>,
    }

    #[async_trait]
    impl PayrollRepository for MockPayrollRepo {
        async fn get_salary_components_history_batch(
            &self,
            _workspace_id: uuid::Uuid,
            _employee_ids: &[uuid::Uuid],
            _start_date: chrono::NaiveDate,
            _end_date: chrono::NaiveDate,
        ) -> Result<
            std::collections::HashMap<
                uuid::Uuid,
                Vec<(
                    crate::models::EmployeeSalaryComponent,
                    crate::models::SalaryComponentDefinition,
                )>,
            >,
            crate::errors::AppError,
        > {
            Ok(std::collections::HashMap::new())
        }

        async fn get_resolved_rule_assignments(
            &self,
            _w: Uuid,
            _e: Uuid,
            _d: Option<Uuid>,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<ResolvedRuleAssignment>, AppError> {
            Ok(self
                .assignment_specs
                .iter()
                .map(|(method, config, rule_id)| ResolvedRuleAssignment {
                    rule_definition_id: *rule_id,
                    override_config: None,
                    code: "TEST_RULE".into(),
                    calculation_method: *method,
                    default_config: config.clone(),
                    category: PayrollRuleCategory::Allowance,
                })
                .collect())
        }
        async fn get_payroll_rule_tiers(
            &self,
            _w: Uuid,
            _r: Uuid,
        ) -> Result<Vec<crate::models::PayrollRuleTier>, AppError> {
            Ok(self.tiers.clone())
        }
        async fn get_active_incentive_policies(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::PayrollIncentivePolicy>, AppError> {
            Ok(vec![])
        }
        async fn get_active_incentive_policies_for_period(
            &self,
            _w: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<Vec<crate::models::PayrollIncentivePolicy>, AppError> {
            Ok(vec![])
        }
        async fn get_policy_tiers(
            &self,
            _w: Uuid,
            _p: Uuid,
        ) -> Result<Vec<crate::models::IncentiveTier>, AppError> {
            Ok(vec![])
        }
        async fn create_incentive_policy(
            &self,
            _w: Uuid,
            _r: &crate::models::CreatePayrollIncentivePolicyRequest,
        ) -> Result<crate::models::PayrollIncentivePolicy, AppError> {
            unimplemented!()
        }
        async fn delete_incentive_policy(&self, _w: Uuid, _p: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn clone_incentive_policy(
            &self,
            _w: Uuid,
            _p: Uuid,
        ) -> Result<crate::models::PayrollIncentivePolicy, AppError> {
            unimplemented!()
        }
        async fn create_payslip_incentive(
            &self,
            _w: Uuid,
            _p: Uuid,
            _r: &crate::models::IncentiveEvaluationResult,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_payslip_incentives(
            &self,
            _w: Uuid,
            _p: Uuid,
        ) -> Result<Vec<crate::models::IncentiveEvaluationResult>, AppError> {
            Ok(vec![])
        }
        async fn create_payslip_rule_evaluation(
            &self,
            _w: Uuid,
            _p: Uuid,
            _c: String,
            _a: Decimal,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_payslip_rule_evaluations(
            &self,
            _w: Uuid,
            _p: Uuid,
        ) -> Result<Vec<crate::models::PayslipRuleEvaluation>, AppError> {
            Ok(vec![])
        }
        async fn get_payroll_settings(
            &self,
            _w: Uuid,
        ) -> Result<Option<crate::models::PayrollSettings>, AppError> {
            Ok(None)
        }
        async fn get_payroll_settings_at(
            &self,
            _w: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<crate::models::PayrollSettings>, AppError> {
            Ok(None)
        }
        async fn get_payroll_settings_history(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::PayrollSettings>, AppError> {
            Ok(vec![])
        }
        async fn update_payroll_settings(
            &self,
            _w: Uuid,
            _r: &crate::models::SavePayrollSettings,
        ) -> Result<crate::models::PayrollSettings, AppError> {
            unimplemented!()
        }
        async fn get_salary_component_definitions(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::SalaryComponentDefinition>, AppError> {
            Ok(vec![])
        }
        async fn create_salary_component_definition(
            &self,
            _w: Uuid,
            _n: String,
            _c: String,
            _b: Option<Vec<Uuid>>,
        ) -> Result<crate::models::SalaryComponentDefinition, AppError> {
            unimplemented!()
        }
        async fn assign_employment_salary_component(
            &self,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _component_definition_id: Uuid,
            _amount: Decimal,
            _effective_date: NaiveDate,
        ) -> Result<crate::models::EmployeeSalaryComponent, AppError> {
            unimplemented!()
        }
        async fn get_employment_salary_components(
            &self,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _reference_date: NaiveDate,
        ) -> Result<
            Vec<(
                crate::models::EmployeeSalaryComponent,
                crate::models::SalaryComponentDefinition,
            )>,
            AppError,
        > {
            Ok(vec![])
        }
        async fn create_payslip(
            &self,
            _p: &crate::models::Payslip,
        ) -> Result<crate::models::Payslip, AppError> {
            unimplemented!()
        }
        async fn get_payslips_by_employee(
            &self,
            _w: Uuid,
            _e: Uuid,
        ) -> Result<Vec<crate::models::Payslip>, AppError> {
            Ok(vec![])
        }
        async fn get_payslips_by_workspace_period(
            &self,
            _w: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<Vec<crate::models::Payslip>, AppError> {
            Ok(vec![])
        }
        async fn get_salary_components_history(
            &self,
            _w: Uuid,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<
            Vec<(
                crate::models::EmployeeSalaryComponent,
                crate::models::SalaryComponentDefinition,
            )>,
            AppError,
        > {
            Ok(vec![])
        }
        async fn create_payslip_segment(
            &self,
            _w: Uuid,
            _p: Uuid,
            _a: &crate::models::CreatePayslipSegmentArgs,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_payslip_segments(
            &self,
            _w: Uuid,
            _p: Uuid,
        ) -> Result<Vec<crate::models::PayslipSegment>, AppError> {
            Ok(vec![])
        }
        async fn persist_calculated_payslip_data(
            &self,
            _w: Uuid,
            _p: &crate::models::Payslip,
            _s: &[crate::models::CreatePayslipSegmentArgs],
            _i: &[crate::models::IncentiveEvaluationResult],
            _r: &[(String, Decimal)],
        ) -> Result<crate::models::Payslip, AppError> {
            unimplemented!()
        }
        async fn get_department_payroll_config_at(
            &self,
            _ws: Uuid,
            _d: Uuid,
            _dt: NaiveDate,
        ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError> {
            Ok(None)
        }
        async fn get_salary_components_history_for_employments(
            &self,
            _workspace_id: Uuid,
            _employment_ids: &[Uuid],
        ) -> Result<
            Vec<(
                crate::models::EmployeeSalaryComponent,
                crate::models::SalaryComponentDefinition,
            )>,
            AppError,
        > {
            Ok(vec![])
        }
    }

    fn make_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
    }

    fn spec(
        method: PayrollCalculationMethod,
        config: serde_json::Value,
    ) -> (PayrollCalculationMethod, serde_json::Value, Uuid) {
        (method, config, Uuid::new_v4())
    }

    fn base_map(gross: f64) -> HashMap<String, Decimal> {
        let mut m = HashMap::new();
        m.insert("gross".into(), Decimal::from_f64(gross).unwrap());
        m
    }

    // ── Percentage ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_percentage_rule_basic() {
        let repo = MockPayrollRepo {
            assignment_specs: vec![spec(
                PayrollCalculationMethod::Percentage,
                json!({"rate": 10.0, "base": "gross"}),
            )],
            tiers: vec![],
        };
        let base = base_map(10_000_000.0);
        let results = PayrollRuleEvaluator::evaluate_rules(
            &repo,
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            make_date(),
            make_date(),
            &base,
        )
        .await
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_code, "TEST_RULE");
        // 10% of 10_000_000 = 1_000_000
        assert_eq!(results[0].amount, Decimal::from(1_000_000));
    }

    #[tokio::test]
    async fn test_percentage_rule_with_cap() {
        let repo = MockPayrollRepo {
            assignment_specs: vec![spec(
                PayrollCalculationMethod::Percentage,
                json!({"rate": 50.0, "base": "gross", "cap_amount": 500_000.0}),
            )],
            tiers: vec![],
        };
        let base = base_map(5_000_000.0);
        let results = PayrollRuleEvaluator::evaluate_rules(
            &repo,
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            make_date(),
            make_date(),
            &base,
        )
        .await
        .unwrap();
        // 50% of 5M = 2.5M but capped at 500K
        assert_eq!(results[0].amount, Decimal::from(500_000));
    }

    #[tokio::test]
    async fn test_percentage_rule_zero_result_not_included() {
        // rate = 0 → amount stays 0 → not pushed to results
        let repo = MockPayrollRepo {
            assignment_specs: vec![spec(
                PayrollCalculationMethod::Percentage,
                json!({"rate": 0.0, "base": "gross"}),
            )],
            tiers: vec![],
        };
        let base = base_map(10_000_000.0);
        let results = PayrollRuleEvaluator::evaluate_rules(
            &repo,
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            make_date(),
            make_date(),
            &base,
        )
        .await
        .unwrap();
        assert!(
            results.is_empty(),
            "Zero-amount results should be filtered out"
        );
    }

    // ── Fixed ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_fixed_rule() {
        let repo = MockPayrollRepo {
            assignment_specs: vec![spec(
                PayrollCalculationMethod::Fixed,
                json!({"amount": 2_000_000.0}),
            )],
            tiers: vec![],
        };
        let results = PayrollRuleEvaluator::evaluate_rules(
            &repo,
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            make_date(),
            make_date(),
            &HashMap::new(),
        )
        .await
        .unwrap();
        assert_eq!(results[0].amount, Decimal::from(2_000_000));
    }

    // ── Tiered ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_tiered_rule() {
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
        let repo = MockPayrollRepo {
            assignment_specs: vec![(
                PayrollCalculationMethod::Tiered,
                json!({"base": "gross"}),
                rule_id,
            )],
            tiers,
        };
        let base = base_map(10_000_000.0);
        let results = PayrollRuleEvaluator::evaluate_rules(
            &repo,
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            make_date(),
            make_date(),
            &base,
        )
        .await
        .unwrap();
        assert!(!results.is_empty());
    }

    // ── Empty assignments ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_no_assignments_returns_empty() {
        let repo = MockPayrollRepo {
            assignment_specs: vec![],
            tiers: vec![],
        };
        let results = PayrollRuleEvaluator::evaluate_rules(
            &repo,
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            make_date(),
            make_date(),
            &HashMap::new(),
        )
        .await
        .unwrap();
        assert!(results.is_empty());
    }
}
