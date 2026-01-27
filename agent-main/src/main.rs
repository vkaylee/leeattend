use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use agent::worker_client::WorkerClient;
use agent::database::Database;
use agent::central_client::CentralClient;
use agent::models;
use serde_json::json;
use log::{info, error, warn};
use dotenvy::dotenv;
use tokio_util::sync::CancellationToken;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Semaphore};

#[derive(Clone)]
struct WorkerConfig {
    name: String,
    brand: String,
    ip: String,
    port: u32,
    sidecar_script: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("🚀 Starting Multi-Brand Rust Agent Orchestrator...");

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

    // Load dynamic config
    let db_max_workers = db.get_agent_config("max_concurrent_workers").await.unwrap_or(None);
    let mut current_max_workers = if let Some(s) = db_max_workers {
        s.parse().unwrap_or(config.max_concurrent_workers)
    } else {
        config.max_concurrent_workers
    };
    info!("⚙️ Effective Max Workers: {}", current_max_workers);


    let (central_client, central_rx, mut reload_rx) = CentralClient::new(
        config.central_server_url.clone(), 
        config.agent_id.clone(), 
        config.agent_token.clone(), 
        db.clone()
    );
    central_client.start(central_rx).await;

    // --- DB Writer Worker (Batch Mode) ---
    let (log_tx, mut log_rx) = mpsc::channel::<(String, models::AttendanceData)>(10000); 
    let writer_db = db.clone();
    // Semaphore moved to loop
    
    tokio::spawn(async move {
        info!("💾 DB Writer Worker started (Batch Mode)");
        let mut buffer = Vec::with_capacity(500);
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            tokio::select! {
                msg = log_rx.recv() => {
                    match msg {
                        Some(item) => {
                            buffer.push(item);
                            if buffer.len() >= 500 {
                                let to_save = std::mem::replace(&mut buffer, Vec::with_capacity(500));
                                if let Err(e) = writer_db.save_raw_attendance_batch(to_save).await {
                                    error!("Failed to save batch to DB: {}", e);
                                }
                            }
                        }
                        None => break,
                    }
                }
                _ = interval.tick() => {
                    if !buffer.is_empty() {
                        let to_save = std::mem::replace(&mut buffer, Vec::with_capacity(500));
                        if let Err(e) = writer_db.save_raw_attendance_batch(to_save).await {
                            error!("Failed to save timed batch to DB: {}", e);
                        }
                    }
                }
            }
        }
        info!("💾 DB Writer Worker stopped");
    });

    while !cancel_token.is_cancelled() {
        let semaphore = Arc::new(Semaphore::new(current_max_workers));

        let db_devices = match db.get_active_devices().await {
            Ok(devices) => devices,
            Err(e) => {
                error!("Failed to load devices from DB: {}", e);
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        if db_devices.is_empty() {
            warn!("No active devices found in database. Waiting for config push...");
        } else {
            info!("Initializing {} workers from database...", db_devices.len());
        }

        let workers: Vec<WorkerConfig> = db_devices.into_iter().map(|d| {
            let sidecar_script = match d.brand.as_str() {
                "zkteco" => "../agent-zk-worker/zk_worker.py".to_string(),
                "mock" => "../agent-mock-worker/mock_worker.py".to_string(),
                _ => "../agent-zk-worker/zk_worker.py".to_string(),
            };
            WorkerConfig {
                name: d.name,
                brand: d.brand,
                ip: d.ip,
                port: d.port as u32,
                sidecar_script,
            }
        }).collect();

        let worker_cancel_token = CancellationToken::new();

        for worker in workers {
            let log_tx = log_tx.clone();
            let token = worker_cancel_token.clone();
            let python_path = config.python_path.clone();
            let sem = semaphore.clone();

            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                info!("[{}] Device Polling Loop started ({})", worker.name, worker.ip);
                
                while !token.is_cancelled() {
                    let listener = match TcpListener::bind("127.0.0.1:0").await {
                        Ok(l) => l,
                        Err(e) => {
                            error!("[{}] Failed to bind local RPC listener: {}", worker.name, e);
                            sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    };
                    let rpc_port = listener.local_addr().unwrap().port();

                    info!("[{}] Spawning sidecar ({}) on RPC port {}...", worker.name, worker.brand, rpc_port);
                    
                    let mut sidecar = match Command::new(&python_path)
                        .arg(&worker.sidecar_script)
                        .arg(rpc_port.to_string())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn() {
                            Ok(s) => s,
                            Err(e) => {
                                error!("[{}] Failed to spawn sidecar: {}", worker.name, e);
                                sleep(Duration::from_secs(10)).await;
                                continue;
                            }
                        };

                    let stream = tokio::select! {
                        res = listener.accept() => {
                            match res {
                                Ok((s, _)) => s,
                                Err(e) => {
                                    error!("[{}] Accept error: {}", worker.name, e);
                                    let _ = sidecar.kill().await;
                                    sleep(Duration::from_secs(2)).await;
                                    continue;
                                }
                            }
                        }
                        _ = sleep(Duration::from_secs(10)) => {
                            error!("[{}] Sidecar connection timeout", worker.name);
                            let _ = sidecar.kill().await;
                            continue;
                        }
                        _ = token.cancelled() => {
                            let _ = sidecar.kill().await;
                            return;
                        }
                    };

                    let mut client = match WorkerClient::new(&stream.peer_addr().unwrap().to_string()) {
                        Ok(c) => c,
                        Err(e) => {
                            error!("[{}] Failed to create client: {}", worker.name, e);
                            let _ = sidecar.kill().await;
                            continue;
                        }
                    };

                    info!("[{}] Sidecar connected and ready.", worker.name);

                    if let Err(e) = client.call("device.connect", json!({"ip": worker.ip, "port": worker.port})) {
                        error!("[{}] Device connection failed: {}", worker.name, e);
                        let _ = sidecar.kill().await;
                        sleep(Duration::from_secs(10)).await;
                        continue;
                    }

                    while !token.is_cancelled() {
                        match client.call("device.get_logs", json!({})) {
                            Ok(res) => {
                                if let Some(logs) = res.as_array().filter(|l| !l.is_empty()) {
                                    info!("[{}] Fetched {} logs", worker.name, logs.len());
                                    for log in logs {
                                        if let Ok(attendance) = serde_json::from_value::<models::AttendanceData>(log.clone()) {
                                            let _ = log_tx.send((worker.name.clone(), attendance)).await;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("[{}] Polling error: {}", worker.name, e);
                                break;
                            }
                        }
                        sleep(Duration::from_secs(5)).await;
                    }
                    
                    let _ = sidecar.kill().await;
                    if !token.is_cancelled() {
                        warn!("[{}] Sidecar connection lost, restarting...", worker.name);
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            });
        }

        tokio::select! {
            _ = reload_rx.recv() => {
                info!("🔄 Configuration change detected! Restarting all workers...");
                
                // Reload config
                if let Ok(Some(s)) = db.get_agent_config("max_concurrent_workers").await
                    && let Ok(new_val) = s.parse::<usize>()
                    && new_val != current_max_workers
                {
                    info!("⚡ Updating Max Workers: {} -> {}", current_max_workers, new_val);
                    current_max_workers = new_val;
                }

                worker_cancel_token.cancel();
                sleep(Duration::from_secs(2)).await;
            }
            _ = cancel_token.cancelled() => {
                info!("Stopping orchestrator...");
                worker_cancel_token.cancel();
                break;
            }
        }
    }

    info!("Orchestrator shut down.");
    Ok(())
}
