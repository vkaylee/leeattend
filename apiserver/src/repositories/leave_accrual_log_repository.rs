use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::LeaveAccrualLog;

// ---------------------------------------------------------------------------
// Trait: LeaveAccrualLogRepository — DIP abstraction for accrual log persistence
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait LeaveAccrualLogRepository: Send + Sync {
    /// Find an existing accrual log for a specific employment/leave-type/date.
    async fn find_by_employment_date_type(
        &self,
        employment_id: Uuid,
        leave_type_id: Uuid,
        accrual_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<Option<ExistingAccrualLog>, AppError>;

    /// Fetch the `effective_from` date for a leave policy assignment.
    async fn get_assignment_effective_from(
        &self,
        assignment_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<NaiveDate, AppError>;

    /// Fetch the workspace timezone string.
    async fn get_workspace_timezone(&self, workspace_id: Uuid) -> Result<String, AppError>;

    async fn get_employment_leave_history_logs(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryLogItem>, AppError>;

    async fn update_accrual_amount(
        &self,
        log_id: Uuid,
        workspace_id: Uuid,
        new_amount: Decimal,
    ) -> Result<LeaveAccrualLog, AppError>;

    async fn insert_accrual_log(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        assignment_id: Option<Uuid>,
        accrual_date: NaiveDate,
        amount: Decimal,
        leave_type_id: Uuid,
    ) -> Result<Option<LeaveAccrualLog>, AppError>;

    async fn upsert_balance_accrual(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        year: i32,
        leave_type_id: Uuid,
        amount: Decimal,
    ) -> Result<(), AppError>;

    async fn adjust_balance_accrual(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        year: i32,
        leave_type_id: Uuid,
        diff: Decimal,
    ) -> Result<(), AppError>;

    /// Atomically find, update/insert log and adjust the balance
    async fn persist_accrual(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        employment_id: Uuid,
        assignment_id: Uuid,
        leave_type_id: Uuid,
        accrual_date: NaiveDate,
        amount: f64,
        force: bool,
    ) -> Result<Option<LeaveAccrualLog>, AppError>;

    /// Batch persist multiple accrual entries in a single transaction.
    /// Eliminates N+1 writes by processing all entries atomically.
    async fn persist_accruals_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        entries: &[AccrualBatchEntry],
        force: bool,
    ) -> Result<Vec<LeaveAccrualLog>, AppError>;
}

/// Minimal view of an existing accrual log (only the fields the orchestrator needs).
pub struct ExistingAccrualLog {
    pub id: Uuid,
    pub amount: rust_decimal::Decimal,
}

/// Entry for batch accrual persistence.
pub struct AccrualBatchEntry {
    pub employment_id: Uuid,
    pub assignment_id: Uuid,
    pub leave_type_id: Uuid,
    pub accrual_date: NaiveDate,
    pub amount: f64,
}

#[derive(Clone)]
pub struct LeaveHistoryLogItem {
    pub id: Uuid,
    pub accrual_date: NaiveDate,
    pub amount: rust_decimal::Decimal,
    pub leave_type_name: String,
    pub notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub policy_assignment_id: Option<Uuid>,
}
