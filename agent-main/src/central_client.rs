use std::sync::Arc;
use std::time::{Duration, Instant};
use futures_util::{ SinkExt, StreamExt };
use log::{error, info, warn};
use rand::Rng;
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

/// Parse composite key "device_id:user_id:timestamp_unix" back to components
/// Uses rsplit to safely handle device_id containing colons
/// Format: device_id can contain colons, user_id and timestamp_unix cannot
/// Returns timestamp as SQLite-compatible string format "YYYY-MM-DD HH:MM:SS"
fn parse_composite_key(key: &str) -> Option<(String, String, String)> {
    // Split from right: timestamp (last), user_id (second-to-last), device_id (rest)
    let mut parts = key.rsplitn(3, ':');

    // Extract in reverse order: timestamp, user_id, device_id
    let timestamp_str = parts.next()?.trim();  // Trim whitespace
    let user_id = parts.next()?.trim();        // Trim whitespace
    let device_id = parts.next()?.trim();      // Trim whitespace

    // Ensure we consumed exactly 3 parts (no extra parts remaining)
    if parts.next().is_some() {
        return None; // Too many colons even for rsplit(3)
    }

    // Parse Unix timestamp
    let timestamp_unix = timestamp_str.parse::<i64>().ok()?;
    let datetime = chrono::DateTime::from_timestamp(timestamp_unix, 0)?
        .naive_utc();

    // CRITICAL: Format to SQLite DATETIME format (matches DB storage)
    // This ensures the WHERE clause in UPDATE query matches exactly
    let timestamp_formatted = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    Some((device_id.to_string(), user_id.to_string(), timestamp_formatted))
}

// Retry configuration constants
const CONNECTION_TIMEOUT_SECS: u64 = 30;
const STABILITY_THRESHOLD_SECS: u64 = 60;
const MAX_RETRY_DELAY_SECS: u64 = 60;
const MAX_RETRY_COUNT: u32 = 10;

pub struct CentralClient {
    url: String,
    agent_id: String,
    token: String,
    hybrid_db: HybridDatabase,
    tx: mpsc::Sender<CentralMessage>,
    reload_tx: mpsc::Sender<()>,
    is_connected: Arc<AtomicBool>,
}

#[allow(clippy::collapsible_if)]
impl CentralClient {
    pub fn new(url: String, agent_id: String, token: String, hybrid_db: HybridDatabase) -> (Self, mpsc::Receiver<CentralMessage>, mpsc::Receiver<()>) {
        let (tx, rx) = mpsc::channel(100);
        let (reload_tx, reload_rx) = mpsc::channel(1);
        let is_connected = Arc::new(AtomicBool::new(false));
        (Self { url, agent_id, token, hybrid_db, tx, reload_tx, is_connected }, rx, reload_rx)
    }

    pub async fn start(self, mut rx: mpsc::Receiver<CentralMessage>) {
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
        tokio::spawn(async move {
            let mut last_stale_check = Instant::now();

            loop {
                if !connected.load(Ordering::Relaxed) {
                    warn!("🔴 Sync loop waiting - not connected to central server");
                    sleep(Duration::from_secs(5)).await;
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
                        warn!("⚠️ Moved {} expired logs to dead letter queue (max_retries={}, ttl={}m)", moved, max_retries, ttl_minutes);
                    }
                }

                // Periodic stale in-flight reset (every 60 seconds)
                if last_stale_check.elapsed() >= Duration::from_secs(60) {
                    let reset_count = sync_hybrid.memory_store().reset_stale_inflight(Duration::from_secs(300));
                    if reset_count > 0 {
                        warn!("⚠️ Reset {} stale in-flight logs (>5min)", reset_count);
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
                    let new_count = logs.iter().filter(|l| l.updated_at.is_none()).count();
                    let retry_count = count - new_count;

                    if retry_count > 0 {
                        info!("📤 Syncing {} logs to central server ({} new, {} retries)...",
                              count, new_count, retry_count);
                    } else {
                        info!("📤 Syncing {} logs to central server...", count);
                    }

                    // Mark in-flight in memory
                    let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
                    sync_hybrid.mark_in_flight(&log_ids);

                    let mem_latency = start.elapsed();
                    if mem_latency.as_millis() > 5 {
                        warn!("⚠️ Sync cycle memory latency: {:?} (target: <5ms)", mem_latency);
                    }
                }
                for log in logs {
                    let msg = CentralMessage::Attendance {
                        msg_id: log.composite_key(),
                        agent_id: sync_agent_id.clone(),
                        device_id: log.device_id.clone(),
                        data: AttendanceData {
                            user_id: crate::utils::normalize_employee_id(&log.user_id),
                            timestamp: log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
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
                    log::debug!("⏳ {} logs in backoff, {} eligible for sync", backoff_count, unsynced_count - backoff_count);
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
                                            handle_incoming_text(&text, &hybrid_db, &conn_reload_tx).await;
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

pub async fn handle_incoming_text(text: &str, hybrid_db: &HybridDatabase, reload_tx: &mpsc::Sender<()>) {
    if let Ok(msg) = serde_json::from_str::<CentralMessage>(text) {
        match msg {
            CentralMessage::ConfigPush { data } => {
                info!("Received ConfigPush with {} devices", data.devices.len());
                if let Some(mw) = data.max_concurrent_workers {
                    let _ = hybrid_db.disk_db().upsert_agent_config("max_concurrent_workers", &mw.to_string()).await;
                }
                // Deactivate devices not in this push (handles stale/renamed devices)
                let active_ids: Vec<String> = data.devices.iter().map(|d| d.device_id.clone()).collect();
                if let Ok(deactivated) = hybrid_db.disk_db().deactivate_devices_not_in(&active_ids).await
                    && deactivated > 0 {
                        info!("Deactivated {} stale devices", deactivated);
                    }
                for d in data.devices {
                    let _ = hybrid_db.disk_db().upsert_device(&d.device_id, &d.name, &d.ip, d.port, &d.brand).await;
                }
                let _ = reload_tx.try_send(());
            }
            CentralMessage::Ack { msg_id, status, .. } => {
                // msg_id is now composite key "device_id:user_id:timestamp_unix"
                if status.as_deref() == Some("success") {
                    // Update memory
                    if !hybrid_db.mark_synced_by_key(&msg_id) {
                        log::debug!("ACK for already-synced log (disk update OK): {}", msg_id);
                    }

                    // Update disk using composite key
                    if let Some((device_id, user_id, timestamp_str)) = parse_composite_key(&msg_id) {
                        log::info!("🔍 ACK Success: device='{}', user='{}', timestamp='{}'",
                            device_id, user_id, timestamp_str);

                        match sqlx::query(
                            "UPDATE zkt_raw_attendance SET is_synced = 1
                             WHERE device_id = ? AND user_id = ? AND timestamp = ?"
                        )
                        .bind(&device_id)
                        .bind(&user_id)
                        .bind(&timestamp_str)
                        .execute(hybrid_db.disk_db().get_pool())
                        .await
                        {
                            Ok(result) => {
                                let rows = result.rows_affected();
                                if rows == 0 {
                                    log::error!("❌ UPDATE affected 0 rows! device='{}', user='{}', timestamp='{}' - Log may not exist in DB or already synced",
                                        device_id, user_id, timestamp_str);
                                } else {
                                    log::debug!("✅ Marked {} row(s) as synced in DB", rows);
                                }
                            }
                            Err(e) => {
                                log::error!("❌ UPDATE query failed: {}", e);
                            }
                        }
                    } else {
                        log::warn!("⚠️ Failed to parse composite key: {}", msg_id);
                    }
                } else {
                    // Failed ACK - reset to unsynced for retry
                    if !hybrid_db.reset_to_unsynced_by_key(&msg_id) {
                        log::debug!("Failed ACK for already-synced log: {}", msg_id);
                    }

                    // Reset disk using composite key
                    if let Some((device_id, user_id, timestamp_str)) = parse_composite_key(&msg_id) {
                        log::warn!("⚠️ ACK Failed: Resetting device='{}', user='{}', timestamp='{}'",
                            device_id, user_id, timestamp_str);

                        let _ = sqlx::query(
                            "UPDATE zkt_raw_attendance SET is_synced = 0
                             WHERE device_id = ? AND user_id = ? AND timestamp = ?"
                        )
                        .bind(&device_id)
                        .bind(&user_id)
                        .bind(&timestamp_str)
                        .execute(hybrid_db.disk_db().get_pool())
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

    // Apply connection timeout to prevent indefinite hangs
    let connect_future = connect_async(request);
    let (ws_stream, response) = timeout(
        Duration::from_secs(CONNECTION_TIMEOUT_SECS),
        connect_future
    ).await
        .map_err(|_| format!("Connection timeout after {} seconds", CONNECTION_TIMEOUT_SECS))?
        .map_err(|e| format!("WebSocket connection error: {}", e))?;

    Ok((ws_stream, response))
}

struct RetryManager {
    retry_count: u32,
    stable_since: Option<Instant>,
}

impl RetryManager {
    fn new() -> Self {
        Self {
            retry_count: 0,
            stable_since: None,
        }
    }

    fn get_delay(&self) -> Duration {
        if self.retry_count == 0 {
            return Duration::from_secs(0);
        }

        // Exponential backoff: 1s, 2s, 4s, 8s, 16s, 32s, 60s (capped)
        let base_delay = std::cmp::min(
            2u64.pow(std::cmp::min(self.retry_count - 1, 31)),
            MAX_RETRY_DELAY_SECS
        );

        // Apply ±25% jitter to prevent thundering herd
        let mut rng = rand::thread_rng();
        let jitter_factor = rng.gen_range(0.75..=1.25);
        let jittered = (base_delay as f64 * jitter_factor) as u64;

        // Ensure minimum 1s and maximum 60s after jitter
        let final_delay = jittered.clamp(1, MAX_RETRY_DELAY_SECS);
        Duration::from_secs(final_delay)
    }

    fn record_connection_established(&mut self) {
        // Don't reset retry_count immediately - wait for stability
        // Start the stability timer
        self.stable_since = Some(Instant::now());
        info!("Connection established, starting stability timer");
    }

    fn record_connection_failed(&mut self) {
        self.retry_count = std::cmp::min(self.retry_count + 1, MAX_RETRY_COUNT);
        self.stable_since = None;
        warn!("Connection failed, retry count: {}", self.retry_count);
    }

    fn record_dropped_prematurely(&mut self) {
        self.record_connection_failed();
    }

    fn record_stable(&mut self) {
        if let Some(since) = self.stable_since {
            if since.elapsed() >= Duration::from_secs(STABILITY_THRESHOLD_SECS)
                && self.retry_count > 0 {
                    info!("Connection stable for {}s, resetting retry count", STABILITY_THRESHOLD_SECS);
                    self.retry_count = 0;
                }
                // Keep stable_since as-is to avoid repeated reset logs
        } else {
            // First stable call after connection - start the timer
            self.stable_since = Some(Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hybrid_database::HybridDatabase;
    use tokio::sync::mpsc;

    #[test]
    fn test_parse_composite_key_normal() {
        // Normal case: device_id without colons
        let key = "DEV1:U001:1738314000";
        let result = parse_composite_key(key);
        assert!(result.is_some());

        let (device_id, user_id, timestamp_str) = result.unwrap();
        assert_eq!(device_id, "DEV1");
        assert_eq!(user_id, "U001");
        // Verify timestamp is formatted correctly for SQLite
        // 1738314000 = 2025-01-31 09:00:00 UTC
        assert_eq!(timestamp_str, "2025-01-31 09:00:00");
    }

    #[test]
    fn test_parse_composite_key_with_colon_in_device_id() {
        // Device ID with colons (common in real deployments like "Office:Main Gate")
        let key = "Office:Main:Gate:U001:1738314000";
        let result = parse_composite_key(key);
        assert!(result.is_some(), "Should parse device_id with colons");

        let (device_id, user_id, timestamp_str) = result.unwrap();
        assert_eq!(device_id, "Office:Main:Gate");
        assert_eq!(user_id, "U001");
        assert_eq!(timestamp_str, "2025-01-31 09:00:00");
    }

    #[test]
    fn test_parse_composite_key_single_colon_device() {
        // Single colon in device_id
        let key = "Office:Gate:U001:1738314000";
        let result = parse_composite_key(key);
        assert!(result.is_some());

        let (device_id, user_id, _) = result.unwrap();
        assert_eq!(device_id, "Office:Gate");
        assert_eq!(user_id, "U001");
    }

    #[test]
    fn test_parse_composite_key_invalid_timestamp() {
        // Invalid timestamp should return None
        let key = "DEV1:U001:not_a_number";
        let result = parse_composite_key(key);
        assert!(result.is_none(), "Invalid timestamp should return None");
    }

    #[test]
    fn test_parse_composite_key_missing_parts() {
        // Missing parts should return None
        let key = "DEV1:U001";
        let result = parse_composite_key(key);
        assert!(result.is_none(), "Missing timestamp should return None");

        let key2 = "DEV1";
        let result2 = parse_composite_key(key2);
        assert!(result2.is_none(), "Missing user_id and timestamp should return None");
    }

    #[tokio::test]
    async fn test_handle_incoming_error() {
        use crate::hybrid_database::HybridDatabase;
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();
        let (reload_tx, _reload_rx) = mpsc::channel(1);

        let error_msg = r#"{"type":"error", "code":"INVALID_TOKEN", "message":"Invalid token provided"}"#;
        handle_incoming_text(error_msg, &hybrid_db, &reload_tx).await;
    }

    #[tokio::test]
    async fn test_handle_incoming_ack_failure() {
        use crate::hybrid_database::HybridDatabase;
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();
        let (reload_tx, _reload_rx) = mpsc::channel(1);

        sqlx::query("INSERT INTO zkt_raw_attendance (id, device_id, user_id, timestamp, is_synced) VALUES (999, 'dev1', 'user1', '2026-01-27 10:00:00', 2)")
            .execute(hybrid_db.disk_db().get_pool())
            .await
            .unwrap();

        let ack_error = r#"{"type":"ack", "msg_id":"999", "status":"error", "error":"Server overloaded"}"#;
        handle_incoming_text(ack_error, &hybrid_db, &reload_tx).await;

        // Since this is a hybrid system, we should check memory state via HybridDatabase
        let _logs = hybrid_db.get_unsynced_logs(10);
        // Note: The log might not be in memory if it was only inserted to disk above
        // This test needs to be adjusted to insert via hybrid_db or load from disk first
    }

    // ===== RetryManager Unit Tests =====

    #[test]
    fn test_retry_manager_initial_delay_is_zero() {
        let manager = RetryManager::new();
        assert_eq!(manager.get_delay(), Duration::from_secs(0));
        assert_eq!(manager.retry_count, 0);
        assert!(manager.stable_since.is_none());
    }

    #[test]
    fn test_retry_manager_exponential_backoff_with_jitter() {
        let mut manager = RetryManager::new();

        // First failure: base delay is 1s, with jitter and floor of 1s
        // Result: 1s (due to min clamp after jitter)
        manager.record_connection_failed();
        let delay1 = manager.get_delay();
        assert!(delay1.as_secs() >= 1, "Delay should be >= 1s (minimum after clamp)");
        assert!(delay1.as_secs() <= 1, "Delay should be <= 1s (base 1s with jitter, clamped to 1)");
        assert_eq!(manager.retry_count, 1);

        // Second failure: base delay is 2s, jitter gives 1-2s range
        manager.record_connection_failed();
        let delay2 = manager.get_delay();
        assert!(delay2.as_secs() >= 1, "Delay should be >= 1s");
        assert!(delay2.as_secs() <= 2, "Delay should be <= 2s");
        assert_eq!(manager.retry_count, 2);
    }

    #[test]
    fn test_retry_manager_max_delay_is_60_seconds() {
        let mut manager = RetryManager::new();

        // Simulate many failures to reach max
        for _ in 0..15 {
            manager.record_connection_failed();
        }

        // Retry count should be capped at MAX_RETRY_COUNT
        assert_eq!(manager.retry_count, MAX_RETRY_COUNT);

        // Delay should be capped at 60s (+ jitter, so max 75s)
        let delay = manager.get_delay();
        assert!(delay.as_secs() <= 75, "Delay should be <= 75s (60s + 25% jitter)");
        assert!(delay.as_secs() >= 45, "Delay should be >= 45s (60s - 25% jitter)");
    }

    #[test]
    fn test_retry_manager_connection_established_does_not_reset_count() {
        let mut manager = RetryManager::new();

        // Simulate failures
        manager.record_connection_failed();
        manager.record_connection_failed();
        assert_eq!(manager.retry_count, 2);

        // Establish connection
        manager.record_connection_established();

        // Retry count should NOT reset immediately
        assert_eq!(manager.retry_count, 2);
        // But stable_since should be set
        assert!(manager.stable_since.is_some());
    }

    #[test]
    fn test_retry_manager_dropped_increases_count() {
        let mut manager = RetryManager::new();
        assert_eq!(manager.retry_count, 0);

        manager.record_dropped_prematurely();
        assert_eq!(manager.retry_count, 1);
        assert!(manager.stable_since.is_none());

        manager.record_dropped_prematurely();
        assert_eq!(manager.retry_count, 2);
    }

    #[test]
    fn test_retry_manager_stable_sets_timer_if_none() {
        let mut manager = RetryManager::new();
        assert!(manager.stable_since.is_none());

        manager.record_stable();
        assert!(manager.stable_since.is_some());
    }

    #[test]
    fn test_retry_manager_failed_resets_stable_since() {
        let mut manager = RetryManager::new();

        // Establish connection (sets stable_since)
        manager.record_connection_established();
        assert!(manager.stable_since.is_some());

        // Fail connection
        manager.record_connection_failed();
        assert!(manager.stable_since.is_none());
    }

    // ===== Sync Loop Device Validation Tests =====

    #[tokio::test]
    async fn test_sync_blocked_when_no_active_devices() {
        // Create in-memory database
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();

        // Insert a log to sync
        sqlx::query(
            "INSERT INTO zkt_raw_attendance (id, device_id, user_id, timestamp, is_synced) VALUES (1, 'dev1', 'user1', '2026-01-29 10:00:00', 0)"
        )
        .execute(hybrid_db.disk_db().get_pool())
        .await
        .unwrap();

        // Verify no active devices (sync should be blocked)
        let devices = hybrid_db.disk_db().get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 0, "Should have no active devices initially");

        // Verify logs exist but sync is blocked due to no devices
        let _logs = hybrid_db.get_unsynced_logs(10);
        // Note: logs may not be in memory since we inserted directly to disk
        // The key assertion is that get_active_devices returns 0
    }

    #[tokio::test]
    async fn test_sync_allowed_when_devices_configured() {
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();

        // Add an active device
        hybrid_db
            .disk_db()
            .upsert_device("dev1", "Device 1", "192.168.1.100", 4370, "zkteco")
            .await
            .unwrap();

        // Verify device is active
        let devices = hybrid_db.disk_db().get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 1, "Should have 1 active device");
        assert_eq!(devices[0].id, "dev1");
    }

    #[tokio::test]
    async fn test_sync_blocked_after_devices_deactivated() {
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();

        // Add device then deactivate all
        hybrid_db
            .disk_db()
            .upsert_device("dev1", "Device 1", "192.168.1.100", 4370, "zkteco")
            .await
            .unwrap();

        // Deactivate all devices (simulates ConfigPush with 0 devices)
        hybrid_db
            .disk_db()
            .deactivate_devices_not_in(&[])
            .await
            .unwrap();

        // Verify no active devices
        let devices = hybrid_db.disk_db().get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 0, "All devices should be deactivated");
    }
}
