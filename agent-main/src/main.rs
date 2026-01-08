use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use agent::zk_client::ZKClient;
use serde_json::json;
use log::{info, error, warn};
use dotenvy::dotenv;
use std::env;

mod database;
mod central_client;
mod models;

use database::Database;
use central_client::CentralClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    info!("🚀 Starting Rust Agent Orchestrator...");

    // 1. Initialize Database
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:attendance.db".to_string());
    let db = Database::new(&db_url).await.expect("Failed to initialize database");

    // 2. Start Central Client
    let server_url = env::var("CENTRAL_SERVER_URL").expect("CENTRAL_SERVER_URL must be set");
    let agent_id = env::var("AGENT_ID").expect("AGENT_ID must be set");
    let token = env::var("AGENT_TOKEN").expect("AGENT_TOKEN must be set");

    let (central_client, central_rx) = CentralClient::new(server_url, agent_id, token, db.clone());
    central_client.start(central_rx).await;

    // Supervisor Loop for Auto-Restart
    loop {
        // 3. Start Python Worker (Sidecar)
        info!("Starting ZK Python Worker sidecar...");
        
        let worker_bin_name = if cfg!(target_os = "windows") { "agent-zk-worker.exe" } else { "agent-zk-worker" };
        let paths_to_check = vec![
            std::env::current_dir()?.join(worker_bin_name),
            std::env::current_dir()?.join("../agent-zk-worker/dist").join(worker_bin_name),
        ];

        let mut worker_cmd = None;
        for path in paths_to_check {
            if path.exists() {
                info!("Found packaged worker binary at: {:?}", path);
                worker_cmd = Some(Command::new(path));
                break;
            }
        }

        let mut child = if let Some(mut cmd) = worker_cmd {
            cmd.stdout(Stdio::inherit())
               .stderr(Stdio::inherit())
               .spawn()?
        } else {
            info!("Packaged worker not found. Fallback to Dev Python script...");
            let python_path = if cfg!(target_os = "windows") {
                "../agent-zk-worker/venv/Scripts/python.exe"
            } else {
                "../agent-zk-worker/venv_linux/bin/python3"
            };
            Command::new(python_path)
                .arg("../agent-zk-worker/zk_worker.py")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
        };

        // Wait for worker to boot
        sleep(Duration::from_secs(3)).await;

        // 4. Connect to Sidecar
        let sidecar_addr = "127.0.0.1:5000";
        info!("Connecting to Sidecar at {}...", sidecar_addr);
        
        // Retry loop for sidecar connection
        let mut zk_client = loop {
            match ZKClient::connect(sidecar_addr).await {
                Ok(client) => break client,
                Err(e) => {
                    error!("Failed to connect to sidecar: {}. Retrying in 2s...", e);
                    sleep(Duration::from_secs(2)).await;
                }
            }
        };
        info!("✅ Connected to Sidecar.");

        // 5. Main Control Loop
        // Periodically fetch logs from devices in DB configuration
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut sync_push_interval = tokio::time::interval(Duration::from_secs(10));
        
        loop {
            tokio::select! {
                _ = sync_push_interval.tick() => {
                    match db.get_unsynced_logs(50).await {
                        Ok(logs) => {
                            if !logs.is_empty() {
                                info!("Pushing {} unsynced logs to Central...", logs.len());
                                for log in logs {
                                    let data = models::AttendanceData {
                                        user_id: log.user_id,
                                        timestamp: log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                                        status: log.status.map(|v| v as i32),
                                        punch: log.punch.map(|v| v as i32),
                                    };
                                    central_client.push_log(log.device_id, data).await;
                                    let _ = db.mark_synced(&[log.id]).await;
                                }
                            }
                        }
                        Err(e) => error!("Failed to fetch unsynced logs: {}", e),
                    }
                }

                _ = interval.tick() => {
                    info!("Running periodic sync task...");
                    
                    // A. Get active devices from DB
                    let devices = db.get_active_devices().await.unwrap_or_default();
                    
                    for dev in devices {
                        info!("Syncing device: {} ({})", dev.id, dev.ip);
                        
                        // B. Connect Device (RPC)
                        let connect_res = zk_client.call("zk.connect", json!({
                            "device_id": dev.id,
                            "ip": dev.ip,
                            "port": dev.port
                        })).await;
                        
                        if connect_res.is_ok() {
                            // C. Get Logs
                            match zk_client.call("zk.get_attendance", json!({"device_id": dev.id})).await {
                                Ok(logs) => {
                                    if let Some(logs_arr) = logs.as_array() {
                                        info!("Fetched {} logs from {}", logs_arr.len(), dev.id);
                                        // D. Save to DB
                                        for log in logs_arr {
                                            // Parse log fields
                                            let user_id = log.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
                                            let ts_str = log.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                                            
                                            if let Ok(ts) = chrono::NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d %H:%M:%S") {
                                                 let _ = db.save_raw_attendance(
                                                    &dev.id, 
                                                    user_id, 
                                                    ts, 
                                                    log.get("status").and_then(|v| v.as_i64()).map(|v| v as i32),
                                                    log.get("punch").and_then(|v| v.as_i64()).map(|v| v as i32)
                                                ).await;
                                            }
                                        }
                                    }
                                }
                                Err(e) => error!("Failed to get logs from {}: {:?}", dev.id, e),
                            }
                        } else {
                            warn!("Failed to connect to device {}", dev.id);
                        }
                    }
                }
                
                // Check child process
                status = child.wait() => {
                    error!("Sidecar exited: {:?}. Restarting...", status);
                    break; // Break inner loop to restart worker
                }
                
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutdown signal received.");
                    let _ = child.kill().await;
                    return Ok(()); // Return from main to exit app
                }
            }
        }
        
        warn!("Restarting worker in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}