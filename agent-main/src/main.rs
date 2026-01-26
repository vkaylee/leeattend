use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use agent::zk_client::ZKClient;
use agent::database::Database;
use agent::central_client::CentralClient;
use agent::models;
use serde_json::json;
use log::{info, error, warn};
use dotenvy::dotenv;
use tokio_util::sync::CancellationToken;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("🚀 Starting Rust Agent Orchestrator (Full Service Server Mode)...");

    let cancel_token = CancellationToken::new();
    let cloned_token = cancel_token.clone();

    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Shutdown signal received (Ctrl+C).");
            cloned_token.cancel();
        }
    });

    let config = agent::config::Config::load();
    config.show_summary();

    let db = Database::new(&config.database_url).await.expect("Failed to initialize database");

    let (central_client, central_rx) = CentralClient::new(
        config.central_server_url, 
        config.agent_id, 
        config.agent_token, 
        db.clone()
    );
    central_client.start(central_rx).await;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let actual_port = listener.local_addr()?.port();
    info!("📡 RPC Server listening on 127.0.0.1:{}...", actual_port);

    while !cancel_token.is_cancelled() {
        info!("Starting ZK Python Worker Sidecar...");
        
        let mut child = Command::new(&config.python_path)
            .arg("../agent-zk-worker/zk_worker.py")
            .arg(actual_port.to_string())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stream = tokio::select! {
            res = listener.accept() => {
                match res {
                    Ok((s, addr)) => {
                        info!("✅ Sidecar connected from {}", addr);
                        s
                    },
                    Err(e) => {
                        error!("Accept error: {}", e);
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }
            },
            _ = cancel_token.cancelled() => {
                let _ = child.kill().await;
                break;
            }
            _ = sleep(Duration::from_secs(15)) => {
                error!("Sidecar failed to connect within 15s. Restarting...");
                let _ = child.kill().await;
                continue;
            }
        };

        let mut zk_client = ZKClient::new(stream);
        zk_client.set_timeout(config.rpc_timeout_seconds);

        info!("Entering main polling loop...");

        loop {
            if cancel_token.is_cancelled() {
                break;
            }

            let devices = match db.get_active_devices().await {
                Ok(d) => d,
                Err(e) => {
                    error!("Failed to fetch devices from local DB: {}", e);
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            if devices.is_empty() {
                warn!("No active devices configured. Waiting...");
            }

            for device in devices {
                info!("Processing device: {} ({})", device.name.as_deref().unwrap_or("Unnamed"), device.ip);
                
                let connect_res = zk_client.call("zk.connect", json!({
                    "device_id": device.id,
                    "ip": device.ip,
                    "port": device.port
                })).await;

                if let Err(e) = connect_res {
                    error!("Failed to connect to device {}: {}", device.id, e);
                    continue;
                }

                match zk_client.call("zk.get_attendance", json!({"device_id": device.id})).await {
                    Ok(logs_val) => {
                        if let Ok(logs) = serde_json::from_value::<Vec<models::AttendanceData>>(logs_val) {
                            let count = logs.len();
                            for log in logs {
                                db.save_raw_attendance(&device.id, log).await.ok();
                            }
                            if count > 0 {
                                info!("Successfully fetched and saved {} logs from {}", count, device.id);
                            }
                        }
                    },
                    Err(e) => error!("Failed to get logs from {}: {}", device.id, e),
                }

                let _ = zk_client.call("zk.sync_time", json!({"device_id": device.id})).await;
            }

            tokio::select! {
                _ = sleep(Duration::from_secs(config.poll_interval_seconds)) => {
                    info!("Poll interval reached. Starting next cycle...");
                },
                _ = cancel_token.cancelled() => break,
                exit_status = child.wait() => {
                    warn!("Sidecar process died: {:?}", exit_status);
                    break; 
                }
            }
        }

        if cancel_token.is_cancelled() {
            let _ = child.kill().await;
            break;
        }
    }

    info!("Agent Orchestrator stopped gracefully.");
    Ok(())
}
