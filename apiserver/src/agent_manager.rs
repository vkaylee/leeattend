use crate::models::{AgentStatus, DeviceStatus};
use axum::extract::ws::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

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
    // Cache for device statuses per agent
    pub device_cache: RwLock<HashMap<String, Vec<DeviceStatus>>>,
    // Last seen timestamp for each agent
    pub agent_last_seen: RwLock<HashMap<String, DateTime<Utc>>>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AgentManagerInner {
                agents: RwLock::new(HashMap::new()),
                device_cache: RwLock::new(HashMap::new()),
                agent_last_seen: RwLock::new(HashMap::new()),
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

    pub async fn update_status(&self, agent_id: String, devices: Vec<DeviceStatus>) {
        let mut last_seen = self.inner.agent_last_seen.write().await;
        last_seen.insert(agent_id.clone(), Utc::now());

        let mut cache = self.inner.device_cache.write().await;
        cache.insert(agent_id, devices);
    }

    pub async fn send_command(&self, agent_id: &str, command: AgentCommand) -> Result<(), String> {
        let agents = self.inner.agents.read().await;
        if let Some(tx) = agents.get(agent_id) {
            let msg_text = serde_json::to_string(&command).map_err(|e| e.to_string())?;
            tx.send(Message::Text(msg_text))
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err(format!("Agent {} not found or offline", agent_id))
        }
    }

    pub async fn get_all_agents_status(&self) -> Vec<AgentStatus> {
        let last_seen_map = self.inner.agent_last_seen.read().await;
        let device_cache = self.inner.device_cache.read().await;
        let active_agents = self.inner.agents.read().await;

        let now = Utc::now();
        let mut results = Vec::new();

        for (agent_id, last_seen) in last_seen_map.iter() {
            let devices = device_cache.get(agent_id).cloned().unwrap_or_default();
            let is_online =
                active_agents.contains_key(agent_id) && (now - *last_seen).num_seconds() < 65;

            results.push(AgentStatus {
                agent_id: agent_id.clone(),
                last_seen: *last_seen,
                devices,
                is_online,
            });
        }
        results
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

        // Test update status
        let devices = vec![DeviceStatus {
            device_id: "D1".to_string(),
            status: "online".to_string(),
            last_sync: None,
            error: None,
        }];
        manager
            .update_status(agent_id.clone(), devices.clone())
            .await;

        let statuses = manager.get_all_agents_status().await;
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].agent_id, agent_id);
        assert_eq!(statuses[0].devices.len(), 1);
        assert!(statuses[0].is_online);

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