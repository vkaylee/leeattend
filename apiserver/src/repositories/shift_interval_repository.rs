use crate::errors::AppError;
use crate::models::{CreateShiftIntervalRequest, ShiftDefaultInterval, UpdateShiftIntervalRequest};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ShiftIntervalRepository: Send + Sync {
    async fn create_interval(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        shift_id: Uuid,
        req: &CreateShiftIntervalRequest,
    ) -> Result<ShiftDefaultInterval, AppError>;

    async fn get_intervals_for_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        shift_id: Uuid,
    ) -> Result<Vec<ShiftDefaultInterval>, AppError>;

    async fn update_interval(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        interval_id: Uuid,
        req: &UpdateShiftIntervalRequest,
    ) -> Result<ShiftDefaultInterval, AppError>;

    async fn delete_interval(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        interval_id: Uuid,
    ) -> Result<(), AppError>;

    async fn reorder_intervals(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        shift_id: Uuid,
        interval_ids: &[Uuid],
    ) -> Result<(), AppError>;

    async fn delete_all_for_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        shift_id: Uuid,
    ) -> Result<(), AppError>;
}
