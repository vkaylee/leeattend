use uuid::Uuid;
use crate::AppState;
use crate::models::AuthorizedAgent;
use crate::agent_manager::AgentCommand;
use crate::repositories::AgentRepository;

pub struct AgentService;

impl AgentService {
    pub async fn notify_config_update(state: &AppState, agent_id: &str) -> Result<(), String> {
        let devices = state.agent_repo.find_devices_by_agent_id(agent_id).await?;

        let config_msg = serde_json::json!({
            "type": "config_update",
            "devices": devices.into_iter().map(|d| {
                serde_json::json!({
                    "device_id": d.id.to_string(),
                    "ip": d.ip,
                    "port": d.port
                })
            }).collect::<Vec<_>>()
        });

        let command = AgentCommand {
            id: Uuid::new_v4().to_string(),
            r#type: "config_push".to_string(),
            method: "update_config".to_string(),
            device_id: None,
            data: Some(config_msg),
        };

        state
            .agent_manager
            .send_command(agent_id, command)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn authorize_agent(repo: &dyn AgentRepository, id: &str, token: &str) -> Result<Option<AuthorizedAgent>, String> {
        let agent = repo.find_by_id(id).await?;

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

    pub async fn update_last_ip(repo: &dyn AgentRepository, id: &str, ip: &str) -> Result<(), String> {
        repo.update_last_ip(id, ip).await
    }
}