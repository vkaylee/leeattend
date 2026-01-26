use crate::agent_manager::{AgentManager, CentralMessage, ConfigData, DeviceConfigPush};
use crate::models::AuthorizedAgent;
use crate::repositories::AgentRepository;
use std::sync::Arc;

pub struct AgentService {
    repo: Arc<dyn AgentRepository>,
    manager: AgentManager,
}

impl AgentService {
    pub fn new(repo: Arc<dyn AgentRepository>, manager: AgentManager) -> Self {
        Self { repo, manager }
    }

    pub async fn notify_config_update(&self, agent_id: &str) -> Result<(), String> {
        let devices = self.repo.find_devices_by_agent_id(agent_id).await?;

        let config_data = ConfigData {
            devices: devices.into_iter().map(|d| {
                DeviceConfigPush {
                    device_id: d.id.to_string(),
                    name: d.name,
                    ip: d.ip,
                    port: d.port
                }
            }).collect()
        };

        self.manager
            .push_message(agent_id, CentralMessage::ConfigPush { data: config_data })
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn authorize_agent(
        &self,
        id: &str,
        token: &str,
    ) -> Result<Option<AuthorizedAgent>, String> {
        let agent = self.repo.find_by_id(id).await?;

        if let Some(ref a) = agent {
            if a.token != token {
                return Err("INVALID_TOKEN".to_string());
            }
            if !a.is_trusted {
                return Err("AGENT_NOT_TRUSTED".to_string());
            }
        }

        Ok(agent)
    }

    pub async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), String> {
        self.repo.update_last_ip(id, ip).await
    }

    pub async fn update_last_seen(&self, id: &str) -> Result<(), String> {
        self.repo.update_last_seen(id).await
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use super::*;
    use crate::models::{AuthorizedAgent, Device};
    use async_trait::async_trait;
    use std::sync::Arc;

    struct MockAgentRepo;

    #[async_trait]
    impl AgentRepository for MockAgentRepo {
        async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, String> {
            if id == "trusted_agent" {
                Ok(Some(AuthorizedAgent {
                    id: id.to_string(),
                    name: "Trusted".to_string(),
                    token: "valid_token".to_string(),
                    is_trusted: true,
                    last_ip: None,
                    created_at: chrono::Utc::now(),
                    last_seen_at: chrono::Utc::now(),
                    is_active: true,
                    workspace_id: Uuid::new_v4(),
                }))
            } else if id == "untrusted_agent" {
                Ok(Some(AuthorizedAgent {
                    id: id.to_string(),
                    name: "Untrusted".to_string(),
                    token: "valid_token".to_string(),
                    is_trusted: false,
                    last_ip: None,
                    created_at: chrono::Utc::now(),
                    last_seen_at: chrono::Utc::now(),
                    is_active: true,
                    workspace_id: Uuid::new_v4(),
                }))
            } else {
                Ok(None)
            }
        }

        async fn update_last_ip(&self, _id: &str, _ip: &str) -> Result<(), String> {
            Ok(())
        }

        async fn update_last_seen(&self, _id: &str) -> Result<(), String> {
            Ok(())
        }

        async fn find_devices_by_agent_id(&self, _agent_id: &str) -> Result<Vec<Device>, String> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_authorize_agent_success() {
        let repo = Arc::new(MockAgentRepo);
        let manager = AgentManager::new();
        let service = AgentService::new(repo, manager);

        let res = service
            .authorize_agent("trusted_agent", "valid_token")
            .await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_authorize_agent_wrong_token() {
        let repo = Arc::new(MockAgentRepo);
        let manager = AgentManager::new();
        let service = AgentService::new(repo, manager);

        let res = service
            .authorize_agent("trusted_agent", "wrong_token")
            .await;
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), "INVALID_TOKEN");
    }

    #[tokio::test]
    async fn test_authorize_agent_untrusted() {
        let repo = Arc::new(MockAgentRepo);
        let manager = AgentManager::new();
        let service = AgentService::new(repo, manager);

        let res = service
            .authorize_agent("untrusted_agent", "valid_token")
            .await;
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), "AGENT_NOT_TRUSTED");
    }

    #[tokio::test]
    async fn test_authorize_agent_not_found() {
        let repo = Arc::new(MockAgentRepo);
        let manager = AgentManager::new();
        let service = AgentService::new(repo, manager);

        let res = service.authorize_agent("unknown", "any").await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }
}
