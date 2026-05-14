use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Department, DepartmentPayrollConfig, EmployeeDepartmentHistory};

// ---------------------------------------------------------------------------
// Trait: DepartmentRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait DepartmentRepository: Send + Sync {
    async fn create_department(
        &self,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        manager_id: Option<Uuid>,
        default_working_pattern_id: Option<Uuid>,
    ) -> Result<Department, AppError>;
    async fn get_department_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Department>, AppError>;
    async fn get_department_by_name(
        &self,
        workspace_id: Uuid,
        name: &str,
    ) -> Result<Option<Uuid>, AppError>;
    async fn update_department(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: crate::models::UpdateDepartmentRequest,
    ) -> Result<Department, AppError>;
    async fn delete_department(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError>;
    async fn get_departments(&self, workspace_id: Uuid) -> Result<Vec<Department>, AppError>;
    async fn transfer_employee(
        &self,
        employee_id: Uuid,
        department_id: Uuid,
        effective_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<EmployeeDepartmentHistory, AppError>;
    async fn get_employee_department_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDepartmentHistory>, AppError>;
    async fn get_current_department(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, AppError>;
    async fn upsert_department_payroll_config(
        &self,
        department_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<DepartmentPayrollConfig, AppError>;
    async fn get_department_payroll_config_at(
        &self,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DepartmentPayrollConfig>, AppError>;
}
