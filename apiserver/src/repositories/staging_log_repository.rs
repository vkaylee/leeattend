use crate::errors::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: StagingLogRepository — DIP abstraction
// (update_staging_status_tx is excluded — it takes &mut Transaction)
// ---------------------------------------------------------------------------
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
        status: &str,
    ) -> Result<bool, AppError>;
    async fn get_pending_staging_logs(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<StagingLog>, AppError>;

    async fn count_pending_staging_logs(&self, workspace_id: Uuid) -> Result<i64, AppError>;

    async fn claim_staged_logs(
        &self,
        workspace_id: Uuid,
        raw_user_id: &str,
        employee_id: Uuid,
    ) -> Result<u64, AppError>;
    async fn resolve_staging_log(
        &self,
        log_id: Uuid,
        workspace_id: Uuid,
        employee_id: Uuid,
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
