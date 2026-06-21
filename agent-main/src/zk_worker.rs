use crate::worker::Worker;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use rustzk::ZK;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// Simple error type to match the previous JsonRpcError structure for compatibility
#[derive(Debug)]
pub struct ZkError {
    pub message: String,
}

impl std::fmt::Display for ZkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ZkError {}

impl From<ZkError> for crate::errors::AgentError {
    fn from(err: ZkError) -> Self {
        if err.message.contains("timed out") {
            crate::errors::AgentError::ConnectionTimeout(err.message)
        } else if err.message.contains("not connected") || err.message.contains("Not connected") {
            crate::errors::AgentError::DeviceNotConnected
        } else if err.message.contains("failed") || err.message.contains("Failed") {
            crate::errors::AgentError::ConnectionFailed(err.message)
        } else {
            crate::errors::AgentError::ProtocolError(err.message)
        }
    }
}

pub struct ZkWorker {
    zk: Arc<Mutex<Option<ZK>>>,
    ip: String,
    // Cached static info
    mac: Option<String>,
    serial: Option<String>,
    firmware: Option<String>,
    platform: Option<String>,
    device_name: Option<String>,
    users_capacity: Option<usize>,
    logs_capacity: Option<usize>,
    password: Option<u32>,
    /// If set, skip checksum auto-detect handshake (saves 5-10s per connect).
    use_legacy_checksum: Option<bool>,
    /// Checksum mode detected/used during last successful connect.
    detected_checksum: Option<bool>,
    /// Timeout (seconds) for connect + initial info fetch
    connect_timeout_secs: u64,
    /// Timeout (seconds) for pulling attendance logs
    inflight_timeout_secs: u64,
    /// Concurrency lock to prevent socket leaks and duplicate operations on timeouts
    active_operation_lock: Arc<tokio::sync::Mutex<()>>,
}

impl ZkWorker {
    pub fn new(
        password: Option<u32>,
        use_legacy_checksum: Option<bool>,
        connect_timeout_secs: u64,
        inflight_timeout_secs: u64,
    ) -> Self {
        Self {
            zk: Arc::new(Mutex::new(None)),
            ip: String::new(),
            mac: None,
            serial: None,
            firmware: None,
            platform: None,
            device_name: None,
            users_capacity: None,
            logs_capacity: None,
            password,
            use_legacy_checksum,
            detected_checksum: None,
            connect_timeout_secs,
            inflight_timeout_secs,
            active_operation_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub const fn inflight_timeout_secs(&self) -> u64 {
        self.inflight_timeout_secs
    }
}

#[async_trait]
impl Worker for ZkWorker {
    async fn connect(&mut self, ip: &str, port: u16) -> Result<bool, crate::errors::AgentError> {
        info!("[{ip}] Connecting to ZK device...");

        self.ip = ip.to_string();
        let ip_clone = self.ip.clone();
        let mut zk = ZK::new(&ip_clone, port);

        if let Some(pwd) = self.password {
            zk.set_password(pwd);
        }

        // Skip checksum auto-detect if configured (saves 5-10s per connect)
        if let Some(legacy) = self.use_legacy_checksum {
            zk.set_legacy_checksum(legacy);
            debug!(
                "[{ip}] Checksum mode set to {} (skipping auto-detect)",
                if legacy { "legacy" } else { "modern" }
            );
        }

        let operation_lock = self.active_operation_lock.clone();
        let mut join_handle = tokio::task::spawn_blocking(move || {
            let _guard = operation_lock.blocking_lock();
            let res = zk.connect(rustzk::ZKProtocol::Auto);

            // If connected successfully, fetch static info immediately
            let static_info = if res.is_ok() {
                let mac = zk.get_mac().ok();
                let serial = zk.get_serial_number().ok();
                let firmware = zk.get_firmware_version().ok();
                let platform = zk.get_platform().ok();
                let device_name = zk.get_device_name().ok();

                // Read capacities
                let _ = zk.read_sizes();
                let users_cap = Some(usize::try_from(zk.users_cap()).unwrap_or(0));
                let logs_cap = Some(usize::try_from(zk.records_cap()).unwrap_or(0));

                Some((
                    mac,
                    serial,
                    firmware,
                    platform,
                    device_name,
                    users_cap,
                    logs_cap,
                ))
            } else {
                None
            };

            (zk, res, static_info)
        });

        match tokio::time::timeout(
            Duration::from_secs(self.connect_timeout_secs),
            &mut join_handle,
        )
        .await
        {
            Ok(Ok((zk_instance, Ok(_), static_info))) => {
                // Capture detected checksum mode BEFORE moving ZK into mutex
                let detected = zk_instance.use_legacy_checksum();
                self.detected_checksum = Some(detected);
                if self.use_legacy_checksum.is_none() {
                    info!(
                        "[{ip}] Auto-detected checksum: {} (will be remembered for next connect)",
                        if detected { "legacy" } else { "modern" }
                    );
                }

                // Store ZK in Arc<Mutex>
                {
                    let mut guard = self.zk.lock().await;
                    *guard = Some(zk_instance);
                }

                // Cache static info
                if let Some((mac, serial, firmware, platform, device_name, users_cap, logs_cap)) =
                    static_info
                {
                    self.mac = mac;
                    self.serial = serial;
                    self.firmware = firmware;
                    self.platform = platform;
                    self.device_name = device_name;
                    self.users_capacity = users_cap;
                    self.logs_capacity = logs_cap;

                    info!(
                        "✅ [{ip}] Connected & cached info: MAC={:?}, Serial={:?}",
                        self.mac, self.serial
                    );
                } else {
                    info!("✅ [{ip}] Successfully connected to ZK device (no static info)");
                }

                Ok(true)
            }
            Ok(Ok((_, Err(e), _))) => {
                let msg = format!("❌ [{ip}] Device connection failed: {e}");
                warn!("{msg}");
                Err(ZkError { message: msg }.into())
            }
            Ok(Err(e)) => {
                let msg = format!("❌ [{ip}] Task join error: {e}");
                error!("{msg}");
                Err(ZkError { message: msg }.into())
            }
            Err(_) => {
                let timeout_secs = self.connect_timeout_secs;
                warn!(
                    "⚠️ [{ip}] Connection timed out after {timeout_secs}s — spawning cleanup task for blocking thread"
                );
                let ip_clone = ip.to_string();
                let operation_lock_cleanup = self.active_operation_lock.clone();
                tokio::spawn(async move {
                    match join_handle.await {
                        Ok((mut zk, _, _)) => {
                            info!(
                                "[{ip_clone}] Connection background task finished post-timeout. Disconnecting..."
                            );
                            let _ = tokio::task::spawn_blocking(move || {
                                let _guard = operation_lock_cleanup.blocking_lock();
                                let _ = zk.disconnect();
                            })
                            .await;
                        }
                        Err(e) => {
                            error!(
                                "[{ip_clone}] Connection background task join error post-timeout: {e}"
                            );
                        }
                    }
                });
                let msg = format!("❌ [{ip}] Connection timed out after {timeout_secs}s");
                Err(ZkError { message: msg }.into())
            }
        }
    }

    async fn get_attendance_logs(
        &mut self,
    ) -> Result<Vec<crate::models::AttendanceData>, crate::errors::AgentError> {
        let ip = self.ip.clone();
        let zk_arc = self.zk.clone();

        // Take ZK out of Mutex to avoid holding the lock during blocking call
        let zk_val = {
            let mut guard = zk_arc.lock().await;
            match guard.take() {
                Some(zk) => zk,
                None => {
                    return Err(ZkError {
                        message: "Device not connected".to_string(),
                    }
                    .into());
                }
            }
        };

        debug!("[{ip}] Fetching attendance logs from device...");

        let operation_lock = self.active_operation_lock.clone();
        let mut join_handle = tokio::task::spawn_blocking(move || {
            let _guard = operation_lock.blocking_lock();
            let mut zk = zk_val;
            let res = zk.get_attendance().map_err(|e| format!("{e}"));
            (zk, res)
        });

        match tokio::time::timeout(
            Duration::from_secs(self.inflight_timeout_secs),
            &mut join_handle,
        )
        .await
        {
            Ok(Ok((zk, Ok(logs)))) => {
                debug!("[{ip}] Successfully fetched {} logs", logs.len());

                // Restore connection to Mutex
                {
                    let mut guard = zk_arc.lock().await;
                    *guard = Some(zk);
                }

                // Optimization: Pre-allocate vector
                let mut models_logs = Vec::with_capacity(logs.len());

                for log in logs {
                    models_logs.push(crate::models::AttendanceData {
                        user_id: log.user_id,
                        timestamp: log.timestamp.to_string().replace('T', " "),
                        status: Some(log.status as i32),
                        punch: Some(log.punch as i32),
                    });
                }

                Ok(models_logs)
            }
            Ok(Ok((zk, Err(e)))) => {
                // Restore connection to Mutex on normal error
                {
                    let mut guard = zk_arc.lock().await;
                    *guard = Some(zk);
                }
                let msg = format!("[{ip}] Failed to get logs: {e}");
                warn!("{msg}");
                Err(ZkError { message: msg }.into())
            }
            Ok(Err(e)) => {
                let msg = format!("[{ip}] Task join error during get_logs: {e}");
                error!("{msg}");
                Err(ZkError { message: msg }.into())
            }
            Err(_) => {
                let timeout_secs = self.inflight_timeout_secs;
                let msg = format!(
                    "[{ip}] Getting logs timed out after {timeout_secs}s — connection dropped"
                );
                warn!("{msg}");
                let ip_clone = ip.clone();
                let operation_lock_cleanup = self.active_operation_lock.clone();
                tokio::spawn(async move {
                    match join_handle.await {
                        Ok((mut zk, _)) => {
                            info!(
                                "[{ip_clone}] Get logs background task finished post-timeout. Disconnecting..."
                            );
                            let _ = tokio::task::spawn_blocking(move || {
                                let _guard = operation_lock_cleanup.blocking_lock();
                                let _ = zk.disconnect();
                            })
                            .await;
                        }
                        Err(e) => {
                            error!(
                                "[{ip_clone}] Get logs background task join error post-timeout: {e}"
                            );
                        }
                    }
                });
                Err(ZkError { message: msg }.into())
            }
        }
    }

    async fn get_info(
        &mut self,
        time_sync: &crate::time_sync::TimeSync,
    ) -> Result<Value, crate::errors::AgentError> {
        let ip = self.ip.clone();
        let ip_thread = ip.clone();
        let zk_arc = self.zk.clone();

        // Take ZK out of Mutex to avoid holding the lock during blocking call
        let zk_val = {
            let mut guard = zk_arc.lock().await;
            match guard.take() {
                Some(zk) => zk,
                None => {
                    return Err(ZkError {
                        message: "Device not connected".to_string(),
                    }
                    .into());
                }
            }
        };

        // Clone cached values for thread safety
        let cached_firmware = self.firmware.clone().unwrap_or("N/A".to_string());
        let cached_mac = self.mac.clone().unwrap_or("N/A".to_string());
        let cached_serial = self.serial.clone().unwrap_or("N/A".to_string());
        let cached_platform = self.platform.clone().unwrap_or("N/A".to_string());
        let cached_name = self.device_name.clone().unwrap_or("N/A".to_string());
        let cached_users_cap = self.users_capacity.unwrap_or(0);
        let cached_logs_cap = self.logs_capacity.unwrap_or(0);
        let last_sync_time = time_sync.corrected_now().to_rfc3339();

        let operation_lock = self.active_operation_lock.clone();
        let mut join_handle = tokio::task::spawn_blocking(move || {
            let _guard = operation_lock.blocking_lock();
            let mut zk = zk_val;

            // Only fetch dynamic checks: get_time and read_sizes (for current counts)
            debug!("[{ip_thread}] Starting read_sizes (dynamic)...");
            let (users_count, logs_count) = match zk.read_sizes() {
                Ok(_) => (zk.users() as usize, zk.records() as usize),
                Err(e) => {
                    warn!("[{ip_thread}] read_sizes failed: {e:?}");
                    (0, 0)
                }
            };

            debug!("[{ip_thread}] Starting get_time...");
            let time = match zk.get_time() {
                Ok(t) => {
                    let s = t.naive_local().to_string().replace('T', " ");
                    debug!("[{ip_thread}] get_time success: {s}");
                    s
                }
                Err(e) => {
                    debug!("[{ip_thread}] get_time unavailable: {e:?}");
                    "N/A".to_string()
                }
            };

            let info = serde_json::json!({
                "users_count": users_count,
                "users_capacity": cached_users_cap,
                "logs_count": logs_count,
                "logs_capacity": cached_logs_cap,
                "device_time": time,
                "firmware": cached_firmware,
                "mac": cached_mac,
                "serial": cached_serial,
                "platform": cached_platform,
                "hw_name": cached_name,
                "last_sync": last_sync_time
            });

            (zk, Ok::<_, String>(info))
        });

        match tokio::time::timeout(Duration::from_secs(30), &mut join_handle).await {
            Ok(Ok((zk, Ok(info)))) => {
                // Restore connection to Mutex
                {
                    let mut guard = zk_arc.lock().await;
                    *guard = Some(zk);
                }
                Ok(info)
            }
            Ok(Ok((zk, Err(e)))) => {
                // Restore connection to Mutex on error
                {
                    let mut guard = zk_arc.lock().await;
                    *guard = Some(zk);
                }
                Err(ZkError {
                    message: format!("[{ip}] get_info operation failed: {e}"),
                }
                .into())
            }
            Ok(Err(e)) => Err(ZkError {
                message: format!("[{ip}] Task join error during get_info: {e}"),
            }
            .into()),
            Err(_) => {
                let msg =
                    format!("[{ip}] Getting device info timed out after 30s — connection dropped");
                warn!("{msg}");
                let ip_clone = ip.clone();
                let operation_lock_cleanup = self.active_operation_lock.clone();
                tokio::spawn(async move {
                    match join_handle.await {
                        Ok((mut zk, _)) => {
                            info!(
                                "[{ip_clone}] Get info background task finished post-timeout. Disconnecting..."
                            );
                            let _ = tokio::task::spawn_blocking(move || {
                                let _guard = operation_lock_cleanup.blocking_lock();
                                let _ = zk.disconnect();
                            })
                            .await;
                        }
                        Err(e) => {
                            error!(
                                "[{ip_clone}] Get info background task join error post-timeout: {e}"
                            );
                        }
                    }
                });
                Err(ZkError { message: msg }.into())
            }
        }
    }

    async fn ping(&mut self) -> bool {
        let zk_arc = self.zk.clone();
        let ip = self.ip.clone();

        // Take ZK out of Mutex to avoid holding the lock during blocking call
        let zk_val = {
            if let Ok(mut guard) = zk_arc.try_lock() {
                match guard.take() {
                    Some(zk) => zk,
                    None => return false,
                }
            } else {
                return false;
            }
        };

        let operation_lock = self.active_operation_lock.clone();
        let mut join_handle = tokio::task::spawn_blocking(move || {
            let _guard = operation_lock.blocking_lock();
            let mut zk = zk_val;
            // Use get_time() as a lightweight probe — it's a single UDP roundtrip
            let res = zk.get_time().map_err(|e| format!("{e}"));
            (zk, res)
        });

        match tokio::time::timeout(Duration::from_secs(5), &mut join_handle).await {
            Ok(Ok((zk, Ok(_)))) => {
                // Restore connection to Mutex
                {
                    let mut guard = zk_arc.lock().await;
                    *guard = Some(zk);
                }
                true
            }
            Ok(Ok((zk, Err(e)))) => {
                // Restore connection to Mutex
                {
                    let mut guard = zk_arc.lock().await;
                    *guard = Some(zk);
                }
                debug!("[{ip}] Ping failed: {e}");
                false
            }
            Ok(Err(e)) => {
                debug!("[{ip}] Ping task error: {e}");
                false
            }
            Err(_) => {
                warn!(
                    "[{ip}] Ping timed out after 5s — connection likely dead. Spawning cleanup task."
                );
                let ip_clone = ip.clone();
                let operation_lock_cleanup = self.active_operation_lock.clone();
                tokio::spawn(async move {
                    match join_handle.await {
                        Ok((mut zk, _)) => {
                            info!(
                                "[{ip_clone}] Ping background task finished post-timeout. Disconnecting..."
                            );
                            let _ = tokio::task::spawn_blocking(move || {
                                let _guard = operation_lock_cleanup.blocking_lock();
                                let _ = zk.disconnect();
                            })
                            .await;
                        }
                        Err(e) => {
                            error!(
                                "[{ip_clone}] Ping background task join error post-timeout: {e}"
                            );
                        }
                    }
                });
                false
            }
        }
    }

    async fn disconnect(&mut self) {
        let zk_arc = self.zk.clone();
        let ip = self.ip.clone();
        let mut guard = zk_arc.lock().await;
        if let Some(mut zk) = guard.take() {
            let ip_clone = ip.clone();
            let operation_lock = self.active_operation_lock.clone();
            let join_handle = tokio::task::spawn_blocking(move || {
                let _guard = operation_lock.blocking_lock();
                info!("[{ip_clone}] Explicitly disconnecting from ZK device...");
                let _ = zk.disconnect();
            });
            match tokio::time::timeout(Duration::from_secs(5), join_handle).await {
                Ok(Ok(_)) => {
                    info!("[{ip}] Disconnected successfully.");
                }
                Ok(Err(e)) => {
                    error!("[{ip}] Disconnect task join error: {e}");
                }
                Err(_) => {
                    warn!("[{ip}] Disconnect timed out after 5s — continuing shutdown/reload");
                }
            }
        }
    }

    fn detected_checksum_mode(&self) -> Option<bool> {
        self.detected_checksum
    }
}

impl Default for ZkWorker {
    fn default() -> Self {
        Self::new(None, None, 60, 300)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zk_error_display() {
        let err = ZkError {
            message: "connection timeout".into(),
        };
        assert_eq!(format!("{err}"), "connection timeout");
    }

    #[test]
    fn test_zk_error_debug() {
        let err = ZkError {
            message: "test".into(),
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("ZkError"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_zk_error_is_std_error() {
        let err = ZkError {
            message: "x".into(),
        };
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_zk_worker_new_no_password() {
        let worker = ZkWorker::new(None, None, 60, 300);
        assert!(worker.zk.blocking_lock().is_none());
    }

    #[test]
    fn test_zk_worker_new_with_password() {
        let worker = ZkWorker::new(Some(12345), None, 60, 300);
        assert!(worker.zk.blocking_lock().is_none()); // Not connected yet
    }

    #[test]
    fn test_zk_worker_new_with_legacy_checksum() {
        let worker = ZkWorker::new(None, Some(true), 60, 300);
        assert!(worker.use_legacy_checksum == Some(true));
        assert!(worker.zk.blocking_lock().is_none());
    }

    #[test]
    fn test_zk_worker_new_with_modern_checksum() {
        let worker = ZkWorker::new(None, Some(false), 60, 300);
        assert!(worker.use_legacy_checksum == Some(false));
    }

    #[test]
    fn test_zk_worker_inflight_timeout_config() {
        let worker = ZkWorker::new(None, None, 60, 600);
        assert_eq!(worker.inflight_timeout_secs(), 600);
    }

    #[test]
    fn test_zk_worker_default() {
        let worker = ZkWorker::default();
        assert!(worker.zk.blocking_lock().is_none());
    }
}
