use std::sync::Arc;
use std::time::{Duration, Instant};
use futures_util::{ SinkExt, StreamExt };
use log::{debug, error, info, warn};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::client::IntoClientRequest,
    tungstenite::http::HeaderValue,
    tungstenite::Message,
    MaybeTlsStream,
    WebSocketStream,
};

use crate::models::{CentralMessage, AttendanceData};
use crate::hybrid_database::HybridDatabase;

use std::sync::atomic::{AtomicBool, Ordering};

use super::message_handler::handle_incoming_text;
use super::retry_manager::{RetryManager, calculate_sync_stats};

// Retry configuration constants
pub(super) const CONNECTION_TIMEOUT_SECS: u64 = 30;
#[allow(dead_code)]
pub(super) const STABILITY_THRESHOLD_SECS: u64 = 60;
pub(super) const MAX_RETRY_DELAY_SECS: u64 = 60;
pub(super) const MAX_RETRY_COUNT: u32 = 10;

pub struct CentralClient {
    url: String,
    agent_id: String,
    token: String,
    hybrid_db: HybridDatabase,
    inflight_timeout_secs: u64,
    tx: mpsc::Sender<CentralMessage>,
    reload_tx: mpsc::Sender<()>,
    is_connected: Arc<AtomicBool>,
}

#[allow(clippy::collapsible_if)]
impl CentralClient {
    pub fn new(url: String, agent_id: String, token: String, hybrid_db: HybridDatabase, inflight_timeout_secs: u64) -> (Self, mpsc::Receiver<CentralMessage>, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel(100);
        let (reload_tx, reload_rx) = mpsc::channel(1);
        let is_connected = Arc::new(AtomicBool::new(false));
        (Self { url, agent_id, token, hybrid_db, inflight_timeout_secs, tx, reload_tx, is_connected }, rx, reload_rx)
    }

    pub async fn start(self, rx: mpsc::Receiver<CentralMessage>) {
        let url = self.url.clone();
        let agent_id = self.agent_id.clone();
        let token = self.token.clone();
        let tx = self.tx.clone();
        let hybrid_db = self.hybrid_db.clone();
        let ws_reload_tx = self.reload_tx.clone();

        // 1. Sync Loop
        let sync_hybrid = hybrid_db.clone();
        let sync_tx = tx.clone();
        let sync_agent_id = agent_id.clone();
        let connected = self.is_connected.clone();
        let inflight_timeout_secs = self.inflight_timeout_secs;
        tokio::spawn(async move {
            let mut last_stale_check = Instant::now();

            loop {
                if !connected.load(Ordering::Relaxed) {
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }

                // Block syncing until valid device config is received
                let device_count = match sync_hybrid.disk_db().get_active_devices().await {
                    Ok(devices) => devices.len(),
                    Err(_) => 0,
                };
                if device_count == 0 {
                    warn!("🔴 Sync loop waiting - no active devices configured");
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }

                // Periodic cleanup: Move expired logs to DLQ
                let max_retries = match sync_hybrid.disk_db().get_agent_config("max_retry_count").await {
                    Ok(Some(val)) => val.parse::<i64>().unwrap_or(10),
                    _ => 10, // Default: 10 retries
                };
                let ttl_minutes = match sync_hybrid.disk_db().get_agent_config("retry_ttl_minutes").await {
                    Ok(Some(val)) => val.parse::<i64>().unwrap_or(60),
                    _ => 60, // Default: 1 hour
                };

                if let Ok(moved) = sync_hybrid.disk_db().move_expired_logs_to_dlq(max_retries, ttl_minutes).await {
                    if moved > 0 {
                        warn!("⚠️ Moved {moved} expired logs to dead letter queue (max_retries={max_retries}, ttl={ttl_minutes}m)");
                    }
                }

                // Periodic stale in-flight reset (every 60 seconds)
                if last_stale_check.elapsed() >= Duration::from_secs(60) {
                    let reset_count = sync_hybrid.memory_store().reset_stale_inflight(Duration::from_secs(inflight_timeout_secs));
                    if reset_count > 0 {
                        warn!("⚠️ Reset {reset_count} stale in-flight logs (>5min)");
                    }
                    last_stale_check = Instant::now();
                }

                // Read from memory
                let limit = 500;
                let start = Instant::now();
                let logs = sync_hybrid.get_unsynced_logs(limit);
                let count = logs.len();

                if count > 0 {
                    // Classify logs: new (updated_at IS NULL) vs retry (updated_at IS NOT NULL)


                    // Calculate stats for better visibility
                    let stats_str = calculate_sync_stats(&logs);

                    let total_pending = sync_hybrid.count_unsynced_logs();

                    info!("📤 Syncing {count} logs to central server ({stats_str} | Pending: {total_pending})...");

                    // Mark in-flight in memory
                    let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
                    sync_hybrid.mark_in_flight(&log_ids);

                    let mem_latency = start.elapsed();
                    if mem_latency.as_millis() > 5 {
                        warn!("⚠️ Sync cycle memory latency: {mem_latency:?} (target: <5ms)");
                    }
                }
                for log in logs {
                    // Convert Unix timestamp to formatted string
                    let timestamp_str = chrono::DateTime::from_timestamp(log.timestamp, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| format!("invalid_{}", log.timestamp));

                    let msg = CentralMessage::Attendance {
                        msg_id: log.composite_key(),
                        agent_id: sync_agent_id.clone(),
                        device_id: log.device_id.to_string(),
                        data: AttendanceData {
                            user_id: crate::utils::normalize_employee_id(&log.user_id),
                            timestamp: timestamp_str,
                            status: log.status,
                            punch: log.punch,
                        },
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    if sync_tx.send(msg).await.is_err() {
                        error!("❌ Failed to send log to sync channel");
                        break;
                    }
                }

                // Adaptive sleep based on queue size
                let unsynced_count = sync_hybrid.count_unsynced_logs();

                // Log backoff status for monitoring
                let backoff_count = sync_hybrid.memory_store().count_in_backoff();
                if backoff_count > 0 {
                    debug!("⏳ {} logs in backoff, {} eligible for sync", backoff_count, unsynced_count - backoff_count);
                }

                let sleep_duration = if count == 0 {
                    Duration::from_millis(500)  // Idle: check every 500ms
                } else if unsynced_count > 1000 {
                    Duration::from_millis(50)   // Backlogged: drain fast
                } else if unsynced_count > 100 {
                    Duration::from_millis(100)  // Moderate: drain
                } else {
                    Duration::from_millis(200)  // Normal: balanced
                };

                sleep(sleep_duration).await;
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
            let mut rx_opt: Option<mpsc::Receiver<CentralMessage>> = Some(rx);

            loop {
                let delay = manager.get_delay();
                if delay > Duration::from_secs(0) { sleep(delay).await; }

                // Take the receiver; if unavailable, connection is unrecoverable
                let rx = match rx_opt.take() {
                    Some(r) => r,
                    None => {
                        error!("❌ Lost message receiver, cannot reconnect");
                        break;
                    }
                };

                info!("Connecting to Central Server at {url}...");
                match connect_central(&url, &agent_id, &token).await {
                    Ok((ws_stream, _)) => {
                        info!("✅ Connected to Central Server.");
                        ws_connected.store(true, Ordering::Relaxed);
                        manager.record_connection_established();
                        let (mut write, read) = ws_stream.split();

                        let handshake = CentralMessage::Handshake {
                            agent_id: agent_id.clone(),
                            token: token.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        if let Ok(text) = serde_json::to_string(&handshake) {
                            if write.send(Message::Text(text.into())).await.is_err() {
                                ws_connected.store(false, Ordering::Relaxed);
                                manager.record_connection_failed();
                                rx_opt = Some(rx);
                                continue;
                            }
                        }

                        // Split into independent read/write tasks to prevent
                        // outgoing messages from starving incoming ACK processing.
                        // Use a shutdown signal so write_task can gracefully return rx.
                        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

                        let read_hybrid = hybrid_db.clone();
                        let read_reload_tx = conn_reload_tx.clone();
                        let mut read_task = tokio::spawn(async move {
                            let mut read = read;
                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        handle_incoming_text(&text, &read_hybrid, &read_reload_tx).await;
                                    }
                                    Ok(Message::Ping(_)) => {
                                        // Pong is handled automatically by tungstenite
                                    }
                                    Ok(Message::Close(f)) => {
                                        warn!("🚪 Server closed connection: {f:?}");
                                        break;
                                    }
                                    Err(e) => {
                                        error!("❌ WS Read Error: {e}");
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        });

                        let mut write_task = tokio::spawn(async move {
                            #[allow(unused_mut)]
                            let mut rx = rx;
                            let mut shutdown_rx = shutdown_rx;
                            loop {
                                tokio::select! {
                                    msg = rx.recv() => {
                                        match msg {
                                            Some(out_msg) => {
                                                if let Ok(text) = serde_json::to_string(&out_msg) {
                                                    if write.send(Message::Text(text.into())).await.is_err() {
                                                        break;
                                                    }
                                                }
                                            }
                                            None => break,
                                        }
                                    }
                                    _ = &mut shutdown_rx => {
                                        break; // Graceful shutdown, rx preserved
                                    }
                                }
                            }
                            rx // Return receiver for reuse on reconnect
                        });

                        // Wait for either task to finish (connection broken)
                        tokio::select! {
                            _ = &mut read_task => {
                                warn!("🔌 WS read loop ended.");
                                let _ = shutdown_tx.send(()); // Signal write_task to stop gracefully
                                if let Ok(returned_rx) = write_task.await {
                                    rx_opt = Some(returned_rx);
                                }
                            }
                            result = &mut write_task => {
                                warn!("🔌 WS write loop ended.");
                                read_task.abort();
                                if let Ok(returned_rx) = result {
                                    rx_opt = Some(returned_rx);
                                }
                            }
                        }
                        ws_connected.store(false, Ordering::Relaxed);
                        manager.record_dropped_prematurely();
                    }
                    Err(e) => {
                        error!("❌ Connection failed: {e}");
                        ws_connected.store(false, Ordering::Relaxed);
                        manager.record_connection_failed();
                        rx_opt = Some(rx);
                    }
                }
            }
        });
    }
}

async fn connect_central(url: &str, agent_id: &str, token: &str) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::handshake::client::Response), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = url.into_client_request()?;
    request.headers_mut().insert("X-Agent-Id", HeaderValue::from_str(agent_id)?);
    request.headers_mut().insert("X-Token", HeaderValue::from_str(token)?);

    // Apply connection timeout to prevent indefinite hangs
    let connect_future = connect_async(request);
    let (ws_stream, response) = timeout(
        Duration::from_secs(CONNECTION_TIMEOUT_SECS),
        connect_future
    ).await
        .map_err(|_| format!("Connection timeout after {CONNECTION_TIMEOUT_SECS} seconds"))?
        .map_err(|e| format!("WebSocket connection error: {e}"))?;

    Ok((ws_stream, response))
}
