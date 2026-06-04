use crate::models::{CreateWorkLocationRequest, UpdateWorkLocationRequest, WorkLocation};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait LocationRepository: Send + Sync {
    async fn find_all_by_workspace(
        &self,
        workspace_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkLocation>, i64), String>;
    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<WorkLocation>, String>;
    async fn create(
        &self,
        workspace_id: Uuid,
        data: CreateWorkLocationRequest,
    ) -> Result<WorkLocation, String>;
    async fn update(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        data: UpdateWorkLocationRequest,
    ) -> Result<WorkLocation, String>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), String>;
    async fn count_employees_in_location(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<i64, String>;
}
