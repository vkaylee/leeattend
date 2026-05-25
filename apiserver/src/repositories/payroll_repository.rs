use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::{
    EmployeeSalaryComponent, IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy,
    PayrollSettings, Payslip, PayslipSegment, SalaryComponentDefinition,
};

// ---------------------------------------------------------------------------
// Trait: PayrollRepository — DIP abstraction over payroll data access
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
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

    async fn get_policy_tiers(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<Vec<IncentiveTier>, AppError>;

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
        workspace_id: Uuid,
        payslip_id: Uuid,
        result: &IncentiveEvaluationResult,
    ) -> Result<(), AppError>;

    async fn get_payslip_incentives(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<IncentiveEvaluationResult>, AppError>;

    async fn create_payslip_rule_evaluation(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
        rule_code: String,
        amount: Decimal,
    ) -> Result<(), AppError>;

    async fn get_payslip_rule_evaluations(
        &self,
        workspace_id: Uuid,
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
        workspace_id: Uuid,
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

    async fn get_salary_components_history_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<
        std::collections::HashMap<Uuid, Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>>,
        AppError,
    >;

    async fn create_payslip_segment(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
        args: &crate::models::CreatePayslipSegmentArgs,
    ) -> Result<(), AppError>;

    async fn get_payslip_segments(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipSegment>, AppError>;

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
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<crate::models::PayrollRuleTier>, AppError>;

    async fn get_department_payroll_config_at(
        &self,
        workspace_id: Uuid,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError>;
}

#[derive(Debug)]
pub struct ResolvedRuleAssignment {
    pub rule_definition_id: Uuid,
    pub override_config: Option<serde_json::Value>,
    pub code: String,
    pub calculation_method: crate::models::enums::PayrollCalculationMethod,
    pub default_config: serde_json::Value,
    pub category: crate::models::enums::PayrollRuleCategory,
}
