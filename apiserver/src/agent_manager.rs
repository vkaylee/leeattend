use crate::models::{AgentStatus, DeviceStatus};
use axum::extract::ws::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub devices: Vec<DeviceConfigPush>,
    pub max_concurrent_workers: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CentralMessage {
    ConfigPush { data: ConfigData },
    StatusUpdate {
        agent_id: String,
        devices: Vec<DeviceStatus>,
        timestamp: String,
    },
    CommandResponse {
        command_id: String,
        status: String,
        data: Option<serde_json::Value>,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCommand {
    pub id: String,
    pub r#type: String,
    pub method: String,
    pub device_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct AgentManager {
    pub inner: Arc<AgentManagerInner>,
}

pub struct AgentManagerInner {
    // Map Agent ID -> Sender to push messages to that agent's websocket
    pub agents: RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>,
    // Last seen timestamp for each agent
    pub agent_last_seen: RwLock<HashMap<String, DateTime<Utc>>>,
    // Registry for pending commands: command_id -> Sender to resolve the command result
    pub command_registry: RwLock<HashMap<String, tokio::sync::oneshot::Sender<CentralMessage>>>,
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AgentManagerInner {
                agents: RwLock::new(HashMap::new()),
                agent_last_seen: RwLock::new(HashMap::new()),
                command_registry: RwLock::new(HashMap::new()),
            }),
        }
    }

    pub async fn add_agent(&self, agent_id: String, tx: mpsc::UnboundedSender<Message>) {
        let mut agents = self.inner.agents.write().await;
        agents.insert(agent_id.clone(), tx);

        let mut last_seen = self.inner.agent_last_seen.write().await;
        last_seen.insert(agent_id, Utc::now());
    }

    pub async fn remove_agent(&self, agent_id: &str) {
        let mut agents = self.inner.agents.write().await;
        agents.remove(agent_id);
    }

    pub async fn update_status(&self, agent_id: String) {
        let mut last_seen = self.inner.agent_last_seen.write().await;
        last_seen.insert(agent_id, Utc::now());
    }

    pub async fn send_command(&self, agent_id: &str, command: AgentCommand) -> Result<(), String> {
        let agents = self.inner.agents.read().await;
        if let Some(tx) = agents.get(agent_id) {
            let msg_text = serde_json::to_string(&command).map_err(|e| e.to_string())?;
            tx.send(Message::Text(msg_text))
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err(format!("Agent {agent_id} not found or offline"))
        }
    }

    pub async fn get_all_agents_status(&self) -> Vec<AgentStatus> {
        let last_seen_map = self.inner.agent_last_seen.read().await;
        let active_agents = self.inner.agents.read().await;

        let now = Utc::now();
        let mut results = Vec::new();

        for (agent_id, last_seen) in last_seen_map.iter() {
            let is_online =
                active_agents.contains_key(agent_id) && (now - *last_seen).num_seconds() < 65;

            results.push(AgentStatus {
                agent_id: agent_id.clone(),
                last_seen: *last_seen,
                devices: Vec::new(), // Device info now fetched on-demand
                is_online,
            });
        }
        results
    }


    pub async fn push_message(&self, agent_id: &str, msg: CentralMessage) -> Result<(), String> {
        let agents = self.inner.agents.read().await;
        if let Some(tx) = agents.get(agent_id) {
            let msg_text = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            tx.send(Message::Text(msg_text))
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err(format!("Agent {agent_id} not found or offline"))
        }
    }
    pub async fn shutdown(&self) {
        let agents = self.inner.agents.read().await;
        tracing::info!(
            "Notifying {} connected agents about shutdown...",
            agents.len()
        );

        let shutdown_msg = serde_json::json!({
            "type": "system",
            "code": "SERVER_SHUTDOWN",
            "message": "API Server is shutting down gracefully."
        })
        .to_string();

        for (id, tx) in agents.iter() {
            if let Err(e) = tx.send(Message::Text(shutdown_msg.clone())) {
                tracing::warn!("Failed to send shutdown notice to agent {}: {}", id, e);
            }
        }
    }

    pub async fn is_online(&self, agent_id: &str) -> bool {
        let agents = self.inner.agents.read().await;
        agents.contains_key(agent_id)
    }

    pub async fn get_device_info(&self, agent_id: &str, device_id: &str) -> Result<serde_json::Value, String> {
        let command_id = uuid::Uuid::new_v4().to_string();
        let command = AgentCommand {
            id: command_id.clone(),
            r#type: "command".to_string(),
            method: "get_device_info".to_string(),
            device_id: Some(device_id.to_string()),
            data: None,
        };

        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut registry = self.inner.command_registry.write().await;
            registry.insert(command_id.clone(), tx);
        }

        self.send_command(agent_id, command).await?;

        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(CentralMessage::CommandResponse { status, data, error, .. })) => {
                if status == "success" {
                    data.ok_or_else(|| "No data in success response".to_string())
                } else {
                    Err(error.unwrap_or_else(|| "Unknown error in command response".to_string()))
                }
            }
            Ok(Ok(_)) => Err("Received unexpected message type in response".to_string()),
            Ok(Err(_)) => Err("Response channel closed".to_string()),
            Err(_) => {
                // Cleanup on timeout
                let mut registry = self.inner.command_registry.write().await;
                registry.remove(&command_id);
                Err("Command timed out".to_string())
            }
        }
    }

    pub async fn handle_response(&self, response: CentralMessage) {
        if let CentralMessage::CommandResponse { ref command_id, .. } = response {
            let mut registry = self.inner.command_registry.write().await;
            if let Some(tx) = registry.remove(command_id) {
                let _ = tx.send(response);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_lifecycle() {
        let manager = AgentManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let agent_id = "agent_123".to_string();

        // Test add
        manager.add_agent(agent_id.clone(), tx).await;
        {
            let agents = manager.inner.agents.read().await;
            assert!(agents.contains_key(&agent_id));
            let last_seen = manager.inner.agent_last_seen.read().await;
            assert!(last_seen.contains_key(&agent_id));
        }

        // Test update status (heartbeat only)
        manager.update_status(agent_id.clone()).await;

        let statuses = manager.get_all_agents_status().await;
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].agent_id, agent_id);
        assert!(statuses[0].devices.is_empty());

        // Test send command
        let command = AgentCommand {
            id: "cmd_1".to_string(),
            r#type: "command".to_string(),
            method: "ping".to_string(),
            device_id: None,
            data: None,
        };
        manager.send_command(&agent_id, command).await.unwrap();

        let msg = rx.recv().await.unwrap();
        if let Message::Text(text) = msg {
            assert!(text.contains("\"method\":\"ping\""));
        } else {
            panic!("Expected text message");
        }

        // Test remove
        manager.remove_agent(&agent_id).await;
        {
            let agents = manager.inner.agents.read().await;
            assert!(!agents.contains_key(&agent_id));
        }
    }
}
