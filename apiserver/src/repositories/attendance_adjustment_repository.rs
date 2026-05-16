use crate::errors::AppError;
use crate::models::{AdjustmentStatus, AttendanceAdjustmentRequest};
use async_trait::async_trait;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: AttendanceAdjustmentRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AttendanceAdjustmentRepository: Send + Sync {
    async fn create_request(&self, request: &AttendanceAdjustmentRequest) -> Result<(), AppError>;
    async fn get_request_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<AttendanceAdjustmentRequest>, AppError>;
    async fn update_adjustment_status(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: AdjustmentStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError>;
    async fn get_workspace_adjustment_requests(
        &self,
        workspace_id: Uuid,
        status: Option<AdjustmentStatus>,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, AppError>;
    async fn get_employee_adjustment_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, AppError>;
}
