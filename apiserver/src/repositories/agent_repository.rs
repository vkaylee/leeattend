use async_trait::async_trait;
use crate::models::{AuthorizedAgent, Device};

#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, String>;
    async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), String>;
    async fn find_devices_by_agent_id(&self, agent_id: &str) -> Result<Vec<Device>, String>;
}
