use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Department, DepartmentPayrollConfig, EmployeeDepartmentHistory};

// ---------------------------------------------------------------------------
// Trait: DepartmentRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DepartmentRepository: Send + Sync {
    async fn create_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        name: String,
        description: Option<String>,
        manager_id: Option<Uuid>,
    ) -> Result<Department, AppError>;
    async fn get_department_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Department>, AppError>;
    async fn get_departments_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<Vec<Department>, AppError>;
    async fn get_department_by_name(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        name: &str,
    ) -> Result<Option<Uuid>, AppError>;
    async fn get_department_ids_by_names(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        names: &[String],
    ) -> Result<HashMap<String, Uuid>, AppError>;
    async fn bulk_create_departments_skip_duplicates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        departments: &[Department],
    ) -> Result<(), AppError>;
    async fn update_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: crate::models::UpdateDepartmentRequest,
    ) -> Result<Department, AppError>;
    async fn delete_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;
    async fn get_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Department>, i64), AppError>;
    async fn transfer_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        department_id: Uuid,
        effective_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<EmployeeDepartmentHistory, AppError>;
    async fn get_employee_department_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDepartmentHistory>, AppError>;
    async fn get_employee_department_history_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<EmployeeDepartmentHistory>>, AppError>;
    async fn get_current_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, AppError>;
    async fn get_current_departments_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, EmployeeDepartmentHistory>, AppError>;
    async fn upsert_department_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<DepartmentPayrollConfig, AppError>;
    async fn get_department_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DepartmentPayrollConfig>, AppError>;
    async fn get_department_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
    ) -> Result<Vec<DepartmentPayrollConfig>, AppError>;
    async fn get_department_payroll_configs_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<DepartmentPayrollConfig>>, AppError>;
}
