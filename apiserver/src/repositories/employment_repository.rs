use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::employment::{CreateEmploymentRequest, Employment, UpdateEmploymentRequest};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EmploymentRepository: Send + Sync {
    async fn create_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateEmploymentRequest,
    ) -> Result<Employment, AppError>;

    async fn get_employment_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Option<Employment>, AppError>;

    async fn list_employee_employments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<Employment>, AppError>;

    async fn update_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        req: &UpdateEmploymentRequest,
    ) -> Result<Employment, AppError>;

    async fn terminate_employment_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        termination_date: chrono::NaiveDate,
        balances: Vec<crate::models::LeaveBalance>,
    ) -> Result<Employment, AppError>;

    async fn delete_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_active_employment_on_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<Option<Employment>, AppError>;

    async fn get_active_employments_for_workspace(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employment>, AppError>;

    async fn get_active_employments_for_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        date: chrono::NaiveDate,
    ) -> Result<Vec<Employment>, AppError>;

    async fn get_employments_for_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
    ) -> Result<Vec<Employment>, AppError>;

    async fn get_employments_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
    ) -> Result<Vec<Employment>, AppError>;
}
