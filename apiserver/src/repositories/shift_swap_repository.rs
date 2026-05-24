use crate::errors::AppError;
use crate::models::shift_swap::{ShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus};
use async_trait::async_trait;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: ShiftSwapRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ShiftSwapRepository: Send + Sync {
    async fn create_swap_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &ShiftSwapRequest,
    ) -> Result<ShiftSwapRequest, AppError>;
    async fn get_swap_request_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<ShiftSwapRequest>, AppError>;
    async fn get_workspace_swaps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status_filter: Option<ShiftSwapStatus>,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, AppError>;
    async fn update_swap_status(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: ShiftSwapStatus,
        approver_id: Option<Uuid>,
    ) -> Result<(), AppError>;
    async fn get_employee_swaps(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, AppError>;
    async fn has_active_swap(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        requester_id: Uuid,
        target_id: Uuid,
        swap_date: chrono::NaiveDate,
    ) -> Result<bool, AppError>;
    async fn has_approved_swap_on_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        swap_date: chrono::NaiveDate,
    ) -> Result<bool, AppError>;
    async fn get_workspace_swap_stats(&self, auth_token: &crate::diesel_pool::TenantAuthToken) -> Result<SwapStats, AppError>;
}

/// Status counts for a workspace (for stats dashboard).
#[derive(Debug, serde::Serialize)]
pub struct SwapStats {
    pub pending_target: i64,
    pub pending_admin: i64,
    pub approved: i64,
    pub rejected: i64,
    pub cancelled: i64,
    pub approved_this_month: i64,
    pub rejected_this_month: i64,
}
