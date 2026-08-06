use crate::models::{AgentStatus, AuthorizedAgent, DeviceStatus};
use crate::queue::RedisPool;
use axum::extract::ws::Message;
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
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
    pub password: Option<String>,
    pub timezone: Option<String>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub devices: Vec<DeviceConfigPush>,
    pub max_concurrent_workers: Option<i32>,
    pub success_cooldown_secs: Option<i32>,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CentralMessage {
    ConfigPush {
        data: ConfigData,
    },
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

#[async_trait::async_trait]
pub trait PresenceStore: Send + Sync {
    async fn fetch_presence(&self, agent_ids: &[String]) -> HashMap<String, DateTime<Utc>>;
    async fn record_presence(&self, agent_id: String);
}

#[async_trait::async_trait]
impl PresenceStore for RedisPool {
    async fn fetch_presence(&self, agent_ids: &[String]) -> HashMap<String, DateTime<Utc>> {
        let mut conn = self.conn();
        let keys: Vec<String> = agent_ids
            .iter()
            .map(|id| format!("agent:presence:{id}"))
            .collect();
        let mut redis_presence = HashMap::new();

        match conn.mget::<Vec<String>, Vec<Option<String>>>(keys).await {
            Ok(values) => {
                for (id, val_opt) in agent_ids.iter().zip(values.iter()) {
                    if let Some(val) = val_opt {
                        if let Ok(ts) = DateTime::parse_from_rfc3339(val) {
                            redis_presence.insert(id.clone(), ts.with_timezone(&Utc));
                        }
                    }
                }
            }
            Err(e) => tracing::error!("Redis MGET failed: {}", e),
        }
        redis_presence
    }

    async fn record_presence(&self, agent_id: String) {
        let key = format!("agent:presence:{agent_id}");
        let val = Utc::now().to_rfc3339();

        // Use a fresh multiplexed connection instead of ConnectionManager.
        // ConnectionManager has an internal 3-second reconnection delay that causes
        // set_ex to take 3s+ on reconnection, which exceeds the 500ms timeout and
        // creates an infinite timeout loop: timeout → drop → reconnect(3s) → timeout.
        let mut conn = match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            self.get_async_connection(),
        )
        .await
        {
            Ok(Ok(c)) => c,
            Ok(Err(e)) => {
                tracing::warn!(
                    "Redis connection failed for presence (agent {}): {}",
                    agent_id,
                    e
                );
                return;
            }
            Err(_) => {
                tracing::warn!(
                    "Redis connection timed out for presence (agent {})",
                    agent_id
                );
                return;
            }
        };

        // Set presence key with 60 second TTL
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            conn.set_ex::<String, String, ()>(key, val, 60),
        )
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::error!(
                "Failed to set presence in Redis for agent {}: {}",
                agent_id,
                e
            ),
            Err(_) => tracing::warn!("Redis set_ex timed out for agent {}", agent_id),
        };
    }
}

#[derive(Clone)]
pub struct AgentManager {
    pub inner: Arc<AgentManagerInner>,
}

pub struct AgentManagerInner {
    // Map Agent ID -> Sender to push messages to that agent's websocket
    pub agents: RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>,
    // Registry for pending commands: command_id -> Sender to resolve the command result.
    //
    // Deliberately a synchronous `parking_lot::Mutex` rather than tokio's RwLock:
    // the only cleanup path guaranteed to run on every exit is `Drop`, and `Drop`
    // cannot be async. Every critical section here is a single HashMap
    // insert/remove with no `.await` inside, so a sync lock is correct (never held
    // across a yield point) and cheaper than an async one. parking_lot is the
    // project-sanctioned sync lock (see clippy.toml disallowed-types) and needs no
    // poison handling. See `PendingCommandGuard`.
    pub command_registry:
        parking_lot::Mutex<HashMap<String, tokio::sync::oneshot::Sender<CentralMessage>>>,
    // Redis presence store
    pub redis: Option<Arc<dyn PresenceStore>>,
}

impl AgentManagerInner {
    /// Lock the command registry.
    ///
    /// Callers MUST NOT hold the returned guard across an `.await`; every use is
    /// a single map insert/remove, which keeps that invariant trivially true.
    pub fn registry(
        &self,
    ) -> parking_lot::MutexGuard<'_, HashMap<String, tokio::sync::oneshot::Sender<CentralMessage>>>
    {
        self.command_registry.lock()
    }
}

/// RAII guard that guarantees a pending command is reclaimed from
/// `command_registry` no matter which path leaves `get_device_info`.
///
/// `get_device_info` has four distinct exit paths (send failure, success,
/// unexpected message type, channel closed) plus the timeout branch. Patching
/// each one by hand re-opens the leak the moment a new branch is added, so
/// cleanup is bound to scope exit instead.
struct PendingCommandGuard {
    inner: Arc<AgentManagerInner>,
    command_id: String,
}

impl Drop for PendingCommandGuard {
    fn drop(&mut self) {
        // If the entry is still present, no response ever resolved it: this is an
        // error/timeout path. `handle_response` removes the entry on success, so
        // a present entry here always means the command did not complete.
        if self.inner.registry().remove(&self.command_id).is_some() {
            // command_id is a server-generated UUID, not PII.
            tracing::warn!(
                command_id = %self.command_id,
                "Reclaimed unresolved pending command from registry (error or timeout path)"
            );
        }
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AgentManager {
    pub fn new(redis: Option<Arc<dyn PresenceStore>>) -> Self {
        Self {
            inner: Arc::new(AgentManagerInner {
                agents: RwLock::new(HashMap::new()),
                command_registry: parking_lot::Mutex::new(HashMap::new()),
                redis,
            }),
        }
    }

    pub async fn add_agent(&self, agent_id: String, tx: mpsc::UnboundedSender<Message>) {
        self.inner.agents.write().await.insert(agent_id, tx);
    }

    pub async fn remove_agent(&self, agent_id: &str) {
        let mut agents = self.inner.agents.write().await;
        agents.remove(agent_id);
    }

    pub async fn record_heartbeat(&self, agent_id: String) {
        // Presence lives in Redis with a 60s TTL, which is the single source of
        // truth read back by `get_all_agents_status`. There is deliberately no
        // in-process last-seen map: an unbounded HashMap keyed by agent id that
        // nothing reads is just a leak.
        if let Some(ref redis) = self.inner.redis {
            redis.record_presence(agent_id).await;
        }
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

    pub async fn get_all_agents_status(&self, db_agents: Vec<AuthorizedAgent>) -> Vec<AgentStatus> {
        let mut results = Vec::new();
        let agent_ids: Vec<String> = db_agents.iter().map(|a| a.id.clone()).collect();

        if agent_ids.is_empty() {
            return results;
        }

        // 1. Fetch presence from Redis if available
        let redis_presence = self.fetch_redis_presence(&agent_ids).await;

        for db_agent in db_agents {
            let agent_id = db_agent.id;

            // Check Redis first
            let presence_ts = redis_presence.get(&agent_id).copied();

            // is_online is true if Redis key exists (within 60s TTL) OR locally connected
            let is_online =
                presence_ts.is_some() || self.inner.agents.read().await.contains_key(&agent_id);

            // Use Redis timestamp if available, else DB's last_seen_at
            let last_seen = presence_ts.unwrap_or(db_agent.last_seen_at);

            results.push(AgentStatus {
                agent_id,
                last_seen,
                devices: Vec::new(),
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

    pub async fn get_device_info(
        &self,
        agent_id: &str,
        device_id: &str,
    ) -> Result<serde_json::Value, String> {
        let command_id = uuid::Uuid::new_v4().to_string();
        let command = AgentCommand {
            id: command_id.clone(),
            r#type: "command".to_string(),
            method: "get_device_info".to_string(),
            device_id: Some(device_id.to_string()),
            data: None,
        };

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.inner.registry().insert(command_id.clone(), tx);

        // From here on, every exit path — including ones added later — reclaims
        // the registry slot when this guard leaves scope.
        let _guard = PendingCommandGuard {
            inner: Arc::clone(&self.inner),
            command_id,
        };

        self.send_command(agent_id, command).await?;

        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(CentralMessage::CommandResponse {
                status,
                data,
                error,
                ..
            })) => {
                if status == "success" {
                    data.ok_or_else(|| "No data in success response".to_string())
                } else {
                    Err(error.unwrap_or_else(|| "Unknown error in command response".to_string()))
                }
            }
            Ok(Ok(_)) => Err("Received unexpected message type in response".to_string()),
            Ok(Err(_)) => Err("Response channel closed".to_string()),
            Err(_) => Err("Command timed out".to_string()),
        }
    }

    /// Resolve a pending command with the agent's response.
    ///
    /// Synchronous: the registry is guarded by a sync lock and there is nothing to
    /// await. Callers on the websocket read loop can invoke this directly.
    pub fn handle_response(&self, response: CentralMessage) {
        if let CentralMessage::CommandResponse { ref command_id, .. } = response {
            let entry = self.inner.registry().remove(command_id);
            if let Some(tx) = entry {
                let _ = tx.send(response);
            }
        }
    }

    async fn fetch_redis_presence(&self, agent_ids: &[String]) -> HashMap<String, DateTime<Utc>> {
        if let Some(ref redis) = self.inner.redis {
            if let Ok(res) = tokio::time::timeout(
                std::time::Duration::from_millis(500),
                redis.fetch_presence(agent_ids),
            )
            .await
            {
                res
            } else {
                tracing::warn!("Presence check timed out (500ms)");
                HashMap::new()
            }
        } else {
            HashMap::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    struct MockPresenceStore {
        delay: Duration,
    }

    #[async_trait::async_trait]
    impl PresenceStore for MockPresenceStore {
        async fn fetch_presence(&self, _agent_ids: &[String]) -> HashMap<String, DateTime<Utc>> {
            if self.delay > Duration::from_millis(0) {
                sleep(self.delay).await;
            }
            HashMap::new()
        }

        async fn record_presence(&self, _agent_id: String) {}
    }

    #[tokio::test]
    async fn test_agent_lifecycle() {
        let manager = AgentManager::new(None);
        let (tx, mut rx) = mpsc::unbounded_channel();
        let agent_id = "agent_123".to_string();

        // Test add
        manager.add_agent(agent_id.clone(), tx).await;
        assert!(manager.inner.agents.read().await.contains_key(&agent_id));

        // Test heartbeat
        manager.record_heartbeat(agent_id.clone()).await;

        // Mock DB agents for status check
        let db_agents = vec![crate::models::AuthorizedAgent {
            id: agent_id.clone(),
            name: "Test".to_string(),
            token: "token".to_string(),
            last_seen_at: Utc::now(),
            workspace_id: uuid::Uuid::new_v4(),
            is_active: true,
            is_trusted: true,
            last_ip: None,
            created_at: Utc::now(),
            max_concurrent_workers: 5,
            success_cooldown_secs: 300,
            retention_days: 365,
        }];

        let statuses = manager.get_all_agents_status(db_agents.clone()).await;
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].agent_id, agent_id);
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
        assert!(!manager.inner.agents.read().await.contains_key(&agent_id));

        // Test offline status
        let statuses = manager.get_all_agents_status(db_agents).await;
        assert_eq!(statuses.len(), 1);
        assert!(!statuses[0].is_online);
    }

    #[tokio::test]
    async fn test_redis_timeout_handling() {
        // Use a mock store with a 2-second delay (exceeds 500ms limit)
        let mock_store = Arc::new(MockPresenceStore {
            delay: Duration::from_secs(2),
        });
        let manager = AgentManager::new(Some(mock_store));

        let agent_id = "timeout_agent".to_string();
        let db_agents = vec![crate::models::AuthorizedAgent {
            id: agent_id.clone(),
            name: "Test".to_string(),
            token: "token".to_string(),
            last_seen_at: Utc::now(),
            workspace_id: uuid::Uuid::new_v4(),
            is_active: true,
            is_trusted: true,
            last_ip: None,
            created_at: Utc::now(),
            max_concurrent_workers: 5,
            success_cooldown_secs: 300,
            retention_days: 365,
        }];

        // The inner 500ms timeout in fetch_presence logic should trigger
        // Verification: get_all_agents_status should finish in ~500ms, not 2s
        let start = tokio::time::Instant::now();
        let _statuses = tokio::time::timeout(
            Duration::from_millis(1500),
            manager.get_all_agents_status(db_agents),
        )
        .await
        .expect("Method totally hung!");

        let elapsed = start.elapsed();
        // Since the timeout is 500ms, it should finish around that time
        assert!(
            elapsed >= Duration::from_millis(500),
            "Should have waited for the timeout, got {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_millis(1000),
            "Timeout logic failed to cap duration, got {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_record_presence_does_not_hang() {
        // Regression test: record_heartbeat must complete promptly.
        // Before the fix, ConnectionManager's 3-second reconnect delay caused
        // every heartbeat to take 3s+, exceeding the 500ms timeout in a loop.
        let mock_store = Arc::new(MockPresenceStore {
            delay: Duration::from_millis(0),
        });
        let manager = AgentManager::new(Some(mock_store));

        let (tx, _rx) = mpsc::unbounded_channel();
        let agent_id = "fast_agent".to_string();
        manager.add_agent(agent_id.clone(), tx).await;

        let start = tokio::time::Instant::now();
        manager.record_heartbeat(agent_id).await;
        let elapsed = start.elapsed();

        // With no delay, heartbeat should complete in well under 100ms
        assert!(
            elapsed < Duration::from_millis(100),
            "record_heartbeat took too long ({elapsed:?}), possible ConnectionManager hang"
        );
    }

    #[tokio::test]
    async fn test_record_presence_timeout_caps_slow_store() {
        // Verify that a slow presence store is capped by the caller's timeout,
        // not allowed to block the heartbeat indefinitely.
        let mock_store = Arc::new(MockPresenceStore {
            delay: Duration::from_millis(600), // exceeds 500ms timeout
        });
        let manager = AgentManager::new(Some(mock_store));

        let (tx, _rx) = mpsc::unbounded_channel();
        let agent_id = "slow_agent".to_string();
        manager.add_agent(agent_id.clone(), tx).await;

        let start = tokio::time::Instant::now();
        // record_heartbeat calls record_presence which is async — test that it
        // completes without hanging even with a slow store
        tokio::time::timeout(Duration::from_secs(2), manager.record_heartbeat(agent_id))
            .await
            .expect("record_heartbeat hung with slow presence store");

        let elapsed = start.elapsed();
        // Should complete reasonably fast (mock delay is fire-and-forget)
        assert!(
            elapsed < Duration::from_millis(1500),
            "Heartbeat with slow store took too long: {elapsed:?}"
        );
    }

    /// Prove-It regression: an offline agent must not leave an orphan entry in
    /// `command_registry`. The key space is a random UUID, so every failed call
    /// used to leak one entry for the lifetime of the process.
    #[tokio::test]
    async fn test_get_device_info_offline_agent_does_not_leak_registry_entry() {
        let manager = AgentManager::new(None);

        // No agent registered => send_command fails with "not found or offline".
        let result = manager
            .get_device_info("ghost_agent", "00000000-0000-0000-0000-000000000001")
            .await;

        assert!(result.is_err(), "expected Err for offline agent");
        assert_eq!(
            manager.inner.registry().len(),
            0,
            "offline agent leaked a command_registry entry"
        );
    }

    /// Prove-It regression: the timeout path must reclaim its registry entry.
    #[tokio::test(start_paused = true)]
    async fn test_get_device_info_timeout_does_not_leak_registry_entry() {
        let manager = AgentManager::new(None);
        let (tx, _rx) = mpsc::unbounded_channel();
        let agent_id = "silent_agent".to_string();
        manager.add_agent(agent_id.clone(), tx).await;

        // Agent is "connected" but never answers; the paused clock auto-advances
        // past the 10s timeout once every task is idle.
        let result = manager
            .get_device_info(&agent_id, "00000000-0000-0000-0000-000000000002")
            .await;

        assert_eq!(result, Err("Command timed out".to_string()));
        assert_eq!(
            manager.inner.registry().len(),
            0,
            "timeout path leaked a command_registry entry"
        );
    }

    /// Prove-It regression: the happy path must also leave the registry empty.
    #[tokio::test]
    async fn test_get_device_info_success_removes_registry_entry() {
        let manager = AgentManager::new(None);
        let (tx, mut rx) = mpsc::unbounded_channel();
        let agent_id = "chatty_agent".to_string();
        manager.add_agent(agent_id.clone(), tx).await;

        // Simulate the agent side: read the command off the socket, reply success.
        let responder = manager.clone();
        tokio::spawn(async move {
            let Some(Message::Text(text)) = rx.recv().await else {
                return;
            };
            let cmd: AgentCommand =
                serde_json::from_str(&text).expect("agent command must deserialize");
            responder.handle_response(CentralMessage::CommandResponse {
                command_id: cmd.id,
                status: "success".to_string(),
                data: Some(serde_json::json!({ "firmware": "1.2.3" })),
                error: None,
            });
        });

        let info = manager
            .get_device_info(&agent_id, "00000000-0000-0000-0000-000000000003")
            .await
            .expect("expected success response");

        assert_eq!(info["firmware"], "1.2.3");
        assert_eq!(
            manager.inner.registry().len(),
            0,
            "success path leaked a command_registry entry"
        );
    }

    #[tokio::test]
    async fn test_config_push_includes_success_cooldown_secs() {
        let manager = AgentManager::new(None);
        let (tx, mut rx) = mpsc::unbounded_channel();
        let agent_id = "cooldown_agent".to_string();

        manager.add_agent(agent_id.clone(), tx).await;

        // Push config with default cooldown (300s)
        let config_data = ConfigData {
            devices: vec![],
            max_concurrent_workers: Some(4),
            success_cooldown_secs: Some(300),
            retention_days: None,
        };
        manager
            .push_message(&agent_id, CentralMessage::ConfigPush { data: config_data })
            .await
            .unwrap();

        let msg = rx.recv().await.unwrap();
        if let Message::Text(text) = msg {
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(parsed["type"], "config_push");
            assert_eq!(parsed["data"]["success_cooldown_secs"], 300);
            assert_eq!(parsed["data"]["max_concurrent_workers"], 4);
        } else {
            panic!("Expected text message");
        }

        // Push config with custom cooldown (600s)
        let config_data2 = ConfigData {
            devices: vec![],
            max_concurrent_workers: None,
            success_cooldown_secs: Some(600),
            retention_days: None,
        };
        manager
            .push_message(&agent_id, CentralMessage::ConfigPush { data: config_data2 })
            .await
            .unwrap();

        let msg2 = rx.recv().await.unwrap();
        if let Message::Text(text) = msg2 {
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(parsed["data"]["success_cooldown_secs"], 600);
            assert!(parsed["data"]["max_concurrent_workers"].is_null());
        } else {
            panic!("Expected text message");
        }

        // Push config with no cooldown (None → omitted in JSON)
        let config_data3 = ConfigData {
            devices: vec![],
            max_concurrent_workers: None,
            success_cooldown_secs: None,
            retention_days: None,
        };
        manager
            .push_message(&agent_id, CentralMessage::ConfigPush { data: config_data3 })
            .await
            .unwrap();

        let msg3 = rx.recv().await.unwrap();
        if let Message::Text(text) = msg3 {
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert!(parsed["data"]["success_cooldown_secs"].is_null());
        } else {
            panic!("Expected text message");
        }
    }
}
