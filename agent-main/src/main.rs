mod cli;
mod zk_worker;
mod mock_worker;

// use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
// use tokio::process::Command;
use tokio::time::{sleep, Duration};
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
use tokio::sync::{mpsc, Semaphore};
use dashmap::DashMap;

enum WorkerAdapter {
    Zk(Box<zk_worker::ZkWorker>),
    Mock(mock_worker::MockWorker),
}

impl WorkerAdapter {
    async fn call(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            WorkerAdapter::Zk(worker) => {
                match method {
                    "device.connect" => {
                        let ip = params["ip"].as_str().unwrap_or("");
                        let port = params["port"].as_u64().unwrap_or(4370) as u16;
                        worker.connect(ip, port).await.map(|_| serde_json::json!(true)).map_err(|e| e.into())
                    },
                    "device.get_logs" => {
                        worker.get_attendance_logs().await.map(serde_json::Value::Array).map_err(|e| e.into())
                    },
                    "device.get_info" => {
                        worker.get_info().await.map_err(|e| e.into())
                    },
                    _ => Err(format!("Unknown method: {method}").into())
                }
            }
            WorkerAdapter::Mock(worker) => {
                 match method {
                    "device.connect" => {
                        let ip = params["ip"].as_str().unwrap_or("");
                        let port = params["port"].as_u64().unwrap_or(4370) as u16;
                        worker.connect(ip, port).await.map(|_| serde_json::json!(true)).map_err(|e| e.into())
                    },
                    "device.get_logs" => {
                        worker.get_attendance_logs().await.map(serde_json::Value::Array).map_err(|e| e.into())
                    },
                    "device.get_info" => {
                        worker.get_info().await.map_err(|e| e.into())
                    },
                    _ => Err(format!("Unknown method: {method}").into())
                }
            }
        }
    }
}

#[derive(Clone)]
struct WorkerConfig {
    id: String,
    name: String,
    brand: String,
    ip: String,
    port: u32,
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
                return cli::handle_failed_logs_command(&db, &args).await;
            }
            "dlq" | "dead-letter-queue" => {
                return cli::handle_dlq_command(&db, &args).await;
            }
            "dlq-stats" | "stats" => {
                return cli::handle_dlq_stats_command(&db).await;
            }
            "config" => {
                return cli::handle_config_command(&db, &args).await;
            }
            "status" => {
                return cli::handle_status_command(&db).await;
            }
            "--help" | "-h" | "help" => {
                cli::print_help();
                return Ok(());
            }
            "--version" | "-v" => {
                cli::print_version();
                return Ok(());
            }
            _ => {
                eprintln!("❌ Unknown command: {}", args[1]);
                eprintln!();
                cli::print_help();
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
            warn!("Previous session info:\n{info}");
        }
    }

    // Initialize HybridDatabase (memory + disk)
    let hybrid_db = HybridDatabase::new(&config.database_url, 100_000, config.signature_ttl_days)
        .await
        .expect("Failed to initialize HybridDatabase");

    // ONE-TIME FIX: Reset any legacy -1 counts in DB to 0
    let _ = sqlx::query("UPDATE devices SET logs_count = 0 WHERE logs_count = -1").execute(hybrid_db.disk_db().get_pool()).await;
    let _ = sqlx::query("UPDATE devices SET users_count = 0 WHERE users_count = -1").execute(hybrid_db.disk_db().get_pool()).await;

    // Perform crash recovery if needed
    if crash_marker.check_crash() {
        let recovered = recover_from_crash(hybrid_db.disk_db()).await
            .expect("Failed to recover from crash");
        info!("✅ Crash recovery complete: {recovered} logs reset");
    }

    // Write new crash marker
    if let Err(e) = crash_marker.write() {
        warn!("Failed to write crash marker: {e}");
    }

    // Clear startup config: Agent should NOT use stale database config.
    // Server will send ConfigPush after Handshake with fresh configuration.
    if let Err(e) = hybrid_db.disk_db().clear_startup_config().await {
        warn!("Failed to clear startup config: {e}");
    } else {
        info!("🔄 Cleared local config on startup - waiting for ConfigPush from Server...");
    }

    // Use defaults until ConfigPush arrives (database config was just cleared)
    let mut current_max_workers = config.max_concurrent_workers;
    info!("⚙️ Using default Max Workers: {current_max_workers} (until ConfigPush)");

    let command_registry = Arc::new(DashMap::new());
    let (central_client, central_rx, mut reload_rx) = CentralClient::new(
        config.central_server_url.clone(),
        config.agent_id.clone(),
        config.agent_token.clone(),
        hybrid_db.clone(),
        config.inflight_timeout_secs,
        command_registry.clone(),
    );
    
    // Get tx before moving central_client into start task
    // sr_central_tx removed as status reporting task is gone
    let central_client_task = central_client.start(central_rx);
    tokio::spawn(central_client_task);

    // --- DB Writer Worker (Batch Mode) ---
    // Use Arc<AtomicBool> to ensure only one writer starts
    let writer_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (log_tx, mut log_rx) = mpsc::channel::<(String, models::AttendanceData)>(10000);
    // Removed status_tx and status_rx channels for periodic reporting
    let writer_hybrid = hybrid_db.clone();

    // Removed status_reporting_task

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
                                        error!("Failed to save batch to memory: {e}");
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
                                error!("Failed to save timed batch to memory: {e}");
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
                                    info!("💾 Checkpoint: {count} logs persisted to disk");
                                }
                            }
                            Err(e) => {
                                error!("❌ Checkpoint failed: {e}");
                            }
                        }
                    }
                }
                _ = checkpoint_cancel.cancelled() => {
                    info!("💾 Checkpoint Worker: Final flush...");
                    match checkpoint_hybrid.flush_all().await {
                        Ok(count) => info!("✅ Final flush: {count} logs written"),
                        Err(e) => error!("❌ Final flush failed: {e}"),
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
                                info!("🧹 Disk cleanup: removed {deleted} old synced logs (>365 days)");
                            }
                        }
                        Err(e) => {
                            error!("❌ Disk cleanup failed: {e}");
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

    // --- Signature Eviction Worker (TTL-based Memory Cleanup) ---
    let evict_hybrid = hybrid_db.clone();
    let evict_cancel = cancel_token.clone();
    let evict_interval_hours = config.signature_cleanup_hours;
    tokio::spawn(async move {
        info!("🧹 Signature Eviction Worker started (every {evict_interval_hours}h)");

        let evict_interval = Duration::from_secs(evict_interval_hours * 3600);
        let mut evict_timer = tokio::time::interval(evict_interval);

        // Skip immediate tick on startup
        evict_timer.tick().await;

        loop {
            tokio::select! {
                _ = evict_timer.tick() => {
                    let (evicted, before, after) = evict_hybrid.evict_old_signatures();
                    if evicted > 0 {
                        info!("🧹 Signature eviction: removed {evicted} expired signatures ({before} -> {after})");
                    }
                }
                _ = evict_cancel.cancelled() => {
                    info!("🧹 Signature Eviction Worker stopped");
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
                error!("Failed to load devices from DB: {e}");
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
            WorkerConfig {
                id: d.id.clone(),
                name: d.name,
                brand: d.brand,
                ip: d.ip,
                port: d.port as u32,
            }
        }).collect();

        let worker_cancel_token = CancellationToken::new();

        for worker in workers {
            let log_tx = log_tx.clone();
            let hybrid_db = hybrid_db.clone();
            let token = worker_cancel_token.clone();
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

                    let mut adapter: WorkerAdapter;
                    
                    if worker.brand == "mock" {
                        adapter = WorkerAdapter::Mock(mock_worker::MockWorker::new());
                    } else {
                        // Default to ZKTeco
                        adapter = WorkerAdapter::Zk(Box::default());
                    }

                    info!("[{}] Worker ready.", worker.name);
                    stable_since = Some(Instant::now());

                    if let Err(e) = adapter.call("device.connect", json!({"ip": worker.ip, "port": worker.port})).await {
                        error!("[{}] Device connection failed: {}", worker.name, e);
                        retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                        continue;
                    }

                    // Send initial status update immediately
                    if let Ok(info) = adapter.call("device.get_info", json!({})).await {
                        // NEW: Persist to local SQLite
                        if let Some(obj) = info.as_object() {
                            let _ = hybrid_db.disk_db().update_device_metadata(
                                &worker.id,
                                obj.get("mac").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                obj.get("serial").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                obj.get("firmware").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                obj.get("platform").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                obj.get("hw_name").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                obj.get("users_count").and_then(|v| v.as_i64()).unwrap_or(0),
                                obj.get("logs_count").and_then(|v| v.as_i64()).unwrap_or(0),
                                obj.get("users_capacity").and_then(|v| v.as_i64()).unwrap_or(0),
                                obj.get("logs_capacity").and_then(|v| v.as_i64()).unwrap_or(0),
                                obj.get("device_time").and_then(|v| v.as_str()).unwrap_or("N/A")
                            ).await;
                        }

                        // Removed periodic status send
                    }

                    // In-memory duplicate detection: hash-based keys (8 bytes each instead of ~40 for String)
                    // Hash of (user_id, timestamp) — device_id is implicit per worker
                    let mut seen_logs: HashSet<u64> = HashSet::with_capacity(32_000);
                    const SEEN_LOGS_CAP: usize = 100_000;

                    // Adaptive polling: reduce frequency when no new data
                    let mut polling_state = AdaptivePollingState::new();
                    let mut poll_count = 0;

                    while !token.is_cancelled() {
                        // Acquire semaphore permit for this poll cycle (fair round-robin)
                        // Released at end of loop iteration, allowing other devices to poll
                        let _permit = match sem.acquire().await {
                            Ok(p) => p,
                            Err(_) => {
                                error!("[{}] Semaphore closed, stopping worker", worker.name);
                                return;
                            }
                        };

                        poll_count += 1;

                        // Periodically fetch device info (every 5 polls or when just connected)
                        if poll_count % 5 == 1 && let Ok(mut info) = adapter.call("device.get_info", json!({})).await {
                                // NEW: Persist to local SQLite
                                if let Some(obj) = info.as_object() {
                                    let _ = hybrid_db.disk_db().update_device_metadata(
                                        &worker.id,
                                        obj.get("mac").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                        obj.get("serial").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                        obj.get("firmware").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                        obj.get("platform").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                        obj.get("hw_name").and_then(|v| v.as_str()).unwrap_or("N/A"),
                                        obj.get("users_count").and_then(|v| v.as_i64()).unwrap_or(0),
                                        obj.get("logs_count").and_then(|v| v.as_i64()).unwrap_or(0),
                                        obj.get("users_capacity").and_then(|v| v.as_i64()).unwrap_or(0),
                                        obj.get("logs_capacity").and_then(|v| v.as_i64()).unwrap_or(0),
                                        obj.get("device_time").and_then(|v| v.as_str()).unwrap_or("N/A")
                                    ).await;
                                }

                                // ENRICH: If sidecar returns -1 for logs_count, try to get from DB
                                if info["logs_count"].as_i64() == Some(-1) && let Ok(Some(meta)) = hybrid_db.disk_db().get_device_metadata(&worker.id).await {
                                    info["logs_count"] = json!(meta.logs_count.unwrap_or(0));
                                    info["users_count"] = json!(meta.users_count.unwrap_or(0));
                                }

                                // Removed periodic status send
                        }

                        match adapter.call("device.get_logs", json!({})).await {
                            Ok(res) => {
                                // Check stability and reset retry count if stable
                                if let Some(since) = stable_since
                                    && since.elapsed() >= Duration::from_secs(SIDECAR_STABILITY_THRESHOLD_SECS) && retry_count > 0 {
                                        info!("[{}] Connection stable for {}s, resetting retry count", worker.name, SIDECAR_STABILITY_THRESHOLD_SECS);
                                        retry_count = 0;
                                    }

                                let mut new_count = 0;
                                if let Some(logs) = res.as_array() {
                                    let fetched_count = logs.len();
                                    let mut duplicate_count = 0;

                                                                                                            // Update hardware snapshot count in DB (preserve users_count by passing -1)
                                                                                                            let _ = hybrid_db.disk_db().update_device_counts(&worker.id, -1, fetched_count as i64).await;
                                                                        
                                                                                                            // NEW: Trigger immediate status reporting with the fresh count
                                                                                                            let mut quick_info = json!({
                                                                                                                "logs_count": fetched_count,
                                                                                                            });
                                                                                                            // Enrich users_count from DB if possible
                                                                                                            if let Ok(Some(meta)) = hybrid_db.disk_db().get_device_metadata(&worker.id).await {
                                                                                                                quick_info["users_count"] = json!(meta.users_count.unwrap_or(0));
                                                                                                                if let Some(mac) = meta.mac { quick_info["mac"] = json!(mac); }
                                                                                                            }
                                                                        
                                                    // Removed periodic status send
                                                                        
                                                                                                            if !logs.is_empty() {                                                                            // Filter duplicates before sending to queue
                                                                            // Consume array values to avoid clone; use hash-based dedup key
                                                                            let logs_owned: Vec<serde_json::Value> = logs.to_vec();
                                                                            for log_val in logs_owned {
                                                                                if let Ok(attendance) = serde_json::from_value::<models::AttendanceData>(log_val) {
                                                                                    // Hash-based dedup key (8 bytes vs ~40 for String)
                                                                                    let mut hasher = rustc_hash::FxHasher::default();
                                                                                    attendance.user_id.hash(&mut hasher);
                                                                                    attendance.timestamp.hash(&mut hasher);
                                                                                    let key = hasher.finish();
                                    
                                                                                    if seen_logs.insert(key) {
                                                                                        let _ = log_tx.send((worker.id.clone(), attendance)).await;
                                                                                        new_count += 1;
                                                                                    } else {
                                                                                        duplicate_count += 1;
                                                                                    }
                                                                                }
                                                                            }
                                    
                                                                            // Eviction: clear when exceeding cap (secondary dedup; memory store signatures are primary)
                                                                            if seen_logs.len() > SEEN_LOGS_CAP {
                                                                                seen_logs.clear();
                                                                                info!("[{}] Cleared seen_logs cache (>{} entries)", worker.name, SEEN_LOGS_CAP);
                                                                            }
                                    
                                                                            if duplicate_count > 0 {
                                                                                info!("[{}] Fetched {} logs ({} new, {} duplicates filtered) [interval: {}s]",
                                                                                      worker.name, fetched_count, new_count, duplicate_count, polling_state.interval());
                                                                            } else {
                                                                                info!("[{}] Fetched {} logs [interval: {}s]", worker.name, fetched_count, polling_state.interval());
                                                                            }
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

                    if !token.is_cancelled() {
                        // Removed periodic status send

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
                    info!("⚡ Updating Max Workers: {current_max_workers} -> {new_val}");
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
        warn!("Failed to write crash marker: {e}");
    }
    info!("✅ Clean shutdown complete.");

    Ok(())
}
