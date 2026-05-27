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
// Trait: EmployeeCoreRepository — Core Employee CRUD and search
// ---------------------------------------------------------------------------
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait EmployeeCoreRepository: Send + Sync {
    async fn find_by_number(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employees_by_ids(
        &self,
        ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError>;

    async fn get_all_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError>;

    async fn get_active_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError>;

    async fn get_active_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_paginated_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: crate::models::EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError>;

    async fn get_employee_departments_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError>;

    async fn update_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError>;

    async fn create_employee(&self, employee: &Employee) -> Result<(), AppError>;

    async fn create_employee_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee: &Employee,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn create_employee_skip_duplicates(&self, employee: &Employee) -> Result<bool, AppError>;

    async fn rehire_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        hire_date: NaiveDate,
    ) -> Result<Option<Employee>, AppError>;

    async fn delete_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError>;

    async fn get_existing_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError>;

    async fn get_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError>;

    async fn bulk_create_employees_skip_duplicates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employees: &[Employee],
    ) -> Result<(), AppError>;

    async fn get_managed_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_employee_id_by_user_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError>;

    async fn get_employee_timezone_fallback(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<chrono_tz::Tz, AppError>;

    async fn get_headquarters_location_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError>;
}

// ---------------------------------------------------------------------------
// Trait: EmployeeWorkingConditionRepository — Working Conditions
// ---------------------------------------------------------------------------
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait EmployeeWorkingConditionRepository: Send + Sync {
    async fn get_working_conditions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError>;

    async fn create_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        condition: &WorkingCondition,
    ) -> Result<(), AppError>;

    async fn delete_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, AppError>;

    async fn get_working_conditions_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, AppError>;

    async fn get_working_conditions_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError>;
}

// ---------------------------------------------------------------------------
// Trait: EmployeePayrollConfigRepository — Payroll configs
// ---------------------------------------------------------------------------
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait EmployeePayrollConfigRepository: Send + Sync {
    async fn upsert_employee_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, AppError>;

    async fn get_employee_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, AppError>;

    async fn get_employee_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeePayrollConfig>, AppError>;
}

// ---------------------------------------------------------------------------
// Trait: EmployeeRecruitmentRepository — Recruitment and Turnover
// ---------------------------------------------------------------------------
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait EmployeeRecruitmentRepository: Send + Sync {
    async fn set_employee_recruitment_source(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError>;

    async fn get_recruitment_sources(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSource>, AppError>;

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<RecruitmentSource>, AppError>;

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError>;

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError>;

    async fn get_recruitment_source_breakdown(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError>;

    async fn get_turnover_analytics(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError>;
}

// ---------------------------------------------------------------------------
// Trait: EmployeeSelfServiceRepository — ESS (Profile, Contacts, Docs)
// ---------------------------------------------------------------------------
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait EmployeeSelfServiceRepository: Send + Sync {
    async fn get_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<crate::models::EmployeeProfile>, AppError>;

    async fn upsert_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &crate::models::UpdateEmployeeProfileRequest,
    ) -> Result<crate::models::EmployeeProfile, AppError>;

    async fn get_emergency_contacts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeEmergencyContact>, AppError>;

    async fn upsert_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &crate::models::CreateEmergencyContactRequest,
    ) -> Result<crate::models::EmployeeEmergencyContact, AppError>;

    async fn delete_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        contact_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn get_employee_documents(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeDocument>, AppError>;

    async fn create_employee_document(
        &self,
        doc: &crate::models::EmployeeDocument,
    ) -> Result<(), AppError>;
}

// ---------------------------------------------------------------------------
// Trait: EmployeeRepository — Facade combining all employee traits
// ---------------------------------------------------------------------------
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait EmployeeRepository: Send + Sync {
    async fn find_by_number(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employees_by_ids(
        &self,
        ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError>;

    async fn get_all_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError>;

    async fn get_active_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError>;

    async fn get_active_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_paginated_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: crate::models::EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError>;

    async fn get_employee_departments_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError>;

    async fn update_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError>;

    async fn create_employee(&self, employee: &Employee) -> Result<(), AppError>;

    async fn create_employee_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee: &Employee,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn create_employee_skip_duplicates(&self, employee: &Employee) -> Result<bool, AppError>;

    async fn rehire_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        hire_date: NaiveDate,
    ) -> Result<Option<Employee>, AppError>;

    async fn delete_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError>;

    async fn get_existing_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError>;

    async fn get_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError>;

    async fn bulk_create_employees_skip_duplicates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employees: &[Employee],
    ) -> Result<(), AppError>;

    async fn get_managed_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_employee_id_by_user_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError>;

    async fn get_employee_timezone_fallback(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<chrono_tz::Tz, AppError>;

    async fn get_headquarters_location_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError>;

    async fn get_working_conditions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError>;

    async fn create_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        condition: &WorkingCondition,
    ) -> Result<(), AppError>;

    async fn delete_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, AppError>;

    async fn get_working_conditions_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, AppError>;

    async fn get_working_conditions_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError>;

    async fn upsert_employee_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, AppError>;

    async fn get_employee_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, AppError>;

    async fn get_employee_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeePayrollConfig>, AppError>;

    async fn set_employee_recruitment_source(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError>;

    async fn get_recruitment_sources(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSource>, AppError>;

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<RecruitmentSource>, AppError>;

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError>;

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError>;

    async fn get_recruitment_source_breakdown(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError>;

    async fn get_turnover_analytics(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError>;

    async fn get_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<crate::models::EmployeeProfile>, AppError>;

    async fn upsert_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &crate::models::UpdateEmployeeProfileRequest,
    ) -> Result<crate::models::EmployeeProfile, AppError>;

    async fn get_emergency_contacts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeEmergencyContact>, AppError>;

    async fn upsert_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &crate::models::CreateEmergencyContactRequest,
    ) -> Result<crate::models::EmployeeEmergencyContact, AppError>;

    async fn delete_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        contact_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn get_employee_documents(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeDocument>, AppError>;

    async fn create_employee_document(
        &self,
        doc: &crate::models::EmployeeDocument,
    ) -> Result<(), AppError>;
}
