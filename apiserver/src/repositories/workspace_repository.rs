use async_trait::async_trait;
use crate::models::{Workspace, MemberExtraRole};
use uuid::Uuid;

#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn create_workspace(&self, name: &str) -> Result<Workspace, String>;
    async fn add_member(&self, workspace_id: Uuid, user_id: Uuid, role: &str) -> Result<(), String>;
    async fn find_primary_role(&self, workspace_id: Uuid, user_id: Uuid) -> Result<Option<String>, String>;
    async fn find_extra_roles(&self, workspace_id: Uuid, user_id: Uuid) -> Result<Vec<MemberExtraRole>, String>;
}
