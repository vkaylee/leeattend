use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::employment::{
    CreateEmploymentRequest, Employment, EmploymentResolution, UpdateEmploymentRequest,
};

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
        termination_reason: String,
        notes: Option<String>,
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

    /// Resolve the employment effective on `date` for the given employee.
    ///
    /// Unlike [`EmploymentRepository::get_active_employment_on_date`], this does
    /// NOT filter by `status = "active"` — a terminated employment whose date
    /// range covers `date` is still returned. This is the correct resolution for
    /// historical log ingestion and reprocessing.
    ///
    /// Returns [`EmploymentResolution::Ambiguous`] when several employments
    /// cover `date`. Callers MUST NOT pick one arbitrarily: overlapping ranges
    /// are not blocked by a DB constraint, so guessing silently attributes
    /// attendance to the wrong employer.
    async fn resolve_employment_on_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<EmploymentResolution, AppError>;

    /// List all employments whose date range overlaps `[start_date, end_date]`
    /// for the given employee, ordered by `start_date DESC`.
    async fn list_employments_effective_in_range(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<Employment>, AppError>;

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
