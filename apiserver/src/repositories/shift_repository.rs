use async_trait::async_trait;
use chrono::NaiveDate;

use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Shift;

#[async_trait]
pub trait ShiftRepository: super::PoolBacked + Send + Sync {
    async fn create_shift(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreateShiftRequest,
        is_active: bool,
        parent_shift_id: Option<Uuid>,
    ) -> Result<Shift, AppError>;

    async fn get_shift_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Shift>, AppError>;

    async fn get_workspace_shifts(&self, workspace_id: Uuid) -> Result<Vec<Shift>, AppError>;

    async fn update_shift_direct(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: &crate::models::UpdateShiftRequest,
    ) -> Result<Shift, AppError>;

    async fn archive_shift(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError>;

    async fn assign_employee_shift(
        &self,
        employee_id: Uuid,
        shift_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), AppError>;

    async fn get_employee_active_shift(
        &self,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Shift>, AppError>;

    async fn update_assignments_to_new_version(
        &self,
        old_shift_id: Uuid,
        new_shift_id: Uuid,
        effective_date: NaiveDate,
    ) -> Result<(), AppError>;
}
