use uuid::Uuid;
use sqlx::PgPool;
use crate::AppState;
use crate::models::{AuthorizedAgent, Device};
use crate::agent_manager::AgentCommand;

pub struct AgentService;

impl AgentService {
    pub async fn notify_config_update(state: &AppState, agent_id: &str) -> Result<(), String> {
        let devices = sqlx::query_as::<_, Device>(
            "SELECT * FROM devices WHERE agent_id = $1 AND is_enabled = true",
        )
        .bind(agent_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

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

    pub async fn authorize_agent(db: &PgPool, id: &str, token: &str) -> Result<Option<AuthorizedAgent>, String> {
        let agent = sqlx::query_as::<_, AuthorizedAgent>(
            "SELECT * FROM authorized_agents WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?;

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

    pub async fn update_last_ip(db: &PgPool, id: &str, ip: &str) -> Result<(), String> {
        sqlx::query("UPDATE authorized_agents SET last_ip = $1 WHERE id = $2")
            .bind(ip)
            .bind(id)
            .execute(db)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
