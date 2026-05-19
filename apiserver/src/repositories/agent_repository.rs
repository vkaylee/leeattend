use crate::errors::AppError;
use crate::models::{AuthorizedAgent, Device};
use async_trait::async_trait;

use uuid::Uuid;

#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, String>;
    async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), String>;
    async fn update_last_seen(&self, id: &str) -> Result<(), String>;
    async fn find_devices_by_agent_id(&self, workspace_id: Uuid, agent_id: &str) -> Result<Vec<Device>, String>;
    async fn get_workspace_agents(
        &self,
        workspace_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<AuthorizedAgent>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn create_authorized_agent(
        &self,
        id: &str,
        name: &str,
        token: &str,
        workspace_id: Uuid,
        max_concurrent_workers: i32,
        success_cooldown_secs: i32,
        retention_days: i32,
    ) -> Result<(), AppError>;
    async fn agent_exists_in_workspace(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
    ) -> Result<bool, AppError>;
    async fn get_workspace_devices(&self, workspace_id: Uuid) -> Result<Vec<Device>, AppError>;
    async fn get_devices_by_agent_ids(
        &self,
        agent_ids: &[String],
        workspace_id: Uuid,
    ) -> Result<Vec<Device>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn create_device(
        &self,
        id: Uuid,
        agent_id: &str,
        name: &str,
        ip: &str,
        port: i32,
        workspace_id: Uuid,
        brand: &str,
        password: &str,
        timezone: &str,
    ) -> Result<(), AppError>;
    async fn get_device_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Device>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn update_device(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        name: &str,
        ip: &str,
        port: i32,
        is_enabled: bool,
        brand: &str,
        password: Option<&str>,
        timezone: &str,
    ) -> Result<(), AppError>;
    async fn get_agent_id_for_device(
        &self,
        device_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<String>, AppError>;
    async fn delete_device(&self, device_id: Uuid, workspace_id: Uuid) -> Result<u64, AppError>;
    async fn update_agent_trust(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
        is_trusted: bool,
    ) -> Result<u64, AppError>;
    async fn get_agent_by_id_in_workspace(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
    ) -> Result<Option<AuthorizedAgent>, AppError>;
    async fn deactivate_agent(&self, agent_id: &str, workspace_id: Uuid) -> Result<u64, AppError>;
}
