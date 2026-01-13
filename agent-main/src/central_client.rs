use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::http::HeaderValue,
    tungstenite::Message, MaybeTlsStream, WebSocketStream,
};

use crate::database::Database;
use crate::models::{CentralMessage, AttendanceData};

pub struct CentralClient {
    url: String,
    agent_id: String,
    token: String,
    db: Database,
    tx: mpsc::Sender<CentralMessage>,
}

impl CentralClient {
    pub fn new(url: String, agent_id: String, token: String, db: Database) -> (Self, mpsc::Receiver<CentralMessage>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                url,
                agent_id,
                token,
                db,
                tx,
            },
            rx,
        )
    }

    pub async fn push_log(&self, device_id: String, log: AttendanceData) {
        let msg = CentralMessage::Attendance {
            msg_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.agent_id.clone(),
            device_id,
            data: log,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        if let Err(e) = self.tx.send(msg).await {
            error!("Failed to queue log: {}", e);
        }
    }
}

#[derive(Debug, PartialEq)]
enum RetryMode {
    Initial,
    Backoff,
}

struct RetryManager {
    retry_count: u32,
    mode: RetryMode,
}

impl RetryManager {
    fn new() -> Self {
        Self {
            retry_count: 0,
            mode: RetryMode::Initial,
        }
    }

    fn get_delay(&self) -> Duration {
        if self.retry_count == 0 {
            return Duration::from_secs(0);
        }

        match self.mode {
            RetryMode::Initial => Duration::from_secs(5),
            RetryMode::Backoff => calculate_backoff(self.retry_count),
        }
    }

    fn record_connection_failed(&mut self) {
        if self.retry_count < 10 {
            self.retry_count += 1;
        }
    }

    fn record_connection_established(&mut self) {
        // We don't reset count yet. 
        // We wait for record_stable to clear the counter.
    }

    fn record_stable(&mut self) {
        self.retry_count = 0;
        self.mode = RetryMode::Initial;
    }

    fn record_dropped_prematurely(&mut self) {
        self.mode = RetryMode::Backoff;
        if self.retry_count < 10 {
            self.retry_count += 1;
        }
    }
}

impl CentralClient {
    pub async fn start(&self, mut rx: mpsc::Receiver<CentralMessage>) {
        let url = self.url.clone();
        let agent_id = self.agent_id.clone();
        let token = self.token.clone();
        let tx = self.tx.clone();
        let db = self.db.clone();

        // Heartbeat Task - sends Heartbeat to channel periodically
        let hb_agent_id = agent_id.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                let hb = CentralMessage::Heartbeat {
                    agent_id: hb_agent_id.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = tx.send(hb).await {
                    error!("Failed to send heartbeat to channel: {}", e);
                }
            }
        });

        tokio::spawn(async move {
            let mut manager = RetryManager::new();

            loop {
                let delay = manager.get_delay();
                if delay > Duration::from_secs(0) {
                    match manager.mode {
                        RetryMode::Initial => info!("Initial connection failed. Retrying in {:?}...", delay),
                        RetryMode::Backoff => warn!("Backoff: Waiting {:?} before reconnecting (Retry {})", delay, manager.retry_count),
                    }
                    sleep(delay).await;
                }

                info!("Connecting to Central Server at {}...", url);
                
                match connect_central(&url, &agent_id, &token).await {
                    Ok((ws_stream, _)) => {
                        info!("✅ Connected to Central Server.");
                        
                        let (mut write, mut read) = ws_stream.split();

                        // 1. Handshake
                        let handshake_msg = CentralMessage::Handshake {
                            agent_id: agent_id.clone(),
                            token: token.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        
                        if let Ok(text) = serde_json::to_string(&handshake_msg) {
                            if let Err(e) = write.send(Message::Text(text.into())).await {
                                error!("❌ Handshake failed: {}", e);
                                manager.record_connection_failed();
                                continue;
                            }
                            info!("🤝 Handshake successful.");
                        }

                        manager.record_connection_established();

                        // Reset state for stable connection tracking
                        let stable_connection_task = tokio::spawn(async move {
                            sleep(Duration::from_secs(10)).await;
                        });

                        let mut message_received = false;
                        
                        // 2. Main Loop - Select between incoming and outgoing
                        loop {
                            tokio::select! {
                                // Outgoing messages from channel
                                Some(out_msg) = rx.recv() => {
                                    if let Ok(text) = serde_json::to_string(&out_msg) {
                                        if let Err(e) = write.send(Message::Text(text.into())).await {
                                            error!("Failed to send message: {}", e);
                                            break; // Connection probably dropped
                                        }
                                    }
                                }
                                
                                // Incoming messages from WebSocket
                                Some(in_msg) = read.next() => {
                                    match in_msg {
                                        Ok(Message::Text(text)) => {
                                            handle_incoming_text(&text, &db).await;
                                            message_received = true;
                                            // Reset on first valid message is a sign of stability
                                            manager.record_stable();
                                        }
                                        Ok(Message::Close(_)) => {
                                            warn!("🔌 Connection closed by server.");
                                            break;
                                        }
                                        Err(e) => {
                                            error!("⚠️ WebSocket error: {}", e);
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        
                        if stable_connection_task.is_finished() || message_received {
                            info!("✨ Connection was stable. Resetting state.");
                            manager.record_stable();
                        } else {
                            warn!("🔄 Connection dropped prematurely. Entering Backoff mode.");
                            manager.record_dropped_prematurely();
                        }

                        stable_connection_task.abort();
                        info!("🔄 Connection status updated. Preparing next step...");
                    }
                    Err(e) => {
                        error!("❌ Connection failed: {}", e);
                        manager.record_connection_failed();
                    }
                }
            }
        });
    }
}

fn calculate_backoff(retry_count: u32) -> Duration {
    let base_delay = Duration::from_secs(1);
    let max_delay = Duration::from_secs(60);
    let exponent = std::cmp::min(retry_count.saturating_sub(1), 31); // 2^0 = 1s on first backoff retry
    std::cmp::min(base_delay.saturating_mul(2u32.pow(exponent)), max_delay)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_manager_initial_failures() {
        let mut manager = RetryManager::new();
        assert_eq!(manager.get_delay(), Duration::from_secs(0));
        
        manager.record_connection_failed();
        assert_eq!(manager.get_delay(), Duration::from_secs(5));
        assert_eq!(manager.mode, RetryMode::Initial);
        
        manager.record_connection_failed();
        assert_eq!(manager.get_delay(), Duration::from_secs(5));
        assert_eq!(manager.mode, RetryMode::Initial);
    }

    #[test]
    fn test_retry_manager_backoff_after_connect() {
        let mut manager = RetryManager::new();
        
        // Initial success
        manager.record_connection_established();
        
        // Dropped prematurely -> Should go to Backoff Retry 1 (1s)
        manager.record_dropped_prematurely();
        assert_eq!(manager.mode, RetryMode::Backoff);
        assert_eq!(manager.get_delay(), Duration::from_secs(1));
        
        // Connect again, but drop again prematurely -> Should go to Backoff Retry 2 (2s)
        manager.record_connection_established();
        manager.record_dropped_prematurely();
        assert_eq!(manager.get_delay(), Duration::from_secs(2));
        
        // Fail to connect (TCP level) -> Should go to Backoff Retry 3 (4s)
        manager.record_connection_failed();
        assert_eq!(manager.get_delay(), Duration::from_secs(4));
    }

    #[test]
    fn test_retry_manager_reset_after_stable() {
        let mut manager = RetryManager::new();
        manager.record_connection_established();
        manager.record_dropped_prematurely();
        assert_eq!(manager.mode, RetryMode::Backoff);
        
        manager.record_stable();
        assert_eq!(manager.mode, RetryMode::Initial);
        assert_eq!(manager.retry_count, 0);
        assert_eq!(manager.get_delay(), Duration::from_secs(0));
    }

    #[test]
    fn test_backoff_calculation_logic() {
        // retry_count starts at 1 for backoff
        assert_eq!(calculate_backoff(1), Duration::from_secs(1)); // 2^0
        assert_eq!(calculate_backoff(2), Duration::from_secs(2)); // 2^1
        assert_eq!(calculate_backoff(6), Duration::from_secs(32)); // 2^5
        assert_eq!(calculate_backoff(7), Duration::from_secs(60)); // 2^6=64, cap 60
    }

    #[tokio::test]
    async fn test_push_log() {
        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db_path = db_file.path().to_str().unwrap();
        let db = Database::new(&format!("sqlite:{}", db_path)).await.unwrap();
        
        let (client, mut rx) = CentralClient::new("ws://localhost".into(), "agent-1".into(), "token-1".into(), db);
        
        let data = AttendanceData {
            user_id: "1".into(),
            timestamp: "2026-01-01T08:00:00Z".into(),
            status: Some(1),
            punch: Some(1),
        };
        
        client.push_log("device-1".into(), data.clone()).await;
        
        let received = rx.recv().await.unwrap();
        match received {
            CentralMessage::Attendance { agent_id, device_id, data: d, .. } => {
                assert_eq!(agent_id, "agent-1");
                assert_eq!(device_id, "device-1");
                assert_eq!(d.user_id, "1");
            },
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_handle_incoming_text_config_push() {
        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db_path = db_file.path().to_str().unwrap();
        let db = Database::new(&format!("sqlite:{}", db_path)).await.unwrap();
        
        let config_push = serde_json::json!({
            "type": "config_push",
            "data": {
                "devices": [
                    { "id": "dev-1", "ip": "192.168.1.10" }
                ]
            }
        });
        
        handle_incoming_text(&config_push.to_string(), &db).await;
        
        let active = db.get_active_devices().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "dev-1");
        assert_eq!(active[0].ip, "192.168.1.10");
    }

    #[tokio::test]
    async fn test_handle_incoming_text_ack_and_command() {
        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db_path = db_file.path().to_str().unwrap();
        let db = Database::new(&format!("sqlite:{}", db_path)).await.unwrap();
        
        // These just log for now, but we test they don't crash
        handle_incoming_text("{\"type\":\"ack\", \"msg_id\":\"123\"}", &db).await;
        handle_incoming_text("{\"type\":\"command\", \"id\":\"1\", \"method\":\"reboot\"}", &db).await;
        handle_incoming_text("invalid json", &db).await;
    }
}

async fn connect_central(
    url: &str,
    agent_id: &str,
    token: &str,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::handshake::client::Response), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = url.into_client_request()?;
    let headers = request.headers_mut();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert("X-Agent-ID", HeaderValue::from_str(agent_id)?);

    let (ws_stream, response) = connect_async(request).await?;
    Ok((ws_stream, response))
}

async fn handle_incoming_text(text: &str, db: &Database) {
    // Note: text comes as Utf8Bytes, so we might need to treat it as string if from_str expects &str.
    // Tungstenite's Utf8Bytes implements Deref<Target=str>, so it should work fine.
    match serde_json::from_str::<CentralMessage>(text) {
        Ok(msg) => match msg {
            CentralMessage::ConfigPush { data } => {
                info!("Received Config Push: {:?}", data);
                if let Some(devices) = data.get("devices").and_then(|d| d.as_array()) {
                    for d in devices {
                        let id = d.get("device_id").or(d.get("id")).and_then(|v| v.as_str());
                        let ip = d.get("ip").and_then(|v| v.as_str());
                        
                        if let (Some(id), Some(ip)) = (id, ip) {
                            let _ = db.upsert_device(id, None, ip, 4370).await;
                        }
                    }
                }
            }
            CentralMessage::Command { id, method, params: _ } => {
                info!("Received Command: {} {}", method, id);
            }
            CentralMessage::Ack { msg_id } => {
                info!("Received ACK for {}", msg_id);
            }
            _ => {}
        },
        Err(e) => error!("Failed to parse incoming message: {}", e),
    }
}