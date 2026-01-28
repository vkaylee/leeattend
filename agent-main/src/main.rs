use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashSet;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use agent::worker_client::WorkerClient;
use agent::database::Database;
use agent::central_client::CentralClient;
use agent::models;
use agent::retry::{get_sidecar_retry_delay, SIDECAR_MAX_RETRY_COUNT, SIDECAR_STABILITY_THRESHOLD_SECS};
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

    // Check for CLI commands
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let config = agent::config::Config::load();
        let db = Database::new(&config.database_url).await.expect("Failed to initialize database");

        match args[1].as_str() {
            "failed-logs" | "failed" => {
                return handle_failed_logs_command(&db, &args).await;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                print_help();
                return Ok(());
            }
        }
    }

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
    // Use Arc<AtomicBool> to ensure only one writer starts
    let writer_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (log_tx, mut log_rx) = mpsc::channel::<(String, models::AttendanceData)>(10000);
    let writer_db = db.clone();

    if !writer_started.swap(true, std::sync::atomic::Ordering::SeqCst) {
        tokio::spawn(async move {
            info!("💾 DB Writer Worker started (Batch Mode)");
            let mut buffer = Vec::with_capacity(100);
            let mut interval = tokio::time::interval(Duration::from_millis(200));

            loop {
                tokio::select! {
                    msg = log_rx.recv() => {
                        match msg {
                            Some(item) => {
                                buffer.push(item);
                                if buffer.len() >= 100 {
                                    let to_save = std::mem::replace(&mut buffer, Vec::with_capacity(100));
                                    if let Err(e) = writer_db.save_raw_attendance_batch(to_save).await {
                                        error!("Failed to save batch to DB: {}", e);
                                    }
                                    // Yield to let sync loop acquire lock
                                    sleep(Duration::from_millis(10)).await;
                                }
                            }
                            None => break,
                        }
                    }
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            let to_save = std::mem::replace(&mut buffer, Vec::with_capacity(100));
                            if let Err(e) = writer_db.save_raw_attendance_batch(to_save).await {
                                error!("Failed to save timed batch to DB: {}", e);
                            }
                        }
                    }
                }
            }
            info!("💾 DB Writer Worker stopped");
        });
    } else {
        warn!("⚠️ DB Writer Worker already running, skipping duplicate spawn");
    }

    // --- Periodic Cleanup Task (Old Synced Logs) ---
    let cleanup_db = db.clone();
    let cleanup_cancel = cancel_token.clone();
    tokio::spawn(async move {
        info!("🧹 Cleanup Worker started (365-day retention)");

        loop {
            // Run cleanup every 24 hours
            tokio::select! {
                _ = sleep(Duration::from_secs(86400)) => {
                    match cleanup_db.cleanup_old_synced_logs(365).await {
                        Ok(deleted) => {
                            if deleted > 0 {
                                info!("🧹 Cleaned up {} old synced logs (>365 days)", deleted);
                            }
                        }
                        Err(e) => {
                            error!("❌ Cleanup failed: {}", e);
                        }
                    }
                }
                _ = cleanup_cancel.cancelled() => {
                    info!("🧹 Cleanup Worker stopped");
                    break;
                }
            }
        }
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
            info!("📡 Found {} devices, max {} concurrent workers", db_devices.len(), current_max_workers);
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
                info!("[{}] Device Polling Loop started ({})", worker.name, worker.ip);

                let mut retry_count: u32 = 0;
                #[allow(unused_assignments)]
                let mut stable_since: Option<Instant> = None;

                while !token.is_cancelled() {
                    // Apply retry delay with exponential backoff
                    if retry_count > 0 {
                        let delay = get_sidecar_retry_delay(retry_count);
                        warn!("[{}] Retry {}/{}, waiting {:?}...", worker.name, retry_count, SIDECAR_MAX_RETRY_COUNT, delay);
                        sleep(delay).await;
                    }

                    let listener = match TcpListener::bind("127.0.0.1:0").await {
                        Ok(l) => l,
                        Err(e) => {
                            error!("[{}] Failed to bind local RPC listener: {}", worker.name, e);
                            retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
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
                                retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
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
                                    retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                                    continue;
                                }
                            }
                        }
                        _ = sleep(Duration::from_secs(10)) => {
                            error!("[{}] Sidecar connection timeout", worker.name);
                            let _ = sidecar.kill().await;
                            retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                            continue;
                        }
                        _ = token.cancelled() => {
                            let _ = sidecar.kill().await;
                            return;
                        }
                    };

                    // Convert tokio TcpStream to std TcpStream for WorkerClient
                    let std_stream = match stream.into_std() {
                        Ok(s) => s,
                        Err(e) => {
                            error!("[{}] Failed to convert stream: {}", worker.name, e);
                            let _ = sidecar.kill().await;
                            retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                            continue;
                        }
                    };

                    let mut client = match WorkerClient::from_stream(std_stream) {
                        Ok(c) => c,
                        Err(e) => {
                            error!("[{}] Failed to create client: {}", worker.name, e);
                            let _ = sidecar.kill().await;
                            retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                            continue;
                        }
                    };

                    info!("[{}] Sidecar connected and ready.", worker.name);
                    stable_since = Some(Instant::now());

                    if let Err(e) = client.call("device.connect", json!({"ip": worker.ip, "port": worker.port})) {
                        error!("[{}] Device connection failed: {}", worker.name, e);
                        let _ = sidecar.kill().await;
                        retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                        continue;
                    }

                    // In-memory duplicate detection: tracks seen logs to avoid sending duplicates to queue
                    // Key format: "user_id:timestamp" (device_id is implicit per worker)
                    let mut seen_logs: HashSet<String> = HashSet::new();

                    while !token.is_cancelled() {
                        // Acquire semaphore permit for this poll cycle (fair round-robin)
                        // Released at end of loop iteration, allowing other devices to poll
                        let _permit = sem.acquire().await.unwrap();

                        match client.call("device.get_logs", json!({})) {
                            Ok(res) => {
                                // Check stability and reset retry count if stable
                                if let Some(since) = stable_since {
                                    if since.elapsed() >= Duration::from_secs(SIDECAR_STABILITY_THRESHOLD_SECS) && retry_count > 0 {
                                        info!("[{}] Connection stable for {}s, resetting retry count", worker.name, SIDECAR_STABILITY_THRESHOLD_SECS);
                                        retry_count = 0;
                                    }
                                }

                                if let Some(logs) = res.as_array().filter(|l| !l.is_empty()) {
                                    let fetched_count = logs.len();
                                    let mut new_count = 0;
                                    let mut duplicate_count = 0;

                                    // Filter duplicates before sending to queue
                                    for log in logs {
                                        if let Ok(attendance) = serde_json::from_value::<models::AttendanceData>(log.clone()) {
                                            // Create unique key: user_id:timestamp
                                            let key = format!("{}:{}", attendance.user_id, attendance.timestamp);

                                            if seen_logs.insert(key) {
                                                // New log - send to queue
                                                let _ = log_tx.send((worker.name.clone(), attendance)).await;
                                                new_count += 1;
                                            } else {
                                                // Duplicate - skip
                                                duplicate_count += 1;
                                            }
                                        }
                                    }

                                    if duplicate_count > 0 {
                                        info!("[{}] Fetched {} logs ({} new, {} duplicates filtered)",
                                              worker.name, fetched_count, new_count, duplicate_count);
                                    } else {
                                        info!("[{}] Fetched {} logs", worker.name, fetched_count);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("[{}] Polling error: {}", worker.name, e);
                                break;
                            }
                        }

                        // Release permit before sleeping to allow other devices to poll
                        drop(_permit);
                        sleep(Duration::from_secs(5)).await;
                    }

                    let _ = sidecar.kill().await;
                    if !token.is_cancelled() {
                        warn!("[{}] Sidecar connection lost, restarting...", worker.name);
                        retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
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

fn print_help() {
    println!("Usage: agent [COMMAND]");
    println!();
    println!("Commands:");
    println!("  failed-logs [limit]    Display logs that failed to sync (default limit: 50)");
    println!("  failed [limit]         Alias for failed-logs");
    println!("  --help, -h             Display this help message");
    println!();
    println!("Examples:");
    println!("  agent                  Start the agent orchestrator");
    println!("  agent failed-logs      Show failed sync logs (default 50)");
    println!("  agent failed-logs 100  Show 100 failed sync logs");
    println!("  agent failed           Short form of failed-logs");
}

async fn handle_failed_logs_command(db: &Database, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let limit = if args.len() > 2 {
        args[2].parse::<i64>().unwrap_or(50)
    } else {
        50
    };

    let total_count = db.count_failed_sync_logs().await?;
    let failed_logs = db.get_failed_sync_logs(limit).await?;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                          FAILED SYNC LOGS REPORT                             ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total failed logs: {} (showing top {})", total_count, failed_logs.len());
    println!();

    if failed_logs.is_empty() {
        println!("✅ No failed sync logs found. All logs either synced or never attempted.");
        println!();
        return Ok(());
    }

    println!("┌─────────┬────────────┬────────────┬─────────────────────┬────────┬───────┐");
    println!("│ ID      │ Device     │ User ID    │ Timestamp           │ Status │ Punch │");
    println!("├─────────┼────────────┼────────────┼─────────────────────┼────────┼───────┤");

    for log in &failed_logs {
        let timestamp_str = log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
        let status_str = log.status.map(|s| s.to_string()).unwrap_or_else(|| "NULL".to_string());
        let punch_str = log.punch.map(|p| p.to_string()).unwrap_or_else(|| "NULL".to_string());

        println!(
            "│ {:7} │ {:10} │ {:10} │ {:19} │ {:6} │ {:5} │",
            log.id,
            truncate(&log.device_id, 10),
            truncate(&log.user_id, 10),
            truncate(&timestamp_str, 19),
            truncate(&status_str, 6),
            truncate(&punch_str, 5)
        );
    }

    println!("└─────────┴────────────┴────────────┴─────────────────────┴────────┴───────┘");
    println!();
    println!("Legend:");
    println!("  - These logs were attempted to sync but failed");
    println!("  - They will be retried in the next sync cycle");
    println!("  - Check central server logs for error details");
    println!();

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
