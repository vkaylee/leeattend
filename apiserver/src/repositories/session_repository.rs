use crate::errors::AppError;
use crate::models::diesel::core::SessionEntity;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait SessionRepository: super::PoolBacked + Send + Sync {
    async fn create_session(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError>;
    async fn get_session_by_token(&self, token: &str)
        -> Result<Option<SessionEntity>, AppError>;
    async fn rotate_refresh_token(
        &self,
        session_id: Uuid,
        new_token: &str,
    ) -> Result<(), AppError>;
    async fn revoke_session(&self, token: &str) -> Result<(), AppError>;
    async fn revoke_all_sessions_for_user(&self, user_id: Uuid) -> Result<(), AppError>;
}
