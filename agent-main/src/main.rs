mod cli;

use std::sync::Arc;
use std::time::Instant;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use tokio::time::{sleep, Duration};
use agent::hybrid_database::HybridDatabase;
use agent::models;
use agent::retry::{get_sidecar_retry_delay, SIDECAR_MAX_RETRY_COUNT, SIDECAR_STABILITY_THRESHOLD_SECS};
use agent::crash_recovery::{CrashMarker, recover_from_crash};
use agent::adaptive_polling::AdaptivePollingState;
use serde_json::json;
use log::{info, error, warn};
use tokio_util::sync::CancellationToken;
use tokio::sync::{mpsc, Semaphore};
use dashmap::DashMap;
use agent::worker::Worker;
use agent::zk_worker;
use agent::mock_worker;

#[derive(Clone)]
struct WorkerConfig {
    id: Arc<String>,
    name: String,
    brand: String,
    ip: String,
    port: u32,
    password: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Check for CLI commands
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let config = agent::config::Config::load();
        let hybrid_db = HybridDatabase::new(&config.database_url, 1000, 90)
            .expect("Failed to initialize database");

        match args[1].as_str() {
            "failed-logs" | "failed" => {
                return cli::handle_failed_logs_command(hybrid_db.store(), &args);
            }
            "dlq" | "dead-letter-queue" => {
                return cli::handle_dlq_command(hybrid_db.store(), &args);
            }
            "dlq-stats" | "stats" => {
                return cli::handle_dlq_stats_command(hybrid_db.store());
            }
            "config" => {
                return cli::handle_config_command(hybrid_db.store(), &args);
            }
            "status" => {
                return cli::handle_status_command(hybrid_db.store());
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

    // Initialize HybridDatabase (Redb-backed)
    let hybrid_db = HybridDatabase::new(&config.database_url, config.max_memory_logs, config.signature_ttl_days)
        .expect("Failed to initialize HybridDatabase");

    // Perform crash recovery if needed
    if crash_marker.check_crash() {
        let recovered = recover_from_crash(hybrid_db.store())
            .expect("Failed to recover from crash");
        info!("✅ Crash recovery complete: {recovered} logs reset");
    }

    // Write new crash marker
    if let Err(e) = crash_marker.write() {
        warn!("Failed to write crash marker: {e}");
    }

    // Clear startup config: Agent should NOT use stale database config.
    // Server will send ConfigPush after Handshake with fresh configuration.
    if let Err(e) = hybrid_db.store().clear_startup_config() {
        warn!("Failed to clear startup config: {e}");
    } else {
        info!("🔄 Cleared local config on startup - waiting for ConfigPush from Server...");
    }

    // Use defaults until ConfigPush arrives (database config was just cleared)
    let mut current_max_workers = config.max_concurrent_workers;
    info!("⚙️ Using default Max Workers: {current_max_workers} (until ConfigPush)");

    let command_registry = Arc::new(DashMap::new());
    let (central_client, central_rx, mut reload_rx) = agent::central_client::CentralClient::new(
        config.central_server_url.clone(),
        config.agent_id.clone(),
        config.agent_token.clone(),
        hybrid_db.clone(),
        config.inflight_timeout_secs,
        command_registry.clone(),
    );

    let central_client_task = central_client.start(central_rx);
    tokio::spawn(central_client_task);

    // --- DB Writer Worker (Batch Mode) ---
    let writer_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (log_tx, mut log_rx) = mpsc::channel::<(Arc<String>, models::AttendanceData)>(10000);
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
                                    if let Err(e) = writer_hybrid.save_raw_attendance_batch(to_save) {
                                        error!("Failed to save batch: {e}");
                                    }
                                    sleep(Duration::from_millis(10)).await;
                                }
                            }
                            None => break,
                        }
                    }
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            let to_save = std::mem::replace(&mut buffer, Vec::with_capacity(100));
                            if let Err(e) = writer_hybrid.save_raw_attendance_batch(to_save) {
                                error!("Failed to save timed batch: {e}");
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

    // NOTE: Checkpoint Worker REMOVED — Redb auto-persists via copy-on-write

    // --- Periodic Disk Cleanup Task (Old Logs >365 Days) ---
    let cleanup_hybrid = hybrid_db.clone();
    let cleanup_cancel = cancel_token.clone();
    tokio::spawn(async move {
        info!("🧹 Disk Cleanup Worker started (365-day retention)");

        let disk_cleanup_interval = Duration::from_secs(86400);
        let mut disk_cleanup_timer = tokio::time::interval(disk_cleanup_interval);

        // Skip immediate tick on startup
        disk_cleanup_timer.tick().await;

        loop {
            tokio::select! {
                _ = disk_cleanup_timer.tick() => {
                    match cleanup_hybrid.store().cleanup_old_synced_logs(365) {
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

        let db_devices = match hybrid_db.store().get_active_devices() {
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

        // Runtime optimization: Load signatures for active devices
        for d in &db_devices {
            if let Err(e) = hybrid_db.load_device_memory(&d.id) {
                error!("Failed to load memory for device {}: {}", d.id, e);
            }
        }

        let workers: Vec<WorkerConfig> = db_devices.into_iter().map(|d| {
            WorkerConfig {
                id: Arc::new(d.id.clone()),
                name: d.name,
                brand: d.brand,
                ip: d.ip,
                port: d.port as u32,
                password: d.password.and_then(|p| p.parse::<u32>().ok()),
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
                    if retry_count > 0 {
                        let delay = get_sidecar_retry_delay(retry_count);
                        warn!("[{}] Retry {}/{}, waiting {:?}...", worker.name, retry_count, SIDECAR_MAX_RETRY_COUNT, delay);
                        sleep(delay).await;
                    }

                    let mut worker_impl: Box<dyn Worker> = if worker.brand == "mock" {
                        Box::new(mock_worker::MockWorker::new())
                    } else {
                        Box::new(zk_worker::ZkWorker::new(worker.password))
                    };

                    info!("[{}] Worker ready.", worker.name);
                    stable_since = Some(Instant::now());

                    if let Err(e) = worker_impl.connect(&worker.ip, worker.port as u16).await {
                        error!("[{}] Device connection failed: {}", worker.name, e);
                        retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                        continue;
                    }

                    if let Ok(info) = worker_impl.get_info().await
                        && let Some(obj) = info.as_object() {
                            let _ = hybrid_db.store().update_device_metadata(
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
                            );
                        }

                    let mut seen_logs: HashSet<u64> = HashSet::with_capacity(16_000);
                    const SEEN_LOGS_CAP: usize = 50_000;

                    let mut polling_state = AdaptivePollingState::new();
                    let mut poll_count = 0;

                    while !token.is_cancelled() {
                        let _permit = match sem.acquire().await {
                            Ok(p) => p,
                            Err(_) => {
                                error!("[{}] Semaphore closed, stopping worker", worker.name);
                                return;
                            }
                        };

                        poll_count += 1;

                        if poll_count % 5 == 1 && let Ok(mut info) = worker_impl.get_info().await {
                                if let Some(obj) = info.as_object() {
                                    let _ = hybrid_db.store().update_device_metadata(
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
                                    );
                                }

                                if info["logs_count"].as_i64() == Some(-1) && let Ok(Some(meta)) = hybrid_db.store().get_device_metadata(&worker.id) {
                                    info["logs_count"] = json!(meta.logs_count.unwrap_or(0));
                                    info["users_count"] = json!(meta.users_count.unwrap_or(0));
                                }
                        }

                        match worker_impl.get_attendance_logs().await {
                            Ok(logs_vec) => {
                                if let Some(since) = stable_since
                                    && since.elapsed() >= Duration::from_secs(SIDECAR_STABILITY_THRESHOLD_SECS) && retry_count > 0 {
                                        info!("[{}] Connection stable for {}s, resetting retry count", worker.name, SIDECAR_STABILITY_THRESHOLD_SECS);
                                        retry_count = 0;
                                    }

                                let mut new_count = 0;
                                {
                                    let fetched_count = logs_vec.len();
                                    let mut duplicate_count = 0;

                                    let _ = hybrid_db.store().update_device_counts(&worker.id, -1, fetched_count as i64);

                                    #[allow(unused_variables)]
                                    let mut quick_info = json!({
                                        "logs_count": fetched_count,
                                    });
                                    if let Ok(Some(meta)) = hybrid_db.store().get_device_metadata(&worker.id) {
                                        quick_info["users_count"] = json!(meta.users_count.unwrap_or(0));
                                        if let Some(mac) = meta.mac { quick_info["mac"] = json!(mac); }
                                    }

                                    if !logs_vec.is_empty() {
                                        for attendance in logs_vec {
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

                                        if seen_logs.len() > SEEN_LOGS_CAP {
                                            seen_logs.clear();
                                            seen_logs.shrink_to(16_000);
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

                        drop(_permit);
                        sleep(Duration::from_secs(polling_state.interval())).await;
                    }

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

                if let Ok(Some(s)) = hybrid_db.store().get_agent_config("max_concurrent_workers")
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

    // No checkpoint needed — Redb auto-persists
    info!("⏳ Waiting for shutdown...");
    sleep(Duration::from_secs(1)).await;

    // Remove crash marker on clean exit
    if let Err(e) = crash_marker.remove() {
        warn!("Failed to write crash marker: {e}");
    }
    info!("✅ Clean shutdown complete.");

    Ok(())
}
