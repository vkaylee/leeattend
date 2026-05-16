use crate::domain::audit::AuditLog;
use crate::errors::AppError;
use async_trait::async_trait;

use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn create_log(&self, config: &AuditLog) -> Result<AuditLog, AppError>;

    // Retrieves audit logs for a workspace, ordered by created_at DESC
    async fn find_by_workspace(
        &self,
        workspace_id: Uuid,
        limit: i64,
        offset: i64,
        action: Option<String>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<AuditLog>, i64), AppError>;
}
