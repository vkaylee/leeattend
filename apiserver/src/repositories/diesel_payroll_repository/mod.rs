use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{
    EmployeeSalaryComponent, IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy,
    PayrollSettings, Payslip, PayslipRuleEvaluation, PayslipSegment, SalaryComponentDefinition,
};
use crate::repositories::payroll_repository::*;
use crate::schema::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

pub mod incentives;
pub mod payslip;
pub mod rules;
pub mod salary_components;
pub mod settings;

#[cfg(test)]
mod tests;

pub struct DieselPayrollRepository {
    diesel_pools: DieselPools,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = salary_component_base_members)]
struct SalaryBaseGroupMemberRow {
    pub component_definition_id: Uuid,
    pub base_group_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DieselPayrollRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl PayrollRepository for DieselPayrollRepository {
    async fn get_active_incentive_policies(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollIncentivePolicy>, AppError> {
        self.get_active_incentive_policies(workspace_id).await
    }

    async fn get_active_incentive_policies_for_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<PayrollIncentivePolicy>, AppError> {
        self.get_active_incentive_policies_for_period(workspace_id, period_start, period_end)
            .await
    }

    async fn get_policy_tiers(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<Vec<IncentiveTier>, AppError> {
        self.get_policy_tiers(workspace_id, policy_id).await
    }

    async fn create_incentive_policy(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreatePayrollIncentivePolicyRequest,
    ) -> Result<PayrollIncentivePolicy, AppError> {
        self.create_incentive_policy(workspace_id, req).await
    }

    async fn delete_incentive_policy(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<(), AppError> {
        self.delete_incentive_policy(workspace_id, policy_id).await
    }

    async fn clone_incentive_policy(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<PayrollIncentivePolicy, AppError> {
        self.clone_incentive_policy(workspace_id, policy_id).await
    }

    async fn create_payslip_incentive(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
        result: &IncentiveEvaluationResult,
    ) -> Result<(), AppError> {
        self.create_payslip_incentive(workspace_id, payslip_id, result)
            .await
    }

    async fn get_payslip_incentives(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<IncentiveEvaluationResult>, AppError> {
        self.get_payslip_incentives(workspace_id, payslip_id).await
    }

    async fn create_payslip_rule_evaluation(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
        rule_code: String,
        amount: Decimal,
    ) -> Result<(), AppError> {
        self.create_payslip_rule_evaluation(workspace_id, payslip_id, rule_code, amount)
            .await
    }

    async fn get_payslip_rule_evaluations(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipRuleEvaluation>, AppError> {
        self.get_payslip_rule_evaluations(workspace_id, payslip_id)
            .await
    }

    async fn get_payroll_settings(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<PayrollSettings>, AppError> {
        self.get_payroll_settings(workspace_id).await
    }

    async fn get_payroll_settings_at(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<PayrollSettings>, AppError> {
        self.get_payroll_settings_at(workspace_id, date).await
    }

    async fn get_payroll_settings_history(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollSettings>, AppError> {
        self.get_payroll_settings_history(workspace_id).await
    }

    async fn update_payroll_settings(
        &self,
        workspace_id: Uuid,
        req: &crate::models::SavePayrollSettings,
    ) -> Result<PayrollSettings, AppError> {
        self.update_payroll_settings(workspace_id, req).await
    }

    async fn get_salary_component_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryComponentDefinition>, AppError> {
        self.get_salary_component_definitions(workspace_id).await
    }

    async fn create_salary_component_definition(
        &self,
        workspace_id: Uuid,
        name: String,
        code: String,
        base_group_ids: Option<Vec<Uuid>>,
    ) -> Result<SalaryComponentDefinition, AppError> {
        self.create_salary_component_definition(workspace_id, name, code, base_group_ids)
            .await
    }

    async fn assign_employment_salary_component(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        component_definition_id: Uuid,
        amount: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeeSalaryComponent, AppError> {
        self.assign_employment_salary_component(
            workspace_id,
            employment_id,
            component_definition_id,
            amount,
            effective_date,
        )
        .await
    }

    async fn get_employment_salary_components(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        reference_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        self.get_employment_salary_components(workspace_id, employment_id, reference_date)
            .await
    }

    async fn create_payslip(&self, payslip: &Payslip) -> Result<Payslip, AppError> {
        self.create_payslip(payslip).await
    }

    async fn get_payslips_by_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<Payslip>, AppError> {
        self.get_payslips_by_employee(workspace_id, employee_id)
            .await
    }

    async fn get_payslips_by_workspace_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Payslip>, AppError> {
        self.get_payslips_by_workspace_period(workspace_id, period_start, period_end)
            .await
    }

    async fn get_salary_components_history(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        self.get_salary_components_history(workspace_id, employment_id, start_date, end_date)
            .await
    }

    async fn get_salary_components_history_batch(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<
        std::collections::HashMap<Uuid, Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>>,
        AppError,
    > {
        self.get_salary_components_history_batch(workspace_id, employment_ids, start_date, end_date)
            .await
    }

    async fn create_payslip_segment(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
        args: &crate::models::CreatePayslipSegmentArgs,
    ) -> Result<(), AppError> {
        self.create_payslip_segment(workspace_id, payslip_id, args)
            .await
    }

    async fn get_payslip_segments(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipSegment>, AppError> {
        self.get_payslip_segments(workspace_id, payslip_id).await
    }

    async fn persist_calculated_payslip_data(
        &self,
        workspace_id: Uuid,
        payslip: &Payslip,
        segments: &[crate::models::CreatePayslipSegmentArgs],
        incentives: &[IncentiveEvaluationResult],
        rule_evaluations: &[(String, Decimal)],
    ) -> Result<Payslip, AppError> {
        self.persist_calculated_payslip_data(
            workspace_id,
            payslip,
            segments,
            incentives,
            rule_evaluations,
        )
        .await
    }

    async fn get_resolved_rule_assignments(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ResolvedRuleAssignment>, AppError> {
        self.get_resolved_rule_assignments(
            workspace_id,
            employee_id,
            department_id,
            start_date,
            end_date,
        )
        .await
    }

    async fn get_payroll_rule_tiers(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<crate::models::PayrollRuleTier>, AppError> {
        self.get_payroll_rule_tiers(workspace_id, rule_definition_id)
            .await
    }

    async fn get_department_payroll_config_at(
        &self,
        workspace_id: Uuid,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError> {
        self.get_department_payroll_config_at(workspace_id, department_id, date)
            .await
    }

    async fn get_salary_components_history_for_employments(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        self.get_salary_components_history_for_employments(workspace_id, employment_ids)
            .await
    }

    async fn get_resolved_rule_assignments_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        employee_departments: &std::collections::HashMap<Uuid, Option<Uuid>>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<ResolvedRuleAssignment>>, AppError> {
        self.get_resolved_rule_assignments_batch(
            workspace_id,
            employee_ids,
            employee_departments,
            start_date,
            end_date,
        )
        .await
    }

    async fn get_payroll_rule_tiers_batch(
        &self,
        workspace_id: Uuid,
        rule_definition_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<crate::models::PayrollRuleTier>>, AppError>
    {
        self.get_payroll_rule_tiers_batch(workspace_id, rule_definition_ids)
            .await
    }
}
