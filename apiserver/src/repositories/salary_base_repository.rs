use crate::errors::AppError;
use crate::models::{SalaryBaseGroup, SalaryBaseGroupMember};
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SalaryBaseRepository: Send + Sync {
    async fn get_groups(&self, workspace_id: Uuid) -> Result<Vec<SalaryBaseGroup>, AppError>;
    async fn create_group(
        &self,
        workspace_id: Uuid,
        name: &str,
        slug: &str,
        description: Option<String>,
    ) -> Result<SalaryBaseGroup, AppError>;
    async fn get_group_members(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryBaseGroupMember>, AppError>;
    async fn assign_component_to_group(
        &self,
        component_id: Uuid,
        base_group_id: Uuid,
    ) -> Result<(), AppError>;
    async fn unassign_component_from_group(
        &self,
        component_id: Uuid,
        base_group_id: Uuid,
    ) -> Result<(), AppError>;
}
