use rustzk::ZK;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, warn, error};
use chrono::{Local, NaiveDateTime};

pub struct ZkService {
    zk: Option<ZK>,
    last_attendance_count: i32,
}

impl ZkService {
    pub fn new() -> Self {
        Self {
            zk: None,
            last_attendance_count: -1,
        }
    }

    pub async fn connect(&mut self, params: Value) -> Result<Value, super::JsonRpcError> {
        let ip = params["ip"].as_str().ok_or(super::JsonRpcError {
            code: -32602,
            message: "Missing 'ip' parameter".to_string(),
        })?;
        let port = params["port"].as_u64().unwrap_or(4370) as u16;

        info!("Connecting to ZK device at {}:{}", ip, port);

        // rustzk::ZK::new creates the struct but doesn't connect yet.
        // The quickstart example says:
        // let mut zk = ZK::new("192.168.1.201", 4370); 
        // if let Ok(_) = zk.connect(true) ...
        
        // Since rustzk calls might be blocking (UDP sockets), strict async usage might block.
        // However, looking at the library, it might use std::net::UdpSocket.
        // For Phase 1 scaffold, we'll try straight usage, but might need spawn_blocking if it blocks the loop.
        // Given Phase 2 is full implementation, here we put placeholders or basic logic.
        
        let mut zk = ZK::new(ip, port);
        
        // Connect logic
        // zk.connect returns Result<(), ZKError>
        // We need to wrap blocking calls if they are blocking.
        // Assuming rustzk is synchronous for now based on typical rust-embedded/iot libs unless documented "async".
        // Research showed "Binary Protocol ... Native implementation".
        
        // For safety in async context, wrap in spawn_blocking if we suspect blocking.
        // But ZK struct isn't Send/Sync usually? 
        // If ZK is Send, we can move it.
        
        let res = tokio::task::spawn_blocking(move || {
            let res = zk.connect(false); // false = UDP
            (zk, res)
        }).await;

        match res {
            Ok((zk_instance, Ok(_))) => {
                 self.zk = Some(zk_instance);
                 Ok(serde_json::json!(true))
            }
            Ok((_, Err(e))) => Err(super::JsonRpcError {
                code: -32603,
                message: format!("Device connection failed: {}", e),
            }),
            Err(e) => Err(super::JsonRpcError {
                code: -32603,
                message: format!("Task join error: {}", e),
            }),
        }
    }

    pub async fn get_attendance_logs(&mut self) -> Result<Value, super::JsonRpcError> {
        let mut zk = self.zk.take().ok_or(super::JsonRpcError {
            code: -32603,
            message: "Device not connected".to_string(),
        })?;

        let res = tokio::task::spawn_blocking(move || {
            let logs = zk.get_attendance();
            (zk, logs)
        }).await;

        match res {
            Ok((zk_instance, Ok(logs))) => {
                self.zk = Some(zk_instance);
                self.last_attendance_count = logs.len() as i32;
                
                let json_logs: Vec<Value> = logs.into_iter().map(|log| {
                    serde_json::json!({
                        "user_id": log.user_id,
                        "timestamp": log.timestamp.to_string().replace("T", " "),
                        "status": log.status,
                        "punch": log.punch
                    })
                }).collect();
                
                Ok(serde_json::json!(json_logs))
            }
            Ok((zk_instance, Err(e))) => {
                self.zk = Some(zk_instance);
                Err(super::JsonRpcError {
                    code: -32603,
                    message: format!("Failed to get logs: {}", e),
                })
            }
            Err(e) => {
                // zk is lost if thread panics, but that's acceptable for now (connection lost)
                Err(super::JsonRpcError {
                    code: -32603,
                    message: format!("Task join error: {}", e),
                })
            }
        }
    }

    pub async fn get_time(&mut self) -> Result<Value, super::JsonRpcError> {
        let mut zk = self.zk.take().ok_or(super::JsonRpcError {
            code: -32603,
            message: "Device not connected".to_string(),
        })?;

        let res = tokio::task::spawn_blocking(move || {
            let time = zk.get_time();
            (zk, time)
        }).await;

        match res {
            Ok((zk_instance, Ok(time))) => {
                self.zk = Some(zk_instance);
                Ok(serde_json::json!(time.to_string().replace("T", " ")))
            }
            Ok((zk_instance, Err(e))) => {
                self.zk = Some(zk_instance);
                Err(super::JsonRpcError {
                    code: -32603,
                    message: format!("Failed to get time: {}", e),
                })
            }
            Err(e) => Err(super::JsonRpcError {
                code: -32603,
                message: format!("Task join error: {}", e),
            }),
        }
    }

    pub async fn sync_time(&mut self) -> Result<Value, super::JsonRpcError> {
        let mut zk = self.zk.take().ok_or(super::JsonRpcError {
            code: -32603,
            message: "Device not connected".to_string(),
        })?;

        let res: Result<(ZK, bool, NaiveDateTime), _> = tokio::task::spawn_blocking(move || {
            let now = chrono::Local::now().naive_local();
            
            // Manual set_time implementation since rustzk doesn't expose it
            // ZK Time Encoding: ((year % 100) * 12 * 31 + ((month - 1) * 31) + day - 1) * (24 * 60 * 60) + (hour * 60 + minute) * 60 + second
            use chrono::Datelike;
            use chrono::Timelike;
            
            let time = (
                ((now.year() % 100) as u32 * 12 * 31 + ((now.month() - 1) * 31) + now.day() - 1) * (24 * 60 * 60) +
                (now.hour() * 60 + now.minute()) * 60 +
                now.second()
            );
            
            let payload = time.to_le_bytes().to_vec();
            let success = match zk.send_command(rustzk::constants::CMD_SET_TIME, payload) {
                Ok(res) => res.command == rustzk::constants::CMD_ACK_OK,
                Err(_) => false,
            };
            
            (zk, success, now)
        }).await;

        match res {
            Ok((zk_instance, true, now)) => {
                self.zk = Some(zk_instance);
                Ok(serde_json::json!(now.to_string().replace("T", " ")))
            }
            Ok((zk_instance, false, _)) => {
                self.zk = Some(zk_instance);
                Err(super::JsonRpcError {
                    code: -32603,
                    message: "Failed to set time".to_string(),
                })
            }
            Err(e) => Err(super::JsonRpcError {
                code: -32603,
                message: format!("Task join error: {}", e),
            }),
        }
    }
    
    pub async fn get_info(&mut self) -> Result<Value, super::JsonRpcError> {
        let mut zk = self.zk.take().ok_or(super::JsonRpcError {
            code: -32603,
            message: "Device not connected".to_string(),
        })?;

        // Run multiple calls in one blocking task to avoid repeated thread spawning/moving
        let res = tokio::task::spawn_blocking(move || {
            let users_count = zk.get_users().map(|u| u.len()).unwrap_or(0);
            let logs_count = zk.get_attendance().map(|l| l.len()).unwrap_or(0); // If fails, use 0
            let time = zk.get_time().map(|t| t.to_string().replace("T", " ")).unwrap_or("N/A".to_string());
            let firmware = zk.get_firmware_version().unwrap_or("N/A".to_string());
            let mac = zk.get_mac().unwrap_or("N/A".to_string());
            let serial = zk.get_serial_number().unwrap_or("N/A".to_string());
            let platform = zk.get_platform().unwrap_or("N/A".to_string());
            let device_name = zk.get_device_name().unwrap_or("N/A".to_string());
            
            let info = serde_json::json!({
                "users_count": users_count,
                "logs_count": logs_count,
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
            Err(e) => Err(super::JsonRpcError {
                code: -32603,
                message: format!("Task join error: {}", e),
            }),
        }
    }
}
