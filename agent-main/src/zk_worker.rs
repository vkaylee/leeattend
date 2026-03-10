use crate::worker::Worker;
use async_trait::async_trait;
use rustzk::ZK;
use serde_json::Value;
use log::{info, warn, debug, error};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
}

impl ZkWorker {
    pub fn new(password: Option<u32>) -> Self {
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
        }
    }
}

#[async_trait]
impl Worker for ZkWorker {
    async fn connect(&mut self, ip: &str, port: u16) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        info!("[{ip}] Connecting to ZK device...");
        
        self.ip = ip.to_string();
        let ip_clone = self.ip.clone();
        let mut zk = ZK::new(&ip_clone, port);
        
        if let Some(pwd) = self.password {
            zk.set_password(pwd);
        }

        let join_handle = tokio::task::spawn_blocking(move || {
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
                let users_cap = Some(zk.users_cap() as usize);
                let logs_cap = Some(zk.records_cap() as usize);
                
                Some((mac, serial, firmware, platform, device_name, users_cap, logs_cap))
            } else {
                None
            };

            (zk, res, static_info)
        });

        match tokio::time::timeout(Duration::from_secs(20), join_handle).await {
            Ok(Ok((zk_instance, Ok(_), static_info))) => {
                 // Store ZK in Arc<Mutex>
                 if let Ok(mut guard) = self.zk.lock() {
                     *guard = Some(zk_instance);
                 }
                 
                 // Cache static info
                 if let Some((mac, serial, firmware, platform, device_name, users_cap, logs_cap)) = static_info {
                     self.mac = mac;
                     self.serial = serial;
                     self.firmware = firmware;
                     self.platform = platform;
                     self.device_name = device_name;
                     self.users_capacity = users_cap;
                     self.logs_capacity = logs_cap;
                     
                     info!("✅ [{ip}] Connected & cached info: MAC={:?}, Serial={:?}", self.mac, self.serial);
                 } else {
                     info!("✅ [{ip}] Successfully connected to ZK device (no static info)");
                 }
                 
                 Ok(true)
            }
            Ok(Ok((_, Err(e), _))) => {
                let msg = format!("❌ [{ip}] Device connection failed: {e}");
                warn!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            },
            Ok(Err(e)) => {
                let msg = format!("❌ [{ip}] Task join error: {e}");
                error!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            },
            Err(_) => {
                // TIMEOUT — blocking thread still running with ZK instance.
                // Abort it so the ZK gets dropped (triggers rustzk cleanup).
                // Note: spawn_blocking tasks cannot be truly aborted, but
                // the ZK will be dropped when the thread eventually finishes.
                warn!("⚠️ [{ip}] Connection timed out after 20s — blocking thread may still be running");
                let msg = format!("❌ [{ip}] Connection timed out after 20s");
                Err(Box::new(ZkError { message: msg }))
            }
        }
    }

    async fn get_attendance_logs(&mut self) -> Result<Vec<crate::models::AttendanceData>, Box<dyn std::error::Error + Send + Sync>> {
        let ip = self.ip.clone();
        let zk_arc = self.zk.clone();

        // Verify connection exists before spawning thread
        {
            let guard = zk_arc.lock().map_err(|_| ZkError { message: "Mutex poisoned".to_string() })?;
            if guard.is_none() {
                return Err(Box::new(ZkError { message: "Device not connected".to_string() }));
            }
        }

        debug!("[{ip}] Fetching attendance logs from device...");

        let join_handle = tokio::task::spawn_blocking(move || {
            let mut guard = zk_arc.lock().map_err(|_| "Mutex poisoned".to_string())?;
            let zk = guard.as_mut().ok_or("Device not connected".to_string())?;
            let logs = zk.get_attendance().map_err(|e| format!("{e}"))?;
            Ok::<_, String>(logs)
        });

        match tokio::time::timeout(Duration::from_secs(300), join_handle).await {
            Ok(Ok(Ok(logs))) => {
                debug!("[{ip}] Successfully fetched {} logs", logs.len());

                // Optimization: Pre-allocate vector
                let mut models_logs = Vec::with_capacity(logs.len());
                
                for log in logs {
                    models_logs.push(crate::models::AttendanceData {
                        user_id: log.user_id,
                        timestamp: log.timestamp.to_string().replace("T", " "), 
                        status: Some(log.status as i32),
                        punch: Some(log.punch as i32),
                    });
                }
                
                Ok(models_logs)
            }
            Ok(Ok(Err(e))) => {
                let msg = format!("[{ip}] Failed to get logs: {e}");
                warn!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            }
            Ok(Err(e)) => {
                let msg = format!("[{ip}] Task join error during get_logs: {e}");
                error!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            }
            Err(_) => {
                // TIMEOUT — blocking thread still holds mutex lock.
                // Instead of spawning a cleanup task (which races with outer disconnect()),
                // we clear the ZK instance here. The outer loop will call disconnect()
                // which will see None and no-op, avoiding the race condition.
                // The blocking thread will eventually finish and release the lock.
                let cleanup_arc = self.zk.clone();
                let cleanup_ip = self.ip.clone();
                tokio::task::spawn_blocking(move || {
                    // Wait for blocking thread to release lock, then take & disconnect
                    if let Ok(mut guard) = cleanup_arc.lock()
                        && let Some(mut zk) = guard.take() {
                            warn!("[{cleanup_ip}] Force-disconnecting after get_logs timeout (300s)");
                            let _ = zk.disconnect();
                        }
                });
                // Mark our local ZK as gone so outer disconnect() is a no-op
                if let Ok(mut guard) = self.zk.try_lock() {
                    let _ = guard.take();
                }
                let msg = format!("[{ip}] Getting logs timed out after 300s — force-disconnect scheduled");
                warn!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            }
        }
    }

    async fn get_info(&mut self, time_sync: &crate::time_sync::TimeSync) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let ip = self.ip.clone();
        let ip_thread = ip.clone();
        let zk_arc = self.zk.clone();

        // Verify connection exists before spawning thread
        {
            let guard = zk_arc.lock().map_err(|_| ZkError { message: "Mutex poisoned".to_string() })?;
            if guard.is_none() {
                return Err(Box::new(ZkError { message: "Device not connected".to_string() }));
            }
        }

        // Clone cached values for thread safety
        let cached_firmware = self.firmware.clone().unwrap_or("N/A".to_string());
        let cached_mac = self.mac.clone().unwrap_or("N/A".to_string());
        let cached_serial = self.serial.clone().unwrap_or("N/A".to_string());
        let cached_platform = self.platform.clone().unwrap_or("N/A".to_string());
        let cached_name = self.device_name.clone().unwrap_or("N/A".to_string());
        let cached_users_cap = self.users_capacity.unwrap_or(0);
        let cached_logs_cap = self.logs_capacity.unwrap_or(0);
        let last_sync_time = time_sync.corrected_now().to_rfc3339();

        let join_handle = tokio::task::spawn_blocking(move || {
            let mut guard = zk_arc.lock().map_err(|_| "Mutex poisoned".to_string())?;
            let zk = guard.as_mut().ok_or("Device not connected".to_string())?;

            // Only fetch dynamic checks: get_time and read_sizes (for current counts)
            debug!("[{ip_thread}] Starting read_sizes (dynamic)...");
            let (users_count, logs_count) = match zk.read_sizes() {
                Ok(_) => {
                    (zk.users() as usize, zk.records() as usize)
                },
                Err(e) => {
                    warn!("[{ip_thread}] read_sizes failed: {e:?}");
                    (0, 0)
                }
            };
            
            debug!("[{ip_thread}] Starting get_time...");
            let time = match zk.get_time() {
                Ok(t) => {
                    let s = t.naive_local().to_string().replace("T", " ");
                    debug!("[{ip_thread}] get_time success: {s}");
                    s
                },
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
            
            // Mutex unlocked here when guard drops
            Ok::<_, String>(info)
        });

        match tokio::time::timeout(Duration::from_secs(30), join_handle).await {
            Ok(Ok(Ok(info))) => {
                Ok(info)
            }
            Ok(Ok(Err(e))) => Err(Box::new(ZkError {
                message: format!("[{ip}] get_info operation failed: {e}"),
            })),
            Ok(Err(e)) => Err(Box::new(ZkError {
                message: format!("[{ip}] Task join error during get_info: {e}"),
            })),
            Err(_) => {
                // TIMEOUT — blocking thread still holds mutex lock.
                // Spawn cleanup that waits for lock release, then disconnects.
                let cleanup_arc = self.zk.clone();
                let cleanup_ip = self.ip.clone();
                tokio::task::spawn_blocking(move || {
                    if let Ok(mut guard) = cleanup_arc.lock()
                        && let Some(mut zk) = guard.take() {
                            warn!("[{cleanup_ip}] Force-disconnecting after get_info timeout (30s)");
                            let _ = zk.disconnect();
                        }
                });
                // Try to clear local ZK so outer disconnect() is a no-op
                if let Ok(mut guard) = self.zk.try_lock() {
                    let _ = guard.take();
                }
                let msg = format!("[{ip}] Getting device info timed out after 30s — force-disconnect scheduled");
                warn!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            }
        }
    }

    async fn ping(&mut self) -> bool {
        let zk_arc = self.zk.clone();
        let ip = self.ip.clone();

        // Quick check: if ZK is already gone, no point pinging
        if let Ok(guard) = self.zk.try_lock()
            && guard.is_none()
        {
            return false;
        }

        let join_handle = tokio::task::spawn_blocking(move || {
            let mut guard = zk_arc.lock().map_err(|_| "Mutex poisoned")?;
            let zk = guard.as_mut().ok_or("Device not connected")?;
            // Use get_time() as a lightweight probe — it's a single UDP roundtrip
            zk.get_time().map_err(|e| format!("{e}"))?;
            Ok::<_, String>(())
        });

        match tokio::time::timeout(Duration::from_secs(5), join_handle).await {
            Ok(Ok(Ok(()))) => true,
            Ok(Ok(Err(e))) => {
                debug!("[{ip}] Ping failed: {e}");
                false
            }
            Ok(Err(e)) => {
                debug!("[{ip}] Ping task error: {e}");
                false
            }
            Err(_) => {
                warn!("[{ip}] Ping timed out after 5s — connection likely dead");
                false
            }
        }
    }

    async fn disconnect(&mut self) {
        let zk_arc = self.zk.clone();
        let ip = self.ip.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(mut guard) = zk_arc.lock()
                && let Some(mut zk) = guard.take() {
                    info!("[{ip}] Explicitly disconnecting from ZK device...");
                    let _ = zk.disconnect();
                }
        }).await;
    }
}

impl Default for ZkWorker {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zk_error_display() {
        let err = ZkError { message: "connection timeout".into() };
        assert_eq!(format!("{err}"), "connection timeout");
    }

    #[test]
    fn test_zk_error_debug() {
        let err = ZkError { message: "test".into() };
        let debug = format!("{err:?}");
        assert!(debug.contains("ZkError"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_zk_error_is_std_error() {
        let err = ZkError { message: "x".into() };
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_zk_worker_new_no_password() {
        let worker = ZkWorker::new(None);
        assert!(worker.zk.lock().unwrap().is_none());
    }

    #[test]
    fn test_zk_worker_new_with_password() {
        let worker = ZkWorker::new(Some(12345));
        assert!(worker.zk.lock().unwrap().is_none()); // Not connected yet
    }

    #[test]
    fn test_zk_worker_default() {
        let worker = ZkWorker::default();
        assert!(worker.zk.lock().unwrap().is_none());
    }
}
