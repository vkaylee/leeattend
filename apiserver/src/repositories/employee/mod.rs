//! Employee repository — abstraction over employee data access.

use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::{
    Employee, EmployeePayrollConfig, RecruitmentSource, RecruitmentSourceBreakdown,
    TurnoverAnalytics, WorkingCondition,
};

/// Minimal employee info for paginated reports
#[derive(Debug, Clone, diesel::Queryable, serde::Serialize)]
pub struct EmployeeBasicInfo {
    pub id: Uuid,
    pub full_name: String,
    pub employee_id_number: String,
}

// ---------------------------------------------------------------------------
// Trait: EmployeeRepository — DIP abstraction over employee data access
// ---------------------------------------------------------------------------
#[async_trait]
pub trait EmployeeRepository: Send + Sync {
    async fn find_by_number(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError>;

    async fn get_active_employees(&self, workspace_id: Uuid) -> Result<Vec<Employee>, AppError>;

    async fn get_active_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_paginated_employee_ids(
        &self,
        workspace_id: Uuid,
        args: crate::models::EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError>;

    async fn get_employee_departments_at(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError>;

    async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &crate::models::UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError>;

    async fn create_employee(&self, employee: &Employee) -> Result<(), AppError>;

    async fn create_employee_skip_duplicates(&self, employee: &Employee) -> Result<bool, AppError>;

    async fn rehire_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        hire_date: NaiveDate,
    ) -> Result<Option<Employee>, AppError>;

    async fn delete_employee(&self, id: Uuid, workspace_id: Uuid) -> Result<u64, AppError>;

    async fn get_working_conditions(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError>;

    async fn create_working_condition(&self, condition: &WorkingCondition) -> Result<(), AppError>;

    async fn delete_working_condition(&self, id: Uuid, employee_id: Uuid)
        -> Result<bool, AppError>;

    async fn get_existing_employee_ids_by_numbers(
        &self,
        workspace_id: Uuid,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError>;

    async fn upsert_employee_payroll_config(
        &self,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, AppError>;

    async fn get_employee_payroll_config_at(
        &self,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, AppError>;

    async fn get_employee_payroll_configs(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeePayrollConfig>, AppError>;

    async fn set_employee_recruitment_source(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError>;

    async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSource>, AppError>;

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<RecruitmentSource>, AppError>;

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError>;

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError>;

    async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError>;

    async fn get_managed_departments(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_employee_id_by_user_id(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError>;
    async fn get_working_conditions_at(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, AppError>;

    async fn get_working_conditions_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError>;

    // --- ESS (Employee Self Service) ---
    async fn get_employee_profile(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<crate::models::EmployeeProfile>, AppError>;

    async fn upsert_employee_profile(
        &self,
        employee_id: Uuid,
        req: &crate::models::UpdateEmployeeProfileRequest,
    ) -> Result<crate::models::EmployeeProfile, AppError>;

    async fn get_emergency_contacts(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeEmergencyContact>, AppError>;

    async fn upsert_emergency_contact(
        &self,
        employee_id: Uuid,
        req: &crate::models::CreateEmergencyContactRequest,
    ) -> Result<crate::models::EmployeeEmergencyContact, AppError>;

    async fn delete_emergency_contact(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        contact_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn get_employee_documents(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeDocument>, AppError>;

    async fn create_employee_document(
        &self,
        doc: &crate::models::EmployeeDocument,
    ) -> Result<(), AppError>;
}
