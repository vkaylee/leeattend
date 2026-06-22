use crate::errors::AppResult;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DebugUserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait DebugRepository: Send + Sync {
    async fn get_debug_users(&self) -> AppResult<Vec<DebugUserInfo>>;
    async fn clear_debug_db(&self, workspace_id: Uuid, group: &str) -> AppResult<()>;
}
