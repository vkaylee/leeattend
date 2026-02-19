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
}

impl ZkWorker {
    pub fn new() -> Self {
        Self {
            zk: None,
            ip: String::new(),
        }
    }

    pub async fn connect(&mut self, ip: &str, port: u16) -> Result<bool, ZkError> {
        info!("[{ip}] Connecting to ZK device...");
        
        self.ip = ip.to_string();
        let ip_clone = self.ip.clone();
        let mut zk = ZK::new(&ip_clone, port);
        
        let join_handle = tokio::task::spawn_blocking(move || {
            let res = zk.connect(rustzk::ZKProtocol::Auto); 
            (zk, res)
        });

        match tokio::time::timeout(Duration::from_secs(10), join_handle).await {
            Ok(Ok((zk_instance, Ok(_)))) => {
                 self.zk = Some(zk_instance);
                 info!("✅ [{ip}] Successfully connected to ZK device");
                 Ok(true)
            }
            Ok(Ok((_, Err(e)))) => {
                let msg = format!("❌ [{ip}] Device connection failed: {e}");
                warn!("{msg}");
                Err(ZkError { message: msg })
            },
            Ok(Err(e)) => {
                let msg = format!("❌ [{ip}] Task join error: {e}");
                error!("{msg}");
                Err(ZkError { message: msg })
            },
            Err(_) => {
                let msg = format!("❌ [{ip}] Connection timed out after 10s");
                warn!("{msg}");
                Err(ZkError { message: msg })
            }
        }
    }

    pub async fn get_attendance_logs(&mut self) -> Result<Vec<Value>, ZkError> {
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
                Err(ZkError { message: msg })
            }
            Ok(Err(e)) => {
                let msg = format!("[{ip}] Task join error during get_logs: {e}");
                error!("{msg}");
                Err(ZkError { message: msg })
            }
            Err(_) => {
                let msg = format!("[{ip}] Getting logs timed out after 30s");
                warn!("{msg}");
                Err(ZkError { message: msg })
            }
        }
    }

    #[allow(dead_code)]
    pub async fn get_time(&mut self) -> Result<String, ZkError> {
        let mut zk = self.zk.take().ok_or(ZkError {
            message: "Device not connected".to_string(),
        })?;

        let res = tokio::task::spawn_blocking(move || {
            let time = zk.get_time();
            (zk, time)
        }).await;

        match res {
            Ok((zk_instance, Ok(time))) => {
                self.zk = Some(zk_instance);
                // time is DateTime<FixedOffset>
                Ok(time.naive_local().to_string().replace("T", " "))
            }
            Ok((zk_instance, Err(e))) => {
                self.zk = Some(zk_instance);
                Err(ZkError {
                    message: format!("Failed to get time: {e}"),
                })
            }
            Err(e) => Err(ZkError {
                message: format!("Task join error: {e}"),
            }),
        }
    }

    pub async fn get_info(&mut self) -> Result<Value, ZkError> {
        let ip = self.ip.clone();
        let ip_thread = ip.clone();
        let mut zk = self.zk.take().ok_or(ZkError {
            message: "Device not connected".to_string(),
        })?;

        let join_handle = tokio::task::spawn_blocking(move || {
            debug!("[{ip_thread}] Starting read_sizes...");
            match zk.read_sizes() {
                Ok(_) => debug!("[{ip_thread}] read_sizes success: users_cap={}, rec_cap={}", zk.users_cap, zk.rec_cap),
                Err(e) => warn!("[{ip_thread}] read_sizes failed: {e:?}"),
            }
            
            // OPTIMIZATION: Use cached counts from read_sizes instead of fetching all data
            let users_count = zk.users as usize;
            let logs_count = zk.records as usize;
            
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

            debug!("[{ip_thread}] Starting hardware info fetching...");
            debug!("[{ip_thread}] - Fetching firmware...");
            let firmware = zk.get_firmware_version().unwrap_or("N/A".to_string());
            debug!("[{ip_thread}] - Fetching mac...");
            let mac = zk.get_mac().unwrap_or("N/A".to_string());
            debug!("[{ip_thread}] - Fetching serial...");
            let serial = zk.get_serial_number().unwrap_or("N/A".to_string());
            debug!("[{ip_thread}] - Fetching platform...");
            let platform = zk.get_platform().unwrap_or("N/A".to_string());
            debug!("[{ip_thread}] - Fetching name...");
            let device_name = zk.get_device_name().unwrap_or("N/A".to_string());
            
            let info = serde_json::json!({
                "users_count": users_count,
                "users_capacity": zk.users_cap,
                "logs_count": logs_count,
                "logs_capacity": zk.rec_cap,
                "device_time": time,
                "firmware": firmware,
                "mac": mac,
                "serial": serial,
                "platform": platform,
                "hw_name": device_name,
                "last_sync": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            });
            
            (zk, info)
        });

        match tokio::time::timeout(Duration::from_secs(60), join_handle).await {
            Ok(Ok((zk_instance, info))) => {
                self.zk = Some(zk_instance);
                Ok(info)
            }
            Ok(Err(e)) => Err(ZkError {
                message: format!("[{ip}] Task join error during get_info: {e}"),
            }),
            Err(_) => {
                // If timeout occurs, the ZK instance is lost because it's still in the spawned thread
                let msg = format!("[{ip}] Getting device info timed out after 30s");
                warn!("{msg}");
                Err(ZkError { message: msg })
            }
        }
    }
}

impl Default for ZkWorker {
    fn default() -> Self {
        Self::new()
    }
}
