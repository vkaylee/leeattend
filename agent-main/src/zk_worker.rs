use rustzk::ZK;
use serde_json::Value;
use log::{info, warn, debug};

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
}

impl ZkWorker {
    pub fn new() -> Self {
        Self {
            zk: None,
        }
    }

    pub async fn connect(&mut self, ip: &str, port: u16) -> Result<bool, ZkError> {
        info!("Connecting to ZK device at {ip}:{port}");
        
        // Clone for move into closure
        let ip = ip.to_string();
        
        let mut zk = ZK::new(&ip, port);
        
        let res = tokio::task::spawn_blocking(move || {
            let res = zk.connect(rustzk::ZKProtocol::Auto); 
            (zk, res)
        }).await;

        match res {
            Ok((zk_instance, Ok(_))) => {
                 self.zk = Some(zk_instance);
                 Ok(true)
            }
            Ok((_, Err(e))) => Err(ZkError {
                message: format!("Device connection failed: {e}"),
            }),
            Err(e) => Err(ZkError {
                message: format!("Task join error: {e}"),
            }),
        }
    }

    pub async fn get_attendance_logs(&mut self) -> Result<Vec<Value>, ZkError> {
        let mut zk = self.zk.take().ok_or(ZkError {
            message: "Device not connected".to_string(),
        })?;

        let res = tokio::task::spawn_blocking(move || {
            let logs = zk.get_attendance();
            (zk, logs)
        }).await;

        match res {
            Ok((zk_instance, Ok(logs))) => {
                self.zk = Some(zk_instance);
                
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
            Ok((zk_instance, Err(e))) => {
                self.zk = Some(zk_instance);
                Err(ZkError {
                    message: format!("Failed to get logs: {e}"),
                })
            }
            Err(e) => {
                Err(ZkError {
                    message: format!("Task join error: {e}"),
                })
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
        let mut zk = self.zk.take().ok_or(ZkError {
            message: "Device not connected".to_string(),
        })?;

        let res = tokio::task::spawn_blocking(move || {
            match zk.read_sizes() {
                Ok(_) => debug!("read_sizes success: users_cap={}, rec_cap={}", zk.users_cap, zk.rec_cap),
                Err(e) => warn!("read_sizes failed: {e:?}"),
            }
            
            let users_count = zk.get_users().map(|u| u.len()).unwrap_or(0);
            let logs_count = zk.get_attendance().map(|l| l.len()).unwrap_or(0);
            
            let time = match zk.get_time() {
                Ok(t) => {
                    let s = t.naive_local().to_string().replace("T", " ");
                    debug!("get_time success: {s}");
                    s
                },
                Err(e) => {
                    debug!("get_time unavailable (device RTC may be unset or use non-standard encoding): {e:?}");
                    "N/A".to_string()
                }
            };

            let firmware = zk.get_firmware_version().unwrap_or("N/A".to_string());
            let mac = zk.get_mac().unwrap_or("N/A".to_string());
            let serial = zk.get_serial_number().unwrap_or("N/A".to_string());
            let platform = zk.get_platform().unwrap_or("N/A".to_string());
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
        }).await;

         match res {
            Ok((zk_instance, info)) => {
                self.zk = Some(zk_instance);
                Ok(info)
            }
            Err(e) => Err(ZkError {
                message: format!("Task join error: {e}"),
            }),
        }
    }
}
