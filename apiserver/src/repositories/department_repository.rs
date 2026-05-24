use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
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
        default_working_pattern_id: Option<Uuid>,
    ) -> Result<Department, AppError>;
    async fn get_department_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Department>, AppError>;
    async fn get_department_by_name(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        name: &str,
    ) -> Result<Option<Uuid>, AppError>;
    async fn update_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: crate::models::UpdateDepartmentRequest,
    ) -> Result<Department, AppError>;
    async fn delete_department(&self, auth_token: &crate::diesel_pool::TenantAuthToken, id: Uuid) -> Result<(), AppError>;
    async fn get_departments(&self, auth_token: &crate::diesel_pool::TenantAuthToken) -> Result<Vec<Department>, AppError>;
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
    async fn get_current_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, AppError>;
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
}
