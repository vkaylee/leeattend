use crate::errors::AppError;
use crate::models::{AuthorizedAgent, Device};
use async_trait::async_trait;

use uuid::Uuid;

pub struct CreateAuthorizedAgentArgs<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub token: &'a str,
    pub auth_token: &'a crate::diesel_pool::TenantAuthToken,
    pub max_concurrent_workers: i32,
    pub success_cooldown_secs: i32,
    pub retention_days: i32,
}

pub struct CreateDeviceArgs<'a> {
    pub id: Uuid,
    pub agent_id: &'a str,
    pub name: &'a str,
    pub ip: &'a str,
    pub port: i32,
    pub auth_token: &'a crate::diesel_pool::TenantAuthToken,
    pub brand: &'a str,
    pub password: &'a str,
    pub timezone: &'a str,
}

pub struct UpdateDeviceArgs<'a> {
    pub id: Uuid,
    pub auth_token: &'a crate::diesel_pool::TenantAuthToken,
    pub name: &'a str,
    pub ip: &'a str,
    pub port: i32,
    pub is_enabled: bool,
    pub brand: &'a str,
    pub password: Option<&'a str>,
    pub timezone: &'a str,
}

#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, AppError>;
    async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), AppError>;
    async fn update_last_seen(&self, id: &str) -> Result<(), AppError>;
    async fn find_devices_by_agent_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        agent_id: &str,
    ) -> Result<Vec<Device>, AppError>;
    async fn get_workspace_agents(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        include_inactive: bool,
    ) -> Result<Vec<AuthorizedAgent>, AppError>;
    async fn create_authorized_agent(
        &self,
        args: CreateAuthorizedAgentArgs<'_>,
    ) -> Result<(), AppError>;
    async fn agent_exists_in_workspace(
        &self,
        agent_id: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<bool, AppError>;
    async fn get_workspace_devices(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Device>, AppError>;
    async fn get_devices_by_agent_ids(
        &self,
        agent_ids: &[String],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Device>, AppError>;
    async fn create_device(
        &self,
        args: CreateDeviceArgs<'_>,
    ) -> Result<(), AppError>;
    async fn get_device_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Device>, AppError>;
    async fn update_device(
        &self,
        args: UpdateDeviceArgs<'_>,
    ) -> Result<(), AppError>;
    async fn get_agent_id_for_device(
        &self,
        device_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<String>, AppError>;
    async fn delete_device(
        &self,
        device_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError>;
    async fn update_agent_trust(
        &self,
        agent_id: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        is_trusted: bool,
    ) -> Result<u64, AppError>;
    async fn get_agent_by_id_in_workspace(
        &self,
        agent_id: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<AuthorizedAgent>, AppError>;
    async fn deactivate_agent(
        &self,
        agent_id: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError>;
    async fn update_authorized_agent(&self, agent: &AuthorizedAgent) -> Result<(), AppError>;
}
