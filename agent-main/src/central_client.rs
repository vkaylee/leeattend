use futures_util::{ SinkExt, StreamExt };
use log::{error, info};
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

pub struct CentralClient {
    url: String,
    agent_id: String,
    token: String,
    db: Database,
    tx: mpsc::Sender<CentralMessage>,
}

#[allow(clippy::collapsible_if)]
impl CentralClient {
    pub fn new(url: String, agent_id: String, token: String, db: Database) -> (Self, mpsc::Receiver<CentralMessage>) {
        let (tx, rx) = mpsc::channel(100);
        (Self { url, agent_id, token, db, tx }, rx)
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

    fn record_connection_established(&mut self) {}

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

#[allow(clippy::collapsible_if)]
impl CentralClient {
    pub async fn start(&self, mut rx: mpsc::Receiver<CentralMessage>) {
        let url = self.url.clone();
        let agent_id = self.agent_id.clone();
        let token = self.token.clone();
        let tx = self.tx.clone();
        let db = self.db.clone();

        // 1. Sync Loop - periodic poll unsynced logs from DB
        let sync_db = db.clone();
        let sync_tx = tx.clone();
        let sync_agent_id = agent_id.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(logs) = sync_db.get_unsynced_logs(50).await {
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
                        let _ = sync_tx.send(msg).await;
                    }
                }
                sleep(Duration::from_secs(10)).await;
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
        tokio::spawn(async move {
            let mut manager = RetryManager::new();
            loop {
                let delay = manager.get_delay();
                if delay > Duration::from_secs(0) {
                    sleep(delay).await;
                }

                info!("Connecting to Central Server at {}...", url);
                match connect_central(&url, &agent_id, &token).await {
                    Ok((ws_stream, _)) => {
                        info!("✅ Connected to Central Server.");
                        manager.record_connection_established();
                        let (mut write, mut read) = ws_stream.split();

                        // Handshake
                        let handshake = CentralMessage::Handshake {
                            agent_id: agent_id.clone(),
                            token: token.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        if let Ok(text) = serde_json::to_string(&handshake) {
                            if write.send(Message::Text(text.into())).await.is_err() {
                                manager.record_connection_failed();
                                continue;
                            }
                        }

                        loop {
                            tokio::select! {
                                Some(out_msg) = rx.recv() => {
                                    if let Ok(text) = serde_json::to_string(&out_msg) {
                                        if write.send(Message::Text(text.into())).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                Some(in_msg) = read.next() => {
                                    match in_msg {
                                        Ok(Message::Text(text)) => {
                                            handle_incoming_text(&text, &db).await;
                                            manager.record_stable();
                                        }
                                        _ => break,
                                    }
                                }
                            }
                        }
                        manager.record_dropped_prematurely();
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
    let exponent = std::cmp::min(retry_count.saturating_sub(1), 31);
    std::cmp::min(base_delay.saturating_mul(2u32.pow(exponent)), max_delay)
}

async fn connect_central(url: &str, agent_id: &str, token: &str) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::handshake::client::Response), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = url.into_client_request()?;
    let headers = request.headers_mut();
    headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", token))?);
    headers.insert("X-Agent-ID", HeaderValue::from_str(agent_id)?);
    let (ws_stream, response) = connect_async(request).await?;
    Ok((ws_stream, response))
}

async fn handle_incoming_text(text: &str, db: &Database) {
    if let Ok(msg) = serde_json::from_str::<CentralMessage>(text) {
        match msg {
            CentralMessage::ConfigPush { data } => {
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
            CentralMessage::Ack { msg_id } => {
                if let Ok(id) = msg_id.parse::<i64>() {
                    let _ = db.mark_synced(&[id]).await;
                }
            }
            _ => {}
        }
    }
}
