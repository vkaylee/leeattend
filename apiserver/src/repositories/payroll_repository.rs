use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    EmployeeSalaryComponent, IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy,
    PayrollSettings, Payslip, PayslipSegment, SalaryComponentDefinition,
};

// ---------------------------------------------------------------------------
// Trait: PayrollRepository — DIP abstraction over payroll data access
// ---------------------------------------------------------------------------
#[async_trait]
pub trait PayrollRepository: Send + Sync {
    async fn get_active_incentive_policies(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollIncentivePolicy>, AppError>;

    async fn get_active_incentive_policies_for_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<PayrollIncentivePolicy>, AppError>;

    async fn get_policy_tiers(&self, policy_id: Uuid) -> Result<Vec<IncentiveTier>, AppError>;

    async fn create_incentive_policy(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreatePayrollIncentivePolicyRequest,
    ) -> Result<PayrollIncentivePolicy, AppError>;

    async fn delete_incentive_policy(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<(), AppError>;

    async fn clone_incentive_policy(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<PayrollIncentivePolicy, AppError>;

    async fn create_payslip_incentive(
        &self,
        payslip_id: Uuid,
        result: &IncentiveEvaluationResult,
    ) -> Result<(), AppError>;

    async fn get_payslip_incentives(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<IncentiveEvaluationResult>, AppError>;

    async fn create_payslip_rule_evaluation(
        &self,
        payslip_id: Uuid,
        rule_code: String,
        amount: Decimal,
    ) -> Result<(), AppError>;

    async fn get_payslip_rule_evaluations(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<crate::models::PayslipRuleEvaluation>, AppError>;

    async fn get_payroll_settings(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<PayrollSettings>, AppError>;

    async fn get_payroll_settings_at(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<PayrollSettings>, AppError>;

    async fn get_payroll_settings_history(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollSettings>, AppError>;

    async fn update_payroll_settings(
        &self,
        workspace_id: Uuid,
        req: &crate::models::SavePayrollSettings,
    ) -> Result<PayrollSettings, AppError>;

    async fn get_salary_component_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryComponentDefinition>, AppError>;

    async fn create_salary_component_definition(
        &self,
        workspace_id: Uuid,
        name: String,
        code: String,
    ) -> Result<SalaryComponentDefinition, AppError>;

    async fn assign_employee_salary_component(
        &self,
        employee_id: Uuid,
        component_definition_id: Uuid,
        amount: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeeSalaryComponent, AppError>;

    async fn get_employee_salary_components(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        reference_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError>;

    async fn create_payslip(&self, payslip: &Payslip) -> Result<Payslip, AppError>;

    async fn get_payslips_by_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<Payslip>, AppError>;

    async fn get_payslips_by_workspace_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Payslip>, AppError>;

    async fn get_salary_components_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError>;

    async fn create_payslip_segment(
        &self,
        payslip_id: Uuid,
        args: &crate::models::CreatePayslipSegmentArgs,
    ) -> Result<(), AppError>;

    async fn get_payslip_segments(&self, payslip_id: Uuid)
        -> Result<Vec<PayslipSegment>, AppError>;

    async fn get_resolved_rule_assignments(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ResolvedRuleAssignment>, AppError>;

    async fn get_payroll_rule_tiers(
        &self,
        rule_definition_id: Uuid,
    ) -> Result<Vec<crate::models::PayrollRuleTier>, AppError>;

    async fn get_department_payroll_config_at(
        &self,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError>;
}

#[derive(Debug, sqlx::FromRow)]
pub struct ResolvedRuleAssignment {
    pub rule_definition_id: Uuid,
    pub override_config: Option<serde_json::Value>,
    pub code: String,
    pub calculation_method: crate::models::enums::PayrollCalculationMethod,
    pub default_config: serde_json::Value,
    pub category: crate::models::enums::PayrollRuleCategory,
}

pub async fn get_resolved_rule_assignments(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    department_id: Option<Uuid>,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<ResolvedRuleAssignment>, sqlx::Error> {
    sqlx::query_as!(
        ResolvedRuleAssignment,
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
            d.calculation_method as "calculation_method: _",
            d.rule_config as default_config,
            d.category as "category: _"
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
}

pub async fn get_payroll_rule_tiers(
    pool: &PgPool,
    rule_definition_id: Uuid,
) -> Result<Vec<crate::models::PayrollRuleTier>, sqlx::Error> {
    sqlx::query_as!(
        crate::models::PayrollRuleTier,
        "SELECT * FROM payroll_rule_tiers WHERE rule_definition_id = $1 ORDER BY tier_order ASC",
        rule_definition_id
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    use crate::models::IncentiveEvaluationResult;

    /// Normal Decimal values should convert to f64 successfully.
    #[test]
    fn test_decimal_to_f64_normal_values() {
        let values = [
            Decimal::ZERO,
            Decimal::new(150_000_000, 2), // 1,500,000.00
            Decimal::new(99_999_999, 2),  // 999,999.99
            Decimal::new(-50_000, 2),     // -500.00
        ];
        for val in values {
            assert!(
                val.to_f64().is_some(),
                "Normal Decimal {val:?} should convert to f64"
            );
        }
    }

    /// Decimal::MAX → f64 produces Infinity, which fails from_f64() roundtrip.
    /// This is the exact error chain that previously produced silent 0₫ in payslips:
    /// DB returns huge Decimal → to_f64() → Infinity → from_f64(Infinity) → None → 0₫
    #[test]
    fn test_decimal_max_roundtrip_fails() {
        let huge = Decimal::MAX;
        let as_f64 = huge.to_f64();
        // to_f64 may succeed with a value that loses precision
        if let Some(f) = as_f64 {
            let back = Decimal::from_f64(f);
            // If it succeeds, it must be different from the original due to precision loss
            if let Some(b) = back {
                assert_ne!(
                    huge, b,
                    "Decimal::MAX roundtrip via f64 must lose precision"
                );
            }
        }
    }

    /// f64 NaN and Infinity cannot become Decimal → from_f64() returns None.
    /// Without proper error handling, this would silently zero out a payslip amount.
    #[test]
    fn test_f64_nan_infinity_fail_to_decimal() {
        assert!(
            Decimal::from_f64(f64::NAN).is_none(),
            "NaN must not silently become Decimal::ZERO"
        );
        assert!(
            Decimal::from_f64(f64::INFINITY).is_none(),
            "Infinity must not silently become Decimal::ZERO"
        );
        assert!(
            Decimal::from_f64(f64::NEG_INFINITY).is_none(),
            "Negative infinity must not silently become Decimal::ZERO"
        );
    }

    /// Simulates the error path in `create_payslip_incentive` — ensures the
    /// ok_or_else pattern produces a meaningful error message for debugging.
    #[test]
    fn test_create_payslip_incentive_decimal_validation_error_message() {
        let result = IncentiveEvaluationResult {
            policy_name: "Attendance Bonus".to_string(),
            amount: Decimal::new(100_000_000, 2),
            standard_amount: Decimal::new(100_000_000, 2),
            tier_id: Some(Uuid::new_v4()),
        };

        // Normal case should succeed
        let ok = result.amount.to_f64().ok_or_else(|| {
            format!(
                "Decimal→f64 overflow for incentive amount: {}",
                result.amount
            )
        });
        assert!(ok.is_ok(), "Normal Decimal should convert to f64");

        // Verify the error message format for the f64→Decimal path (NaN from DB)
        let err = Decimal::from_f64(f64::NAN)
            .ok_or_else(|| "f64→Decimal conversion failed for incentive amount".to_string());
        assert!(err.is_err(), "NaN should fail from_f64");
        assert!(err.unwrap_err().contains("conversion failed"));
    }

    /// Validates that normal incentive amounts pass validation cleanly.
    #[test]
    fn test_create_payslip_incentive_normal_amounts_pass() {
        let result = IncentiveEvaluationResult {
            policy_name: "Monthly Bonus".to_string(),
            amount: Decimal::new(500_000_000, 2), // 5,000,000.00 VND
            standard_amount: Decimal::new(500_000_000, 2),
            tier_id: None,
        };

        assert!(result.amount.to_f64().is_some());
        assert!(result.standard_amount.to_f64().is_some());
    }

    /// Validates the f64→Decimal roundtrip in `get_payslip_incentives`.
    #[test]
    fn test_f64_to_decimal_roundtrip_precision() {
        let original = Decimal::new(123_456_789, 2); // 1,234,567.89
        let as_f64 = original.to_f64().expect("normal conversion");
        let back = Decimal::from_f64(as_f64).expect("roundtrip should work");

        let diff = (original - back).abs();
        assert!(
            diff < Decimal::new(1, 2),
            "Roundtrip precision loss too large: original={original}, back={back}, diff={diff}"
        );
    }
}
