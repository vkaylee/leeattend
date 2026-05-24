use crate::agent_manager::{AgentManager, CentralMessage, ConfigData, DeviceConfigPush};
use crate::models::AuthorizedAgent;
use crate::repositories::AgentRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct AgentService {
    repo: Arc<dyn AgentRepository>,
    manager: AgentManager,
}

impl AgentService {
    pub fn new(repo: Arc<dyn AgentRepository>, manager: AgentManager) -> Self {
        Self { repo, manager }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn notify_config_update(
        &self,
        workspace_id: Uuid,
        agent_id: &str,
    ) -> Result<(), String> {
        let devices = self
            .repo
            .find_devices_by_agent_id(&crate::diesel_pool::TenantAuthToken::system_override(workspace_id), agent_id)
            .await?;
        let agent = self.repo.find_by_id(agent_id).await?;
        let max_workers = agent.as_ref().map(|a| a.max_concurrent_workers);
        let cooldown = agent.as_ref().map(|a| a.success_cooldown_secs);
        let retention = agent.as_ref().map(|a| a.retention_days);

        let config_data = ConfigData {
            devices: devices
                .into_iter()
                .map(|d| DeviceConfigPush {
                    device_id: d.id.to_string(),
                    name: d.name,
                    ip: d.ip,
                    port: d.port,
                    brand: d.brand,
                    password: d.password,
                    timezone: Some(d.timezone),
                    is_enabled: d.is_enabled,
                })
                .collect(),
            max_concurrent_workers: max_workers,
            success_cooldown_secs: cooldown,
            retention_days: retention,
        };

        self.manager
            .push_message(agent_id, CentralMessage::ConfigPush { data: config_data })
            .await
    }

    #[tracing::instrument(skip_all, err)]
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

    #[tracing::instrument(skip_all, err)]
    pub async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), String> {
        self.repo.update_last_ip(id, ip).await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_last_seen(&self, id: &str) -> Result<(), String> {
        self.repo.update_last_seen(id).await
    }
}

#[cfg(test)]
#[allow(clippy::unimplemented)]
mod tests {
    use super::*;
    use crate::models::{AuthorizedAgent, Device};
    use async_trait::async_trait;
    use std::sync::Arc;
    use uuid::Uuid;

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
                    max_concurrent_workers: 5,
                    success_cooldown_secs: 300,
                    retention_days: 365,
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
                    max_concurrent_workers: 5,
                    success_cooldown_secs: 300,
                    retention_days: 365,
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

        async fn find_devices_by_agent_id(
            &self,
            _workspace_id: Uuid,
            _agent_id: &str,
        ) -> Result<Vec<Device>, String> {
            Ok(vec![])
        }

        async fn get_workspace_agents(
            &self,
            _workspace_id: Uuid,
            _include_inactive: bool,
        ) -> Result<Vec<AuthorizedAgent>, crate::errors::AppError> {
            Ok(vec![])
        }

        async fn create_authorized_agent(
            &self,
            _id: &str,
            _name: &str,
            _token: &str,
            _workspace_id: Uuid,
            _max_concurrent_workers: i32,
            _success_cooldown_secs: i32,
            _retention_days: i32,
        ) -> Result<(), crate::errors::AppError> {
            Ok(())
        }

        async fn agent_exists_in_workspace(
            &self,
            _agent_id: &str,
            _workspace_id: Uuid,
        ) -> Result<bool, crate::errors::AppError> {
            Ok(true)
        }

        async fn get_workspace_devices(
            &self,
            _workspace_id: Uuid,
        ) -> Result<Vec<Device>, crate::errors::AppError> {
            Ok(vec![])
        }

        async fn get_devices_by_agent_ids(
            &self,
            _agent_ids: &[String],
            _workspace_id: Uuid,
        ) -> Result<Vec<Device>, crate::errors::AppError> {
            Ok(vec![])
        }

        async fn create_device(
            &self,
            _id: Uuid,
            _agent_id: &str,
            _name: &str,
            _ip: &str,
            _port: i32,
            _workspace_id: Uuid,
            _brand: &str,
            _password: &str,
            _timezone: &str,
        ) -> Result<(), crate::errors::AppError> {
            Ok(())
        }

        async fn get_device_by_id(
            &self,
            _id: Uuid,
            _workspace_id: Uuid,
        ) -> Result<Option<Device>, crate::errors::AppError> {
            Ok(None)
        }

        async fn update_device(
            &self,
            _id: Uuid,
            _workspace_id: Uuid,
            _name: &str,
            _ip: &str,
            _port: i32,
            _is_enabled: bool,
            _brand: &str,
            _password: Option<&str>,
            _timezone: &str,
        ) -> Result<(), crate::errors::AppError> {
            Ok(())
        }

        async fn get_agent_id_for_device(
            &self,
            _device_id: Uuid,
            _workspace_id: Uuid,
        ) -> Result<Option<String>, crate::errors::AppError> {
            Ok(None)
        }

        async fn delete_device(
            &self,
            _device_id: Uuid,
            _workspace_id: Uuid,
        ) -> Result<u64, crate::errors::AppError> {
            Ok(1)
        }

        async fn update_agent_trust(
            &self,
            _agent_id: &str,
            _workspace_id: Uuid,
            _is_trusted: bool,
        ) -> Result<u64, crate::errors::AppError> {
            Ok(1)
        }

        async fn get_agent_by_id_in_workspace(
            &self,
            _agent_id: &str,
            _workspace_id: Uuid,
        ) -> Result<Option<AuthorizedAgent>, crate::errors::AppError> {
            Ok(None)
        }

        async fn deactivate_agent(
            &self,
            _agent_id: &str,
            _workspace_id: Uuid,
        ) -> Result<u64, crate::errors::AppError> {
            Ok(1)
        }
    }

    #[tokio::test]
    async fn test_authorize_agent_success() {
        let repo = Arc::new(MockAgentRepo);
        let manager = AgentManager::new(None);
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
        let manager = AgentManager::new(None);
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
        let manager = AgentManager::new(None);
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
        let manager = AgentManager::new(None);
        let service = AgentService::new(repo, manager);

        let res = service.authorize_agent("unknown", "any").await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }
}
