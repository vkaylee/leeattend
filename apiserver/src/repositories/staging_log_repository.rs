use crate::errors::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Lifecycle state of a staged punch.
///
/// Stored as plain text in `attendance_staging_logs.status` (no DB CHECK
/// constraint), so this enum is the single source of truth for valid values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StagingStatus {
    /// Identity unknown — waits for the employee to be created/imported.
    Pending,
    /// Identity known but several employments cover the punch date; a human
    /// must choose which one owns the punch.
    Ambiguous,
    /// Promoted into `raw_attendance_logs`.
    Processed,
    /// Discarded by an operator without processing.
    Dismissed,
}

impl StagingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ambiguous => "ambiguous",
            Self::Processed => "processed",
            Self::Dismissed => "dismissed",
        }
    }

    /// Statuses still awaiting an outcome — what the HR review queue shows.
    pub fn open_statuses() -> [&'static str; 2] {
        [Self::Pending.as_str(), Self::Ambiguous.as_str()]
    }
}

// ---------------------------------------------------------------------------
// Trait: StagingLogRepository — DIP abstraction
// (update_staging_status_tx is excluded — it takes &mut Transaction)
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait StagingLogRepository: Send + Sync {
    async fn create_staging_log(
        &self,
        workspace_id: Uuid,
        device_id: Option<String>,
        raw_user_id: &str,
        punch_time: DateTime<Utc>,
        raw_data: Option<serde_json::Value>,
        fingerprint: &str,
        status: StagingStatus,
    ) -> Result<bool, AppError>;

    async fn get_pending_staged_logs(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Vec<StagingLog>, AppError>;

    async fn update_staging_status(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        status: StagingStatus,
    ) -> Result<bool, AppError>;

    /// List staging logs awaiting a human decision.
    ///
    /// Includes both [`StagingStatus::Pending`] (unknown user) and
    /// [`StagingStatus::Ambiguous`] (known user, undecidable employment) so
    /// neither queue is invisible to HR.
    async fn get_pending_staging_logs(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<StagingLog>, AppError>;

    async fn count_pending_staging_logs(&self, workspace_id: Uuid) -> Result<i64, AppError>;

    /// Auto-claim `pending` logs for a newly known employee.
    ///
    /// Only touches [`StagingStatus::Pending`]: an `ambiguous` log is parked
    /// precisely because automation cannot decide it, so claiming it here would
    /// reintroduce the guess.
    async fn claim_staged_logs(
        &self,
        workspace_id: Uuid,
        raw_user_id: &str,
        employee_id: Uuid,
    ) -> Result<u64, AppError>;

    /// Resolve one staging log into a real attendance log.
    ///
    /// `employment_id` is the operator's explicit choice; it is validated to
    /// belong to `employee_id` in this workspace and to cover the log's punch
    /// date. When `None`, the employment is resolved from the punch date and an
    /// ambiguous outcome is reported as an error rather than guessed.
    async fn resolve_staging_log(
        &self,
        log_id: Uuid,
        workspace_id: Uuid,
        employee_id: Uuid,
        employment_id: Option<Uuid>,
    ) -> Result<crate::models::RawAttendanceLog, AppError>;
}

/// Staging log for unknown users (not yet mapped to employees)
#[derive(Debug, Clone)]
pub struct StagingLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub device_id: Option<String>,
    pub raw_user_id: String,
    pub punch_time: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
    pub fingerprint: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
