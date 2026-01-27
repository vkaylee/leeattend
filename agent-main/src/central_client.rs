use std::sync::Arc;
use futures_util::{ SinkExt, StreamExt };
use log::{error, info, warn};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async,
    tungstenite::client::IntoClientRequest,
    tungstenite::http::HeaderValue,
    tungstenite::Message,
    MaybeTlsStream,
    WebSocketStream,
};

use crate::database::Database;
use crate::models::{CentralMessage, AttendanceData};

use std::sync::atomic::{AtomicBool, Ordering};

pub struct CentralClient {
    url: String,
    agent_id: String,
    token: String,
    db: Database,
    tx: mpsc::Sender<CentralMessage>,
    reload_tx: mpsc::Sender<()>,
    is_connected: Arc<AtomicBool>,
}

#[allow(clippy::collapsible_if)]
impl CentralClient {
    pub fn new(url: String, agent_id: String, token: String, db: Database) -> (Self, mpsc::Receiver<CentralMessage>, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel(100);
        let (reload_tx, reload_rx) = mpsc::channel(1);
        let is_connected = Arc::new(AtomicBool::new(false));
        (Self { url, agent_id, token, db, tx, reload_tx, is_connected }, rx, reload_rx)
    }

    pub async fn start(self, mut rx: mpsc::Receiver<CentralMessage>) {
        let url = self.url.clone();
        let agent_id = self.agent_id.clone();
        let token = self.token.clone();
        let tx = self.tx.clone();
        let db = self.db.clone();
        let ws_reload_tx = self.reload_tx.clone();

        // 1. Sync Loop
        let sync_db = db.clone();
        let sync_tx = tx.clone();
        let sync_agent_id = agent_id.clone();
        let connected = self.is_connected.clone();
        tokio::spawn(async move {
            loop {
                if !connected.load(Ordering::Relaxed) {
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
                let limit = 500;
                match sync_db.get_unsynced_logs(limit).await {
                    Ok(logs) => {
                        let count = logs.len();
                        for log in logs {
                            let msg = CentralMessage::Attendance {
                                msg_id: log.id.to_string(),
                                agent_id: sync_agent_id.clone(),
                                device_id: log.device_id.clone(),
                                data: AttendanceData {
                                    user_id: log.user_id.clone(),
                                    timestamp: log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                                    status: log.status,
                                    punch: log.punch,
                                },
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            if sync_tx.send(msg).await.is_err() { break; }
                        }
                        if count < limit as usize { sleep(Duration::from_secs(10)).await; }
                        else { sleep(Duration::from_millis(50)).await; }
                    }
                    Err(_) => { sleep(Duration::from_secs(10)).await; }
                }
            }
        });

        // 2. Heartbeat Task
        let hb_agent_id = agent_id.clone();
        let hb_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                let hb = CentralMessage::Heartbeat {
                    agent_id: hb_agent_id.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                let _ = hb_tx.send(hb).await;
            }
        });

        // 3. WS Connection Manager
        let conn_reload_tx = ws_reload_tx.clone();
        let ws_connected = self.is_connected.clone();
        tokio::spawn(async move {
            let mut manager = RetryManager::new();
            loop {
                let delay = manager.get_delay();
                if delay > Duration::from_secs(0) { sleep(delay).await; }

                info!("Connecting to Central Server at {}...", url);
                match connect_central(&url, &agent_id, &token).await {
                    Ok((ws_stream, _)) => {
                        info!("✅ Connected to Central Server.");
                        ws_connected.store(true, Ordering::Relaxed);
                        manager.record_connection_established();
                        let (mut write, mut read) = ws_stream.split();

                        let handshake = CentralMessage::Handshake {
                            agent_id: agent_id.clone(),
                            token: token.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        if let Ok(text) = serde_json::to_string(&handshake) {
                            if write.send(Message::Text(text.into())).await.is_err() {
                                ws_connected.store(false, Ordering::Relaxed);
                                manager.record_connection_failed();
                                continue;
                            }
                        }

                        loop {
                            tokio::select! {
                                Some(out_msg) = rx.recv() => {
                                    if let Ok(text) = serde_json::to_string(&out_msg) {
                                        if write.send(Message::Text(text.into())).await.is_err() { break; }
                                    }
                                }
                                in_msg = read.next() => {
                                    match in_msg {
                                        Some(Ok(Message::Text(text))) => {
                                            handle_incoming_text(&text, &db, &conn_reload_tx).await;
                                            manager.record_stable();
                                        }
                                        Some(Ok(Message::Ping(p))) => {
                                            if write.send(Message::Pong(p)).await.is_err() { break; }
                                        }
                                        Some(Ok(Message::Close(f))) => {
                                            warn!("🚪 Server closed connection: {:?}", f);
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            error!("❌ WS Error: {}", e);
                                            break;
                                        }
                                        None => {
                                            warn!("🔌 WS stream ended.");
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        ws_connected.store(false, Ordering::Relaxed);
                        manager.record_dropped_prematurely();
                    }
                    Err(e) => {
                        error!("❌ Connection failed: {}", e);
                        ws_connected.store(false, Ordering::Relaxed);
                        manager.record_connection_failed();
                    }
                }
            }
        });
    }
}

pub async fn handle_incoming_text(text: &str, db: &Database, reload_tx: &mpsc::Sender<()>) {
    if let Ok(msg) = serde_json::from_str::<CentralMessage>(text) {
        match msg {
            CentralMessage::ConfigPush { data } => {
                info!("Received ConfigPush with {} devices", data.devices.len());
                if let Some(mw) = data.max_concurrent_workers {
                    let _ = db.upsert_agent_config("max_concurrent_workers", &mw.to_string()).await;
                }
                for d in data.devices {
                    let _ = db.upsert_device(&d.device_id, &d.name, &d.ip, d.port, &d.brand).await;
                }
                let _ = reload_tx.try_send(());
            }
            CentralMessage::Ack { msg_id, status, .. } => {
                if let Ok(id) = msg_id.parse::<i64>() {
                    if status.as_deref() == Some("success") {
                        let _ = db.mark_synced(&[id]).await;
                    } else {
                        let _ = sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
                            .bind(id)
                            .execute(db.get_pool())
                            .await;
                    }
                }
            }
            CentralMessage::Error { code, message } => {
                error!("⚠️ Server error [{}]: {}", code, message);
            }
            _ => {}
        }
    }
}

async fn connect_central(url: &str, agent_id: &str, token: &str) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::handshake::client::Response), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = url.into_client_request()?;
    request.headers_mut().insert("X-Agent-Id", HeaderValue::from_str(agent_id)?);
    request.headers_mut().insert("X-Token", HeaderValue::from_str(token)?);
    let (ws_stream, response) = connect_async(request).await?;
    Ok((ws_stream, response))
}

struct RetryManager {
    retry_count: u32,
}

impl RetryManager {
    fn new() -> Self { Self { retry_count: 0 } }
    fn get_delay(&self) -> Duration {
        if self.retry_count == 0 { return Duration::from_secs(0); }
        let base = 1u64;
        let delay = std::cmp::min(base * 2u64.pow(std::cmp::min(self.retry_count - 1, 31)), 60);
        Duration::from_secs(delay)
    }
    fn record_connection_established(&mut self) { self.retry_count = 0; }
    fn record_connection_failed(&mut self) { self.retry_count = std::cmp::min(self.retry_count + 1, 10); }
    fn record_dropped_prematurely(&mut self) { self.record_connection_failed(); }
    fn record_stable(&mut self) { self.retry_count = 0; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_handle_incoming_error() {
        let db = Database::new(":memory:").await.unwrap();
        let (reload_tx, _reload_rx) = mpsc::channel(1);

        let error_msg = r#"{"type":"error", "code":"INVALID_TOKEN", "message":"Invalid token provided"}"#;
        handle_incoming_text(error_msg, &db, &reload_tx).await;
    }

    #[tokio::test]
    async fn test_handle_incoming_ack_failure() {
        let db = Database::new(":memory:").await.unwrap();
        let (reload_tx, _reload_rx) = mpsc::channel(1);

        sqlx::query("INSERT INTO zkt_raw_attendance (id, device_id, user_id, timestamp, is_synced) VALUES (999, 'dev1', 'user1', '2026-01-27 10:00:00', 2)")
            .execute(db.get_pool())
            .await
            .unwrap();

        let ack_error = r#"{"type":"ack", "msg_id":"999", "status":"error", "error":"Server overloaded"}"#;
        handle_incoming_text(ack_error, &db, &reload_tx).await;

        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, 999);
        assert_eq!(logs[0].is_synced, 0);
    }
}
