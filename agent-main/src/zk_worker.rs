use crate::worker::Worker;
use async_trait::async_trait;
use rustzk::ZK;
use serde_json::Value;
use log::{info, warn, debug, error};
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
    zk: Option<ZK>,
    ip: String,
    // Cached static info
    mac: Option<String>,
    serial: Option<String>,
    firmware: Option<String>,
    platform: Option<String>,
    device_name: Option<String>,
    users_capacity: Option<usize>,
    logs_capacity: Option<usize>,
}

impl ZkWorker {
    pub fn new() -> Self {
        Self {
            zk: None,
            ip: String::new(),
            mac: None,
            serial: None,
            firmware: None,
            platform: None,
            device_name: None,
            users_capacity: None,
            logs_capacity: None,
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
                let users_cap = Some(zk.users_cap as usize);
                let logs_cap = Some(zk.rec_cap as usize);
                
                Some((mac, serial, firmware, platform, device_name, users_cap, logs_cap))
            } else {
                None
            };

            (zk, res, static_info)
        });

        match tokio::time::timeout(Duration::from_secs(20), join_handle).await {
            Ok(Ok((zk_instance, Ok(_), static_info))) => {
                 self.zk = Some(zk_instance);
                 
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
                let msg = format!("❌ [{ip}] Connection timed out after 20s");
                warn!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            }
        }
    }

    async fn get_attendance_logs(&mut self) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let ip = self.ip.clone();
        let mut zk = self.zk.take().ok_or(ZkError {
            message: "Device not connected".to_string(),
        })?;

        debug!("[{ip}] Fetching attendance logs from device...");

        let join_handle = tokio::task::spawn_blocking(move || {
            let logs = zk.get_attendance();
            (zk, logs)
        });

        match tokio::time::timeout(Duration::from_secs(300), join_handle).await {
            Ok(Ok((zk_instance, Ok(logs)))) => {
                self.zk = Some(zk_instance);
                debug!("[{ip}] Successfully fetched {} logs", logs.len());
                
                let json_logs: Vec<Value> = logs.into_iter().map(|log| {
                    serde_json::json!({
                        "user_id": log.user_id,
                        "timestamp": log.timestamp.to_string().replace("T", " "),
                        "status": log.status,
                        "punch": log.punch
                    })
                }).collect();
                
                Ok(json_logs)
            }
            Ok(Ok((zk_instance, Err(e)))) => {
                self.zk = Some(zk_instance);
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
                let msg = format!("[{ip}] Getting logs timed out after 300s");
                warn!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            }
        }
    }

    async fn get_info(&mut self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let ip = self.ip.clone();
        let ip_thread = ip.clone();
        let mut zk = self.zk.take().ok_or(ZkError {
            message: "Device not connected".to_string(),
        })?;

        // Clone cached values for thread safety
        let cached_firmware = self.firmware.clone().unwrap_or("N/A".to_string());
        let cached_mac = self.mac.clone().unwrap_or("N/A".to_string());
        let cached_serial = self.serial.clone().unwrap_or("N/A".to_string());
        let cached_platform = self.platform.clone().unwrap_or("N/A".to_string());
        let cached_name = self.device_name.clone().unwrap_or("N/A".to_string());
        let cached_users_cap = self.users_capacity.unwrap_or(0);
        let cached_logs_cap = self.logs_capacity.unwrap_or(0);

        let join_handle = tokio::task::spawn_blocking(move || {
            // Only fetch dynamic checks: get_time and read_sizes (for current counts)
            debug!("[{ip_thread}] Starting read_sizes (dynamic)...");
            // Note: read_sizes updates zk.users and zk.records internally if successful
            let (users_count, logs_count) = match zk.read_sizes() {
                Ok(_) => {
                    // zk.users and zk.records are u16 in rustzk, cast to usize
                    (zk.users as usize, zk.records as usize)
                },
                Err(e) => {
                    warn!("[{ip_thread}] read_sizes failed: {e:?}");
                    // Fallback: return -1 to signal error/unknown
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
                "last_sync": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            });
            
            (zk, info)
        });

        match tokio::time::timeout(Duration::from_secs(30), join_handle).await {
            Ok(Ok((zk_instance, info))) => {
                self.zk = Some(zk_instance);
                Ok(info)
            }
            Ok(Err(e)) => Err(Box::new(ZkError {
                message: format!("[{ip}] Task join error during get_info: {e}"),
            })),
            Err(_) => {
                let msg = format!("[{ip}] Getting device info timed out after 30s");
                warn!("{msg}");
                Err(Box::new(ZkError { message: msg }))
            }
        }
    }
}

impl Default for ZkWorker {
    fn default() -> Self {
        Self::new()
    }
}
