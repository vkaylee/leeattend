use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::OutboxJob;
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OutboxJobRepository: Send + Sync {
    async fn insert_job(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        job_type: &str,
        payload: serde_json::Value,
    ) -> Result<OutboxJob, AppError>;

    async fn update_job_status(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), AppError>;

    async fn get_pending_jobs(
        &self,
        auth_token: &TenantAuthToken,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError>;

    async fn get_stuck_jobs(
        &self,
        auth_token: &TenantAuthToken,
        older_than: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError>;
}
