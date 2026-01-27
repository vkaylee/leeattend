use std::process::Stdio;
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
use tokio::sync::mpsc;
use futures::future::join_all;

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

    let (central_client, central_rx) = CentralClient::new(
        config.central_server_url.clone(), 
        config.agent_id.clone(), 
        config.agent_token.clone(), 
        db.clone()
    );
    central_client.start(central_rx).await;

    // --- DB Writer Worker (Batch Mode) ---
    let (log_tx, mut log_rx) = mpsc::channel::<(String, models::AttendanceData)>(10000); 
    let writer_db = db.clone();
    
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

    // In a real scenario, these would come from the central server configuration
    let workers = vec![
        WorkerConfig {
            name: "ZK-Entrance".to_string(),
            brand: "zkteco".to_string(),
            ip: "192.168.1.201".to_string(),
            port: 4370,
            sidecar_script: "../agent-zk-worker/zk_worker.py".to_string(),
        },
        WorkerConfig {
            name: "Mock-Exit".to_string(),
            brand: "mock".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 9999,
            sidecar_script: "../agent-mock-worker/mock_worker.py".to_string(),
        },
    ];

    let mut polling_handles = vec![];

    for worker in workers {
        let log_tx = log_tx.clone();
        let cancel_token = cancel_token.clone();
        let python_path = config.python_path.clone();

        let handle = tokio::spawn(async move {
            info!("Device Polling Loop started for: {} ({})", worker.name, worker.ip);
            
            while !cancel_token.is_cancelled() {
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
                };

                let mut client = match WorkerClient::new(&stream.local_addr().unwrap().to_string()) {
                    Ok(c) => c,
                    Err(e) => {
                        // Use the stream directly instead of connecting again
                        let rpc_addr = stream.peer_addr().unwrap().to_string();
                        match WorkerClient::new(&rpc_addr) {
                            Ok(c) => c,
                            Err(_) => {
                                error!("[{}] Failed to create client: {}", worker.name, e);
                                let _ = sidecar.kill().await;
                                continue;
                            }
                        }
                    }
                };

                info!("[{}] Sidecar connected and ready.", worker.name);

                if let Err(e) = client.call("device.connect", json!({"ip": worker.ip, "port": worker.port})) {
                    error!("[{}] Device connection failed: {}", worker.name, e);
                    let _ = sidecar.kill().await;
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }

                while !cancel_token.is_cancelled() {
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
                warn!("[{}] Sidecar connection lost, restarting...", worker.name);
                sleep(Duration::from_secs(5)).await;
            }
        });

        polling_handles.push(handle);
    }

    join_all(polling_handles).await;
    info!("Orchestrator shutting down...");
    Ok(())
}
