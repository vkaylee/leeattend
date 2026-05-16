use crate::errors::AppError;
use async_trait::async_trait;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: AttendanceLogErrorRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AttendanceLogErrorRepository: Send + Sync {
    async fn create_error_log(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
    ) -> Result<(), AppError>;
    async fn create_error_log_with_fingerprint(
        &self,
        workspace_id: Uuid,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
        fingerprint: &str,
    ) -> Result<bool, AppError>;
}
