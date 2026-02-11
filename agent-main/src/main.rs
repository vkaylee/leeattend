use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashSet;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use agent::worker_client::WorkerClient;
use agent::database::Database;
use agent::central_client::CentralClient;
use agent::hybrid_database::HybridDatabase;
use agent::models;
use agent::retry::{get_sidecar_retry_delay, SIDECAR_MAX_RETRY_COUNT, SIDECAR_STABILITY_THRESHOLD_SECS};
use agent::crash_recovery::{CrashMarker, recover_from_crash};
use agent::adaptive_polling::AdaptivePollingState;
use serde_json::json;
use log::{info, error, warn};
// use dotenvy::dotenv;
use tokio_util::sync::CancellationToken;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Semaphore};

#[derive(Clone)]
struct WorkerConfig {
    id: String,
    name: String,
    brand: String,
    ip: String,
    port: u32,
    sidecar_script: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

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
            "dlq" | "dead-letter-queue" => {
                return handle_dlq_command(&db, &args).await;
            }
            "dlq-stats" | "stats" => {
                return handle_dlq_stats_command(&db).await;
            }
            "config" => {
                return handle_config_command(&db, &args).await;
            }
            "status" => {
                return handle_status_command(&db).await;
            }
            "--help" | "-h" | "help" => {
                print_help();
                return Ok(());
            }
            "--version" | "-v" => {
                print_version();
                return Ok(());
            }
            _ => {
                eprintln!("❌ Unknown command: {}", args[1]);
                eprintln!();
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
            info!("Flushing in-memory data to disk...");
            cloned_token.cancel();
        }
    });

    let config = agent::config::Config::load();
    config.show_summary();

    // === CRASH RECOVERY ===
    let crash_marker = CrashMarker::new(&config.database_url);

    if crash_marker.check_crash() {
        warn!("⚠️ Detected previous crash or unclean shutdown");
        if let Some(info) = crash_marker.read_crash_info() {
            warn!("Previous session info:\n{}", info);
        }
    }

    // Initialize HybridDatabase (memory + disk)
    let hybrid_db = HybridDatabase::new(&config.database_url, 100_000)
        .await
        .expect("Failed to initialize HybridDatabase");

    // Perform crash recovery if needed
    if crash_marker.check_crash() {
        let recovered = recover_from_crash(hybrid_db.disk_db()).await
            .expect("Failed to recover from crash");
        info!("✅ Crash recovery complete: {} logs reset", recovered);
    }

    // Write new crash marker
    if let Err(e) = crash_marker.write() {
        warn!("Failed to write crash marker: {}", e);
    }

    // Clear startup config: Agent should NOT use stale database config.
    // Server will send ConfigPush after Handshake with fresh configuration.
    if let Err(e) = hybrid_db.disk_db().clear_startup_config().await {
        warn!("Failed to clear startup config: {}", e);
    } else {
        info!("🔄 Cleared local config on startup - waiting for ConfigPush from Server...");
    }

    // Use defaults until ConfigPush arrives (database config was just cleared)
    let mut current_max_workers = config.max_concurrent_workers;
    info!("⚙️ Using default Max Workers: {} (until ConfigPush)", current_max_workers);

    let (central_client, central_rx, mut reload_rx) = CentralClient::new(
        config.central_server_url.clone(),
        config.agent_id.clone(),
        config.agent_token.clone(),
        hybrid_db.clone(),
    );
    central_client.start(central_rx).await;

    // --- DB Writer Worker (Batch Mode) ---
    // Use Arc<AtomicBool> to ensure only one writer starts
    let writer_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (log_tx, mut log_rx) = mpsc::channel::<(String, models::AttendanceData)>(10000);
    let writer_hybrid = hybrid_db.clone();

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
                                    if let Err(e) = writer_hybrid.save_raw_attendance_batch(to_save).await {
                                        error!("Failed to save batch to memory: {}", e);
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
                            if let Err(e) = writer_hybrid.save_raw_attendance_batch(to_save).await {
                                error!("Failed to save timed batch to memory: {}", e);
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

    // --- Checkpoint Worker (Periodic Disk Persistence) ---
    let checkpoint_hybrid = hybrid_db.clone();
    let checkpoint_cancel = cancel_token.clone();
    tokio::spawn(async move {
        info!("💾 Checkpoint Worker started (5s interval)");

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if checkpoint_hybrid.memory_store().is_dirty() {
                        match checkpoint_hybrid.checkpoint().await {
                            Ok(count) => {
                                if count > 0 {
                                    info!("💾 Checkpoint: {} logs persisted to disk", count);
                                }
                            }
                            Err(e) => {
                                error!("❌ Checkpoint failed: {}", e);
                            }
                        }
                    }
                }
                _ = checkpoint_cancel.cancelled() => {
                    info!("💾 Checkpoint Worker: Final flush...");
                    match checkpoint_hybrid.flush_all().await {
                        Ok(count) => info!("✅ Final flush: {} logs written", count),
                        Err(e) => error!("❌ Final flush failed: {}", e),
                    }
                    info!("💾 Checkpoint Worker stopped");
                    break;
                }
            }
        }
    });

    // --- Periodic Disk Cleanup Task (Old Logs >365 Days) ---
    // Note: Memory cleanup is now handled by mark_synced() immediately on ACK
    let cleanup_hybrid = hybrid_db.clone();
    let cleanup_cancel = cancel_token.clone();
    tokio::spawn(async move {
        info!("🧹 Disk Cleanup Worker started (365-day retention)");

        // Disk cleanup interval: every 24 hours
        let disk_cleanup_interval = Duration::from_secs(86400);
        let mut disk_cleanup_timer = tokio::time::interval(disk_cleanup_interval);

        // Skip immediate tick on startup
        disk_cleanup_timer.tick().await;

        loop {
            tokio::select! {
                _ = disk_cleanup_timer.tick() => {
                    match cleanup_hybrid.disk_db().cleanup_old_synced_logs(365).await {
                        Ok(deleted) => {
                            if deleted > 0 {
                                info!("🧹 Disk cleanup: removed {} old synced logs (>365 days)", deleted);
                            }
                        }
                        Err(e) => {
                            error!("❌ Disk cleanup failed: {}", e);
                        }
                    }
                }
                _ = cleanup_cancel.cancelled() => {
                    info!("🧹 Disk Cleanup Worker stopped");
                    break;
                }
            }
        }
    });

    while !cancel_token.is_cancelled() {
        let semaphore = Arc::new(Semaphore::new(current_max_workers));

        let db_devices = match hybrid_db.disk_db().get_active_devices().await {
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

        // Runtime optimization: Load memory for active devices before spawning
        for d in &db_devices {
            if let Err(e) = hybrid_db.load_device_memory(&d.id).await {
                error!("Failed to load memory for device {}: {}", d.id, e);
            }
        }

        let workers: Vec<WorkerConfig> = db_devices.into_iter().map(|d| {
            let sidecar_script = match d.brand.as_str() {
                "zkteco" => "../agent-zk-worker/zk_worker.py".to_string(),
                "mock" => "../agent-mock-worker/mock_worker.py".to_string(),
                _ => "../agent-zk-worker/zk_worker.py".to_string(),
            };
            WorkerConfig {
                id: d.id.clone(),
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

                    // Adaptive polling: reduce frequency when no new data
                    let mut polling_state = AdaptivePollingState::new();

                    while !token.is_cancelled() {
                        // Acquire semaphore permit for this poll cycle (fair round-robin)
                        // Released at end of loop iteration, allowing other devices to poll
                        let _permit = sem.acquire().await.unwrap();

                        match client.call("device.get_logs", json!({})) {
                            Ok(res) => {
                                // Check stability and reset retry count if stable
                                if let Some(since) = stable_since
                                    && since.elapsed() >= Duration::from_secs(SIDECAR_STABILITY_THRESHOLD_SECS) && retry_count > 0 {
                                        info!("[{}] Connection stable for {}s, resetting retry count", worker.name, SIDECAR_STABILITY_THRESHOLD_SECS);
                                        retry_count = 0;
                                    }

                                let mut new_count = 0;
                                if let Some(logs) = res.as_array().filter(|l| !l.is_empty()) {
                                    let fetched_count = logs.len();
                                    let mut duplicate_count = 0;

                                    // Filter duplicates before sending to queue
                                    for log in logs {
                                        if let Ok(attendance) = serde_json::from_value::<models::AttendanceData>(log.clone()) {
                                            // Create unique key: user_id:timestamp
                                            let key = format!("{}:{}", attendance.user_id, attendance.timestamp);

                                            if seen_logs.insert(key) {
                                                // New log - send to queue using unique device ID (not name)
                                                let _ = log_tx.send((worker.id.clone(), attendance)).await;
                                                new_count += 1;
                                            } else {
                                                // Duplicate - skip
                                                duplicate_count += 1;
                                            }
                                        }
                                    }

                                    if duplicate_count > 0 {
                                        info!("[{}] Fetched {} logs ({} new, {} duplicates filtered) [interval: {}s]",
                                              worker.name, fetched_count, new_count, duplicate_count, polling_state.interval());
                                    } else {
                                        info!("[{}] Fetched {} logs [interval: {}s]", worker.name, fetched_count, polling_state.interval());
                                    }
                                }

                                // Adaptive polling: adjust interval based on new data
                                let (new_interval, changed) = polling_state.update(new_count > 0);
                                if changed && new_count == 0 {
                                    info!("[{}] No new data for {} polls, increasing interval: {}s",
                                          worker.name, polling_state.consecutive_no_new_data, new_interval);
                                }
                            }
                            Err(e) => {
                                error!("[{}] Polling error: {}", worker.name, e);
                                break;
                            }
                        }

                        // Release permit before sleeping to allow other devices to poll
                        drop(_permit);
                        sleep(Duration::from_secs(polling_state.interval())).await;
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
                if let Ok(Some(s)) = hybrid_db.disk_db().get_agent_config("max_concurrent_workers").await
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

    // Wait for checkpoint worker to complete final flush
    info!("⏳ Waiting for final checkpoint...");
    sleep(Duration::from_secs(2)).await;

    // Remove crash marker on clean exit
    if let Err(e) = crash_marker.remove() {
        warn!("Failed to remove crash marker: {}", e);
    }
    info!("✅ Clean shutdown complete.");

    Ok(())
}

fn print_version() {
    println!("Agent Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Build: {}", env!("CARGO_PKG_NAME"));
}

fn print_help() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    LeeAttend Agent - Command Line Tool                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("USAGE:");
    println!("    agent [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!();
    println!("  Sync Queue Management:");
    println!("    failed-logs [limit]      Display logs that failed to sync (retrying)");
    println!("    failed [limit]           Alias for failed-logs (default limit: 50)");
    println!();
    println!("  Dead Letter Queue (DLQ):");
    println!("    dlq [limit]              Display permanently failed logs in DLQ");
    println!("    dead-letter-queue [N]    Same as dlq (default limit: 50)");
    println!("    dlq-stats                Show DLQ statistics and failure analysis");
    println!("    stats                    Alias for dlq-stats");
    println!();
    println!("  Configuration & Status:");
    println!("    config [key] [value]     View or set agent configuration");
    println!("    status                   Show agent status and queue metrics");
    println!();
    println!("  General:");
    println!("    --help, -h, help         Display this help message");
    println!("    --version, -v            Display agent version");
    println!();
    println!("EXAMPLES:");
    println!();
    println!("  Start the agent:");
    println!("    agent                    Run agent orchestrator (main mode)");
    println!();
    println!("  Query failed sync logs (retrying):");
    println!("    agent failed-logs        Show 50 logs that failed but will retry");
    println!("    agent failed 100         Show 100 failed logs");
    println!();
    println!("  Query dead letter queue (permanently failed):");
    println!("    agent dlq                Show 50 permanently failed logs");
    println!("    agent dlq 200            Show 200 DLQ entries");
    println!("    agent dlq-stats          View failure statistics and patterns");
    println!();
    println!("  Configuration:");
    println!("    agent config             Show all configuration values");
    println!("    agent config max_retry_count 10");
    println!("                             Set max retry attempts to 10");
    println!("    agent config retry_ttl_minutes 60");
    println!("                             Set retry TTL to 60 minutes");
    println!();
    println!("  Status & Metrics:");
    println!("    agent status             Show queue sizes, retry rates, DLQ growth");
    println!();
    println!("CONFIGURATION KEYS:");
    println!("  max_retry_count          Maximum retry attempts before DLQ (default: 10)");
    println!("  retry_ttl_minutes        Retry timeout in minutes (default: 60)");
    println!("  max_concurrent_workers   Max parallel device connections (default: 2)");
    println!();
    println!("UNDERSTANDING THE SYSTEM:");
    println!();
    println!("  Sync Flow:");
    println!("    Device → Agent Queue → Central Server");
    println!("                ↓ (on failure)");
    println!("              Retry (up to max_retry_count or retry_ttl_minutes)");
    println!("                ↓ (on retry exhaustion)");
    println!("           Dead Letter Queue (DLQ)");
    println!();
    println!("  Log States:");
    println!("    • Unsynced (new)       - Logs fetched from device, not yet sent");
    println!("    • In-flight            - Logs currently being synced to server");
    println!("    • Failed (retrying)    - Sync failed, will retry (use 'failed-logs')");
    println!("    • Synced               - Successfully sent to central server");
    println!("    • Dead Letter Queue    - Retry exhausted, investigate (use 'dlq')");
    println!();
    println!("  Why logs end up in DLQ:");
    println!("    1. Max retries exceeded (e.g., invalid data format)");
    println!("    2. TTL timeout (e.g., central server down for >60 min)");
    println!();
    println!("DATABASE QUERIES (Direct Access):");
    println!();
    println!("  Query failed logs (retrying):");
    println!("    SELECT * FROM zkt_raw_attendance");
    println!("    WHERE is_synced = 0 AND updated_at IS NOT NULL;");
    println!();
    println!("  Query dead letter queue:");
    println!("    SELECT * FROM zkt_failed_logs");
    println!("    ORDER BY failed_at DESC LIMIT 50;");
    println!();
    println!("  DLQ statistics by device:");
    println!("    SELECT device_id, COUNT(*) as failures");
    println!("    FROM zkt_failed_logs");
    println!("    GROUP BY device_id ORDER BY failures DESC;");
    println!();
    println!("  DLQ statistics by failure reason:");
    println!("    SELECT failure_reason, COUNT(*) as count");
    println!("    FROM zkt_failed_logs");
    println!("    GROUP BY failure_reason;");
    println!();
    println!("For more information, see documentation at:");
    println!("  ./docs/system-architecture.md");
    println!("  ./plans/reports/implementation-*-retry-limit-*.md");
    println!();
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
        let timestamp_str = chrono::DateTime::from_timestamp(log.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| format!("invalid_{}", log.timestamp));
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

async fn handle_dlq_command(db: &Database, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let limit = if args.len() > 2 {
        args[2].parse::<i64>().unwrap_or(50)
    } else {
        50
    };

    // Query DLQ table
    let dlq_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_failed_logs")
        .fetch_one(db.get_pool())
        .await?;

    type DlqRecord = (i64, String, String, String, Option<i32>, Option<i32>, String, String, i64);
    let dlq_logs: Vec<DlqRecord> = sqlx::query_as(
        "SELECT id, device_id, user_id, timestamp, status, punch, failed_at, failure_reason, retry_attempts
         FROM zkt_failed_logs
         ORDER BY failed_at DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(db.get_pool())
    .await?;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                      DEAD LETTER QUEUE (DLQ) REPORT                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total DLQ entries: {} (showing top {})", dlq_count, dlq_logs.len());
    println!();

    if dlq_logs.is_empty() {
        println!("✅ No logs in dead letter queue. All logs either synced or still retrying.");
        println!();
        return Ok(());
    }

    println!("┌────────┬───────────┬───────────┬──────────────────┬────────┬────────────────────────┐");
    println!("│ ID     │ Device    │ User ID   │ Failed At        │ Retries│ Failure Reason         │");
    println!("├────────┼───────────┼───────────┼──────────────────┼────────┼────────────────────────┤");

    for log in &dlq_logs {
        let (id, device_id, user_id, _timestamp, _status, _punch, failed_at, failure_reason, retry_attempts) = log;

        // Parse failed_at timestamp
        let failed_display = if let Some(pos) = failed_at.find(' ') {
            &failed_at[..std::cmp::min(failed_at.len(), pos + 9)]
        } else {
            &failed_at[..std::cmp::min(failed_at.len(), 16)]
        };

        println!(
            "│ {:6} │ {:9} │ {:9} │ {:16} │ {:6} │ {:22} │",
            id,
            truncate(device_id, 9),
            truncate(user_id, 9),
            truncate(failed_display, 16),
            retry_attempts,
            truncate(failure_reason, 22)
        );
    }

    println!("└────────┴───────────┴───────────┴──────────────────┴────────┴────────────────────────┘");
    println!();
    println!("Legend:");
    println!("  • These logs exceeded retry limits and were moved to DLQ");
    println!("  • They will NOT be retried automatically");
    println!("  • Investigate failure reasons and manually re-queue if needed");
    println!();
    println!("Next steps:");
    println!("  1. Review failure reasons to identify patterns");
    println!("  2. Fix root cause (data format, central server, network)");
    println!("  3. Use 'agent dlq-stats' for detailed analysis");
    println!();

    Ok(())
}

async fn handle_dlq_stats_command(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let total_dlq: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_failed_logs")
        .fetch_one(db.get_pool())
        .await?;

    let by_reason: Vec<(String, i64)> = sqlx::query_as(
        "SELECT failure_reason, COUNT(*) as count
         FROM zkt_failed_logs
         GROUP BY failure_reason
         ORDER BY count DESC"
    )
    .fetch_all(db.get_pool())
    .await?;

    let by_device: Vec<(String, i64)> = sqlx::query_as(
        "SELECT device_id, COUNT(*) as count
         FROM zkt_failed_logs
         GROUP BY device_id
         ORDER BY count DESC
         LIMIT 10"
    )
    .fetch_all(db.get_pool())
    .await?;

    let recent_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_failed_logs WHERE failed_at > datetime('now', '-24 hours')"
    )
    .fetch_one(db.get_pool())
    .await?;

    let recent_1h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_failed_logs WHERE failed_at > datetime('now', '-1 hour')"
    )
    .fetch_one(db.get_pool())
    .await?;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    DEAD LETTER QUEUE STATISTICS                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total DLQ entries: {}", total_dlq);
    println!("  - Last hour: {}", recent_1h);
    println!("  - Last 24 hours: {}", recent_24h);
    println!();

    if total_dlq == 0 {
        println!("✅ No entries in dead letter queue. System healthy!");
        println!();
        return Ok(());
    }

    println!("Failures by Reason:");
    println!("┌─────────────────────────────────────────────┬────────┬─────────┐");
    println!("│ Failure Reason                              │ Count  │ Percent │");
    println!("├─────────────────────────────────────────────┼────────┼─────────┤");

    for (reason, count) in &by_reason {
        let percent = (*count as f64 / total_dlq as f64) * 100.0;
        println!(
            "│ {:43} │ {:6} │ {:6.1}% │",
            truncate(reason, 43),
            count,
            percent
        );
    }
    println!("└─────────────────────────────────────────────┴────────┴─────────┘");
    println!();

    if !by_device.is_empty() {
        println!("Top Failing Devices:");
        println!("┌──────────────────────────┬────────┬─────────┐");
        println!("│ Device ID                │ Count  │ Percent │");
        println!("├──────────────────────────┼────────┼─────────┤");

        for (device, count) in &by_device {
            let percent = (*count as f64 / total_dlq as f64) * 100.0;
            println!(
                "│ {:24} │ {:6} │ {:6.1}% │",
                truncate(device, 24),
                count,
                percent
            );
        }
        println!("└──────────────────────────┴────────┴─────────┘");
        println!();
    }

    // Health assessment
    println!("Health Assessment:");
    if recent_1h > 20 {
        println!("  ⚠️  CRITICAL: {} logs moved to DLQ in last hour", recent_1h);
        println!("      Action: Check central server status immediately");
    } else if recent_24h > 100 {
        println!("  ⚠️  WARNING: {} logs moved to DLQ in last 24h", recent_24h);
        println!("      Action: Investigate failure reasons and trends");
    } else if total_dlq > 1000 {
        println!("  ℹ️  INFO: {} total DLQ entries (consider cleanup)", total_dlq);
        println!("      Action: Archive old entries or investigate patterns");
    } else {
        println!("  ✅ HEALTHY: Low DLQ growth rate");
    }
    println!();

    Ok(())
}

async fn handle_config_command(db: &Database, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() == 2 {
        // Show all config
        let configs: Vec<(String, String)> = sqlx::query_as(
            "SELECT key, value FROM agent_config ORDER BY key"
        )
        .fetch_all(db.get_pool())
        .await?;

        println!();
        println!("╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║                          AGENT CONFIGURATION                                 ║");
        println!("╚══════════════════════════════════════════════════════════════════════════════╝");
        println!();

        if configs.is_empty() {
            println!("No custom configuration set. Using defaults:");
            println!("  max_retry_count      = 10");
            println!("  retry_ttl_minutes    = 60");
            println!("  max_concurrent_workers = 2");
            println!();
        } else {
            println!("Current Configuration:");
            println!("┌────────────────────────────┬─────────────────────────────────┐");
            println!("│ Key                        │ Value                           │");
            println!("├────────────────────────────┼─────────────────────────────────┤");

            for (key, value) in &configs {
                println!(
                    "│ {:26} │ {:31} │",
                    truncate(key, 26),
                    truncate(value, 31)
                );
            }
            println!("└────────────────────────────┴─────────────────────────────────┘");
            println!();
        }
    } else if args.len() == 3 {
        // Get specific key
        let key = &args[2];
        let value = db.get_agent_config(key).await?;

        println!();
        match value {
            Some(v) => println!("{} = {}", key, v),
            None => println!("{} not set (using default)", key),
        }
        println!();
    } else if args.len() == 4 {
        // Set key value
        let key = &args[2];
        let value = &args[3];

        db.upsert_agent_config(key, value).await?;
        println!();
        println!("✅ Configuration updated: {} = {}", key, value);
        println!("   Note: Agent restart may be required for some settings");
        println!();
    } else {
        eprintln!("Usage: agent config [key] [value]");
        return Ok(());
    }

    Ok(())
}

async fn handle_status_command(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let total_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_raw_attendance")
        .fetch_one(db.get_pool())
        .await?;

    let unsynced: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0")
        .fetch_one(db.get_pool())
        .await?;

    let in_flight: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 2")
        .fetch_one(db.get_pool())
        .await?;

    let synced: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 1")
        .fetch_one(db.get_pool())
        .await?;

    let dlq_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_failed_logs")
        .fetch_one(db.get_pool())
        .await?;

    let dlq_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_failed_logs WHERE failed_at > datetime('now', '-24 hours')"
    )
    .fetch_one(db.get_pool())
        .await?;

    // Get retry classification
    let new_logs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0 AND updated_at IS NULL"
    )
    .fetch_one(db.get_pool())
    .await?;

    let retry_logs = unsynced - new_logs;
    let retry_rate = if unsynced > 0 {
        (retry_logs as f64 / unsynced as f64) * 100.0
    } else {
        0.0
    };

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                            AGENT STATUS                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    println!("Sync Queue Status:");
    println!("  Total logs in database:  {}", total_logs);
    println!("  ├─ Synced (✅):           {}", synced);
    println!("  ├─ In-flight (⏳):        {}", in_flight);
    println!("  └─ Unsynced (📤):         {}", unsynced);
    println!("     ├─ New logs:          {} ({:.1}%)", new_logs, if unsynced > 0 { (new_logs as f64 / unsynced as f64) * 100.0 } else { 0.0 });
    println!("     └─ Retrying:          {} ({:.1}%)", retry_logs, retry_rate);
    println!();

    println!("Dead Letter Queue (DLQ):");
    println!("  Total failed:            {}", dlq_total);
    println!("  Last 24 hours:           {}", dlq_24h);
    println!();

    // Health indicator
    println!("System Health:");
    if retry_rate > 70.0 {
        println!("  ⚠️  CRITICAL: High retry rate ({:.1}%)", retry_rate);
        println!("      Action: Check central server connectivity");
    } else if retry_rate > 30.0 {
        println!("  ⚠️  WARNING: Elevated retry rate ({:.1}%)", retry_rate);
        println!("      Action: Monitor central server");
    } else if dlq_24h > 50 {
        println!("  ⚠️  WARNING: {} logs moved to DLQ in 24h", dlq_24h);
        println!("      Action: Review DLQ stats");
    } else {
        println!("  ✅ HEALTHY: System operating normally");
        println!("      Retry rate: {:.1}%", retry_rate);
        println!("      DLQ growth (24h): {} entries", dlq_24h);
    }
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
