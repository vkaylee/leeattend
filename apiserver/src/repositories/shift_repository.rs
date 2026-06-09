use async_trait::async_trait;
use chrono::NaiveDate;

use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Shift;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ShiftRepository: Send + Sync {
    async fn create_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::CreateShiftRequest,
        is_active: bool,
        parent_shift_id: Option<Uuid>,
    ) -> Result<Shift, AppError>;

    async fn get_shift_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Shift>, AppError>;

    async fn get_workspace_shifts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        location_id: Option<Uuid>,
        employer_id: Option<Uuid>,
    ) -> Result<Vec<Shift>, AppError>;

    async fn update_shift_direct(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: &crate::models::UpdateShiftRequest,
    ) -> Result<Shift, AppError>;

    async fn archive_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;

    async fn assign_employment_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        shift_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), AppError>;

    async fn get_employment_active_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Shift>, AppError>;

    async fn update_assignments_to_new_version(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        old_shift_id: Uuid,
        new_shift_id: Uuid,
        effective_date: NaiveDate,
    ) -> Result<(), AppError>;
}
