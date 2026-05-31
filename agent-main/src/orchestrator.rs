use crate::adaptive_polling::AdaptivePollingState;
use crate::central_client::connection::DeviceLocks;
use crate::config::Config;
use crate::crash_recovery::{CrashMarker, recover_from_crash};
use crate::hybrid_database::HybridDatabase;
use crate::mock_worker;
use crate::models;
use crate::retry::{
    SIDECAR_MAX_RETRY_COUNT, SIDECAR_STABILITY_THRESHOLD_SECS, get_sidecar_retry_delay,
};
use crate::utils::update_metadata_from_info;
use crate::worker::Worker;
use crate::worker_processor::WorkerConfig;
use crate::zk_worker;
use dashmap::DashMap;
use log::{debug, error, info, warn};
use rand::Rng;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

/// Shared context passed to every spawned worker.
#[derive(Clone)]
pub struct WorkerContext {
    pub log_tx: mpsc::Sender<(Arc<String>, models::AttendanceData)>,
    pub hybrid_db: HybridDatabase,
    pub semaphore: Arc<Semaphore>,
    pub success_cooldown_secs: Arc<AtomicU64>,
    pub device_locks: DeviceLocks,
    pub time_sync: Arc<crate::time_sync::TimeSync>,
    pub connection_max_lifetime_secs: u64,
    /// Global legacy checksum override for ZK devices
    pub zk_legacy_checksum: Option<bool>,
    /// Timeout (seconds) for ZK device connect + initial info fetch
    pub zk_connect_timeout_secs: u64,
    /// Timeout (seconds) for pulling attendance logs
    pub inflight_timeout_secs: u64,
}

pub struct Orchestrator {
    config: Arc<Config>,
    hybrid_db: HybridDatabase,
    crash_marker: CrashMarker,
    cancel_token: CancellationToken,
}

impl Orchestrator {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let crash_marker = CrashMarker::new(&config.database_url);

        let hybrid_db = HybridDatabase::new(
            &config.database_url,
            config.max_memory_logs,
            config.signature_ttl_days,
        )?;

        Ok(Self {
            config: Arc::new(config),
            hybrid_db,
            crash_marker,
            cancel_token: CancellationToken::new(),
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.show_summary();
        self.handle_crash_recovery()?;
        self.setup_signal_handlers();

        // Retain last known local config on startup to ensure offline resiliency
        info!(
            "🔄 Retaining last known local config on startup - awaiting ConfigPush from Server..."
        );

        let mut current_max_workers = self.config.max_concurrent_workers;
        info!("⚙️ Using default Max Workers: {current_max_workers} (until ConfigPush)");

        // Shared registries
        let command_registry = Arc::new(DashMap::new());
        let device_locks: DeviceLocks = Arc::new(DashMap::new());
        let time_sync = Arc::new(crate::time_sync::TimeSync::new());

        // Setup central client
        let (central_client, central_rx, mut reload_rx) = crate::central_client::CentralClient::new(
            self.config.central_server_url.clone(),
            self.config.agent_id.clone(),
            self.config.agent_token.clone(),
            self.hybrid_db.clone(),
            self.config.inflight_timeout_secs,
            command_registry.clone(),
            device_locks.clone(),
            time_sync.clone(),
        );

        tokio::spawn(central_client.start(central_rx));

        // Start background workers
        let log_tx = self.start_db_writer_worker();
        self.start_disk_cleanup_worker();
        self.start_signature_eviction_worker();

        let mut semaphore = Arc::new(Semaphore::new(current_max_workers));
        let mut worker_registry: HashMap<
            String,
            (WorkerConfig, CancellationToken, JoinHandle<()>),
        > = HashMap::new();

        let mut worker_ctx = WorkerContext {
            log_tx: log_tx.clone(),
            hybrid_db: self.hybrid_db.clone(),
            semaphore: semaphore.clone(),
            success_cooldown_secs: Arc::new(AtomicU64::new(self.config.success_cooldown_secs)),
            device_locks: device_locks.clone(),
            time_sync: time_sync.clone(),
            connection_max_lifetime_secs: self.config.connection_max_lifetime_secs,
            zk_legacy_checksum: self.config.zk_legacy_checksum,
            zk_connect_timeout_secs: self.config.zk_connect_timeout_secs,
            inflight_timeout_secs: self.config.inflight_timeout_secs,
        };

        // Initial device load
        self.load_initial_devices(&mut worker_registry, &worker_ctx, current_max_workers);

        // Core Event Loop
        loop {
            tokio::select! {
                _ = reload_rx.recv() => {
                    info!("🔄 Configuration change detected, applying differential reload...");
                    self.handle_config_reload(&mut current_max_workers, &mut worker_registry, &mut worker_ctx, &mut semaphore).await;
                }
                _ = self.cancel_token.cancelled() => {
                    info!("Stopping orchestrator...");
                    let mut handles = Vec::new();
                    for (_id, (name_config, token, handle)) in worker_registry.drain() {
                        info!("⏹️ [{}] Stopping worker for shutdown", name_config.name);
                        token.cancel();
                        handles.push(handle);
                    }
                    // Wait for all workers to finish (with timeout)
                    let _ = tokio::time::timeout(
                        Duration::from_secs(5),
                        futures::future::join_all(handles),
                    ).await;
                    break;
                }
            }
        }

        info!("⏳ Waiting for shutdown...");
        sleep(Duration::from_secs(1)).await;

        if let Err(e) = self.crash_marker.remove() {
            warn!("Failed to write crash marker: {e}");
        }
        info!("✅ Clean shutdown complete.");

        Ok(())
    }

    fn handle_crash_recovery(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.crash_marker.check_crash() {
            warn!("⚠️ Detected previous crash or unclean shutdown");
            if let Some(info) = self.crash_marker.read_crash_info() {
                warn!("Previous session info:\n{info}");
            }

            let recovered = recover_from_crash(self.hybrid_db.store())?;
            info!("✅ Crash recovery complete: {recovered} logs reset");
        }

        if let Err(e) = self.crash_marker.write() {
            warn!("Failed to write crash marker: {e}");
        }

        Ok(())
    }

    fn setup_signal_handlers(&self) {
        let cloned_token = self.cancel_token.clone();
        tokio::spawn(async move {
            #[allow(clippy::expect_used)]
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to register SIGTERM handler");

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutdown signal received (SIGINT).");
                }
                _ = sigterm.recv() => {
                    info!("Shutdown signal received (SIGTERM).");
                }
            }
            cloned_token.cancel();
        });
    }

    fn start_db_writer_worker(&self) -> mpsc::Sender<(Arc<String>, models::AttendanceData)> {
        let (log_tx, mut log_rx) = mpsc::channel::<(Arc<String>, models::AttendanceData)>(10000);
        let writer_hybrid = self.hybrid_db.clone();

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
                            if let Err(e) = writer_hybrid.save_raw_attendance_batch(to_save).await {
                                error!("Failed to save timed batch: {e}");
                            }
                        }
                    }
                }
            }

            // Flush remaining logs in buffer on shutdown
            if !buffer.is_empty() {
                let to_save = std::mem::take(&mut buffer);
                info!(
                    "💾 DB Writer Worker flushing {} remaining logs...",
                    to_save.len()
                );
                if let Err(e) = writer_hybrid.save_raw_attendance_batch(to_save).await {
                    error!("Failed to save final batch during shutdown: {e}");
                }
            }

            info!("💾 DB Writer Worker stopped");
        });

        log_tx
    }

    fn start_disk_cleanup_worker(&self) {
        let cleanup_hybrid = self.hybrid_db.clone();
        let cleanup_cancel = self.cancel_token.clone();
        tokio::spawn(async move {
            let disk_cleanup_interval = Duration::from_secs(86400);
            let mut disk_cleanup_timer = tokio::time::interval(disk_cleanup_interval);

            disk_cleanup_timer.tick().await;

            loop {
                tokio::select! {
                    _ = disk_cleanup_timer.tick() => {
                        let retention_days = cleanup_hybrid.store()
                            .get_agent_config("retention_days")
                            .ok()
                            .flatten()
                            .and_then(|s| s.parse::<i64>().ok())
                            .unwrap_or(365);

                        info!("🧹 Disk Cleanup running ({retention_days}-day retention)");

                        let cleanup_hybrid_clone = cleanup_hybrid.clone();
                        match tokio::task::spawn_blocking(move || cleanup_hybrid_clone.store().cleanup_old_synced_logs(retention_days)).await.unwrap_or(Err(crate::redb_store::StoreError::Internal("Task paniced".into()))) {
                            Ok(deleted) => {
                                if deleted > 0 {
                                    info!("🧹 Disk cleanup: removed {deleted} old synced logs (>{retention_days} days)");
                                }
                            }
                            Err(e) => {
                                error!("❌ Disk cleanup failed: {e}");
                            }
                        }

                        // Compact database to reclaim freed disk space
                        let compact_hybrid = cleanup_hybrid.clone();
                        match tokio::task::spawn_blocking(move || compact_hybrid.store().compact()).await.unwrap_or(Err(crate::redb_store::StoreError::Internal("Compaction task paniced".into()))) {
                            Ok(compacted) => {
                                if compacted {
                                    info!("🧹 Database compaction complete: reclaimed unused page space");
                                } else {
                                    debug!("🧹 Database compaction finished (no changes needed)");
                                }
                            }
                            Err(e) => {
                                warn!("⚠️ Database compaction failed: {e}");
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
    }

    fn start_signature_eviction_worker(&self) {
        let evict_hybrid = self.hybrid_db.clone();
        let evict_cancel = self.cancel_token.clone();
        let evict_interval_hours = self.config.signature_cleanup_hours;
        tokio::spawn(async move {
            info!("🧹 Signature Eviction Worker started (every {evict_interval_hours}h)");

            let evict_interval = Duration::from_secs(evict_interval_hours * 3600);
            let mut evict_timer = tokio::time::interval(evict_interval);

            evict_timer.tick().await;

            loop {
                tokio::select! {
                    _ = evict_timer.tick() => {
                        let (evicted, before, after) = evict_hybrid.evict_old_signatures().await;
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
    }

    fn load_initial_devices(
        &self,
        worker_registry: &mut HashMap<String, (WorkerConfig, CancellationToken, JoinHandle<()>)>,
        worker_ctx: &WorkerContext,
        current_max_workers: usize,
    ) {
        match self.hybrid_db.store().get_active_devices() {
            Ok(devices) => {
                if devices.is_empty() {
                    warn!("No active devices found in database. Waiting for config push...");
                } else {
                    info!(
                        "📡 Found {} devices, max {} concurrent workers",
                        devices.len(),
                        current_max_workers
                    );
                }
                for d in &devices {
                    let wc = to_worker_config(d);
                    let token = CancellationToken::new();
                    let handle = spawn_worker(wc.clone(), worker_ctx.clone(), token.clone());
                    worker_registry.insert(d.id.clone(), (wc, token, handle));
                }
            }
            Err(e) => {
                error!("Failed to load devices from DB: {e}");
            }
        }
    }

    async fn handle_config_reload(
        &self,
        current_max_workers: &mut usize,
        worker_registry: &mut HashMap<String, (WorkerConfig, CancellationToken, JoinHandle<()>)>,
        worker_ctx: &mut WorkerContext,
        semaphore: &mut Arc<Semaphore>,
    ) {
        // Check if max_workers changed
        if let Ok(Some(s)) = self
            .hybrid_db
            .store()
            .get_agent_config("max_concurrent_workers")
            && let Ok(new_val) = s.parse::<usize>()
            && new_val != *current_max_workers
        {
            info!(
                "⚡ Max Workers changed: {} -> {new_val}. Full restart required.",
                current_max_workers
            );
            *current_max_workers = new_val;
            let mut handles = Vec::new();
            for (id, (_, token, handle)) in worker_registry.drain() {
                info!("⏹️ [{id}] Stopping worker for semaphore resize");
                token.cancel();
                handles.push(handle);
            }
            // Wait for all workers to actually stop before resizing semaphore
            let _ =
                tokio::time::timeout(Duration::from_secs(5), futures::future::join_all(handles))
                    .await;
            *semaphore = Arc::new(Semaphore::new(*current_max_workers));
            worker_ctx.semaphore = semaphore.clone();
        }
        // Check if success_cooldown_secs changed
        if let Ok(Some(s)) = self
            .hybrid_db
            .store()
            .get_agent_config("success_cooldown_secs")
            && let Ok(new_val) = s.parse::<u64>()
            && new_val != worker_ctx.success_cooldown_secs.load(Ordering::Relaxed)
        {
            let old_val = worker_ctx
                .success_cooldown_secs
                .swap(new_val, Ordering::Relaxed);
            info!(
                "⚡ Success Cooldown changed: {old_val}s -> {new_val}s (live-applied to all workers)"
            );
        }

        // Load new device list
        let new_devices = match self.hybrid_db.store().get_active_devices() {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to load devices: {e}");
                return;
            }
        };

        let new_configs: HashMap<String, WorkerConfig> = new_devices
            .iter()
            .map(|d| (d.id.clone(), to_worker_config(d)))
            .collect();

        // 1. Remove workers for devices no longer active
        let old_ids: Vec<String> = worker_registry.keys().cloned().collect();
        for id in &old_ids {
            if !new_configs.contains_key(id)
                && let Some((wc, token, handle)) = worker_registry.remove(id)
            {
                info!("⏹️ [{}] Device removed/disabled, stopping worker", wc.name);
                token.cancel();
                // Wait for worker to fully stop before proceeding
                let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
            }
        }

        // 2. Collect diffs
        let mut to_restart: Vec<String> = Vec::new();
        let mut to_add: Vec<String> = Vec::new();

        for (id, new_config) in &new_configs {
            if let Some((old_config, _, _)) = worker_registry.get(id) {
                if old_config.connection_changed(new_config) {
                    to_restart.push(id.clone());
                }
            } else {
                to_add.push(id.clone());
            }
        }

        // Apply: restart changed workers — await old worker shutdown before spawning new
        for id in &to_restart {
            if let Some((_, old_token, old_handle)) = worker_registry.remove(id) {
                old_token.cancel();
                let _ = tokio::time::timeout(Duration::from_secs(5), old_handle).await;
            }
            let new_config = &new_configs[id];
            info!(
                "🔄 [{}] Config changed (ip/port/brand/password), restarting worker",
                new_config.name
            );
            let token = CancellationToken::new();
            let handle = spawn_worker(new_config.clone(), worker_ctx.clone(), token.clone());
            worker_registry.insert(id.clone(), (new_config.clone(), token, handle));
        }

        // Apply: spawn new devices
        for id in &to_add {
            let new_config = &new_configs[id];
            info!(
                "➕ [{}] New device detected, spawning worker ({})",
                new_config.name, new_config.ip
            );
            let token = CancellationToken::new();
            let handle = spawn_worker(new_config.clone(), worker_ctx.clone(), token.clone());
            worker_registry.insert(id.clone(), (new_config.clone(), token, handle));
        }

        info!("📡 Active workers: {}", worker_registry.len());
    }
}

// Helper: build WorkerConfig from StoredDevice
fn to_worker_config(d: &crate::redb_store::StoredDevice) -> WorkerConfig {
    WorkerConfig {
        id: d.id.clone(),
        name: d.name.clone(),
        brand: d.brand.clone(),
        ip: d.ip.clone(),
        port: d.port as u32,
        password: d.password.as_ref().and_then(|p| p.parse::<u32>().ok()),
        use_legacy_checksum: d.use_legacy_checksum, // Smart checksum: persisted from last auto-detect
    }
}

/// Maximum poll interval to stay below typical ZK device UDP session timeout (~60s).
const SESSION_SAFE_INTERVAL_SECS: u64 = 45;

pub fn spawn_worker(
    config: WorkerConfig,
    ctx: WorkerContext,
    token: CancellationToken,
) -> JoinHandle<()> {
    let worker_id = Arc::new(config.id.clone());
    let log_tx = ctx.log_tx;
    let hybrid_db = ctx.hybrid_db;
    let sem = ctx.semaphore;
    let success_cooldown = ctx.success_cooldown_secs;
    let device_locks = ctx.device_locks;
    let connection_max_lifetime = ctx.connection_max_lifetime_secs;
    let zk_legacy_checksum = ctx.zk_legacy_checksum;
    let zk_connect_timeout_secs = ctx.zk_connect_timeout_secs;

    tokio::spawn(async move {
        info!(
            "[{}] Device Polling Loop started ({})",
            config.name, config.ip
        );

        // Stagger startup: random delay 0-2s per worker to prevent thundering herd
        let stagger = Duration::from_millis(rand::thread_rng().gen_range(0..2000));
        debug!("[{}] Stagger delay: {:?}", config.name, stagger);
        sleep(stagger).await;

        let mut retry_count: u32 = 0;
        #[allow(unused_assignments)]
        let mut stable_since: Option<Instant> = None;

        // Reconnect frequency monitoring: track timestamps of recent reconnects
        let mut reconnect_timestamps: Vec<Instant> = Vec::new();
        const RECONNECT_WINDOW_SECS: u64 = 3600; // 1 hour
        const RECONNECT_WARN_THRESHOLD: usize = 5;

        let mut seen_logs: HashSet<u64> = HashSet::with_capacity(100_000);
        const SEEN_LOGS_CAP: usize = 200_000;

        while !token.is_cancelled() {
            if retry_count > 0 {
                let delay = get_sidecar_retry_delay(retry_count);
                warn!(
                    "[{}] Retry {}/{}, waiting {:?}...",
                    config.name, retry_count, SIDECAR_MAX_RETRY_COUNT, delay
                );
                sleep(delay).await;
            }

            // Acquire semaphore BEFORE connecting to limit concurrent device access.
            let connect_permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => {
                    error!(
                        "[{}] Semaphore closed during connect phase, stopping worker",
                        config.name
                    );
                    return;
                }
            };

            // Device lock covers connect + initial get_info to prevent
            // commands from racing with the connection setup phase.
            let connect_device_lock = device_locks
                .entry(config.id.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone();
            let connect_device_guard = connect_device_lock.lock().await;

            let mut worker_impl: Box<dyn Worker> = if config.brand == "mock" {
                Box::new(mock_worker::MockWorker::new())
            } else {
                // Apply global legacy checksum if set, otherwise use per-device config
                let checksum = config.use_legacy_checksum.or(zk_legacy_checksum);
                Box::new(zk_worker::ZkWorker::new(
                    config.password,
                    checksum,
                    zk_connect_timeout_secs,
                    ctx.inflight_timeout_secs,
                ))
            };

            info!("[{}] Worker ready (permit acquired).", config.name);
            stable_since = Some(Instant::now());

            if let Err(e) = worker_impl.connect(&config.ip, config.port as u16).await {
                error!("[{}] Device connection failed: {}", config.name, e);
                drop(connect_device_guard);
                drop(connect_permit);
                retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                continue;
            }

            // Smart checksum memory: persist detected mode for future connects
            if let Some(detected) = worker_impl.detected_checksum_mode() {
                if let Err(e) = hybrid_db
                    .store()
                    .update_device_checksum(&config.id, detected)
                {
                    warn!(
                        "[{}] Failed to persist detected checksum: {}",
                        config.name, e
                    );
                } else {
                    debug!(
                        "[{}] Persisted detected checksum: {}",
                        config.name,
                        if detected { "legacy" } else { "modern" }
                    );
                }
            }

            match worker_impl.get_info(ctx.time_sync.as_ref()).await {
                Ok(info) => {
                    update_metadata_from_info(
                        hybrid_db.store(),
                        &config.id,
                        &info,
                        ctx.time_sync.corrected_timestamp(),
                    );
                }
                Err(e) => {
                    warn!(
                        "[{}] Initial get_info failed: {}, disconnecting and retrying...",
                        config.name, e
                    );
                    worker_impl.disconnect().await;
                    drop(connect_device_guard);
                    drop(connect_permit);
                    retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);
                    continue;
                }
            }

            // Connection + initial get_info succeeded — release connect-phase guards.
            let connected_at = Instant::now();
            drop(connect_device_guard);
            drop(connect_permit);

            // Note: seen_logs cache is now persisted across reconnections

            let mut polling_state =
                AdaptivePollingState::with_cooldown(success_cooldown.load(Ordering::Relaxed));
            let mut poll_count = 0;
            let mut proactive_cooldown = false;

            while !token.is_cancelled() {
                // Sync live cooldown value (config pushes update the AtomicU64)
                polling_state.set_cooldown(success_cooldown.load(Ordering::Relaxed));

                // Check connection lifetime — force reconnect if exceeded
                if connected_at.elapsed() >= Duration::from_secs(connection_max_lifetime) {
                    info!(
                        "[{}] Connection lifetime exceeded ({}s), reconnecting...",
                        config.name, connection_max_lifetime
                    );
                    break;
                }

                let permit = match sem.acquire().await {
                    Ok(p) => p,
                    Err(_) => {
                        error!("[{}] Semaphore closed, stopping worker", config.name);
                        return;
                    }
                };

                let device_lock = device_locks
                    .entry(config.id.clone())
                    .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                    .clone();
                let device_guard = device_lock.lock().await;

                poll_count += 1;

                if poll_count % 5 == 1 {
                    match worker_impl.get_info(ctx.time_sync.as_ref()).await {
                        Ok(mut info) => {
                            update_metadata_from_info(
                                hybrid_db.store(),
                                &config.id,
                                &info,
                                ctx.time_sync.corrected_timestamp(),
                            );

                            if info["logs_count"].as_i64() == Some(-1)
                                && let Ok(Some(meta)) =
                                    hybrid_db.store().get_device_metadata(&config.id)
                            {
                                info["logs_count"] = json!(meta.logs_count.unwrap_or(0));
                                info["users_count"] = json!(meta.users_count.unwrap_or(0));
                            }
                        }
                        Err(e) => {
                            error!(
                                "[{}] Periodic get_info failed: {}, disconnecting...",
                                config.name, e
                            );
                            break;
                        }
                    }
                }

                // Heartbeat: lightweight ping before heavy fetch to detect silent UDP loss
                if !worker_impl.ping().await {
                    warn!(
                        "[{}] Heartbeat ping failed, connection lost — reconnecting...",
                        config.name
                    );
                    break;
                }

                match worker_impl.get_attendance_logs().await {
                    Ok(logs_vec) => {
                        if let Some(since) = stable_since
                            && since.elapsed()
                                >= Duration::from_secs(SIDECAR_STABILITY_THRESHOLD_SECS)
                            && retry_count > 0
                        {
                            info!(
                                "[{}] Connection stable for {}s, resetting retry count",
                                config.name, SIDECAR_STABILITY_THRESHOLD_SECS
                            );
                            retry_count = 0;
                        }

                        let mut new_count = 0;
                        {
                            let fetched_count = logs_vec.len();
                            let mut duplicate_count = 0;

                            let _ = hybrid_db.store().update_device_counts(
                                &config.id,
                                -1,
                                fetched_count as i64,
                                ctx.time_sync.corrected_timestamp(),
                            );

                            #[allow(unused_variables)]
                            let mut quick_info = json!({
                                "logs_count": fetched_count,
                            });
                            if let Ok(Some(meta)) =
                                hybrid_db.store().get_device_metadata(&config.id)
                            {
                                quick_info["users_count"] = json!(meta.users_count.unwrap_or(0));
                                if let Some(mac) = meta.mac {
                                    quick_info["mac"] = json!(mac);
                                }
                            }

                            if !logs_vec.is_empty() {
                                for attendance in logs_vec {
                                    let mut hasher = rustc_hash::FxHasher::default();
                                    attendance.user_id.hash(&mut hasher);
                                    attendance.timestamp.hash(&mut hasher);
                                    let key = hasher.finish();

                                    if seen_logs.insert(key) {
                                        if let Err(e) =
                                            log_tx.send((worker_id.clone(), attendance)).await
                                        {
                                            error!(
                                                "[{}] Failed to send attendance log: {}",
                                                config.name, e
                                            );
                                        }
                                        new_count += 1;
                                    } else {
                                        duplicate_count += 1;
                                    }
                                }

                                if seen_logs.len() > SEEN_LOGS_CAP {
                                    seen_logs.clear();
                                    seen_logs.shrink_to(100_000);
                                    info!(
                                        "[{}] Cleared seen_logs cache (>{} entries)",
                                        config.name, SEEN_LOGS_CAP
                                    );
                                }

                                if duplicate_count > 0 {
                                    info!(
                                        "[{}] Fetched {} logs ({} new, {} duplicates filtered) [interval: {}s]",
                                        config.name,
                                        fetched_count,
                                        new_count,
                                        duplicate_count,
                                        polling_state.interval()
                                    );
                                } else {
                                    info!(
                                        "[{}] Fetched {} logs [interval: {}s]",
                                        config.name,
                                        fetched_count,
                                        polling_state.interval()
                                    );
                                }
                            }
                        }

                        let (new_interval, changed) = polling_state.update(new_count > 0);
                        if changed && new_count == 0 {
                            debug!(
                                "[{}] No new data for {} polls, increasing interval: {}s",
                                config.name, polling_state.consecutive_no_new_data, new_interval
                            );
                        }

                        // Proactive disconnect: if cooldown interval exceeds session-safe
                        // limit, disconnect device to free connection slot, sleep, then
                        // reconnect via outer loop.
                        if polling_state.needs_disconnect() {
                            info!(
                                "[{}] Cooldown {}s, disconnecting to free device slot...",
                                config.name, new_interval
                            );
                            drop(device_guard);
                            drop(permit);
                            worker_impl.disconnect().await;
                            tokio::select! {
                                _ = token.cancelled() => { return; }
                                _ = sleep(Duration::from_secs(new_interval)) => {}
                            }
                            // Reset interval to base for fresh reconnect
                            polling_state.current_interval_secs =
                                crate::adaptive_polling::BASE_POLL_INTERVAL_SECS;
                            polling_state.consecutive_no_new_data = 0;
                            proactive_cooldown = true;
                            break; // → outer loop reconnects cleanly
                        }
                    }
                    Err(e) => {
                        error!("[{}] Polling error: {}, disconnecting...", config.name, e);
                        break;
                    }
                }

                drop(device_guard);
                drop(permit);
                let capped_interval =
                    std::cmp::min(polling_state.interval(), SESSION_SAFE_INTERVAL_SECS);
                // Cancel-aware sleep: respond to shutdown immediately instead of
                // sleeping the full interval while new worker is waiting
                tokio::select! {
                    _ = token.cancelled() => { break; }
                    _ = sleep(Duration::from_secs(capped_interval)) => {}
                }
            }

            // Graceful disconnect — skip if proactive cooldown already disconnected
            if !proactive_cooldown {
                worker_impl.disconnect().await;
            }

            if !token.is_cancelled() && !proactive_cooldown {
                warn!("[{}] Sidecar connection lost, restarting...", config.name);
                retry_count = std::cmp::min(retry_count + 1, SIDECAR_MAX_RETRY_COUNT);

                // Reconnect frequency monitoring
                let now = Instant::now();
                reconnect_timestamps
                    .retain(|ts| now.duration_since(*ts).as_secs() < RECONNECT_WINDOW_SECS);
                reconnect_timestamps.push(now);
                if reconnect_timestamps.len() >= RECONNECT_WARN_THRESHOLD {
                    warn!(
                        "⚠️ [{}] High reconnect frequency: {} reconnects in the last hour — possible firmware or network issue",
                        config.name,
                        reconnect_timestamps.len()
                    );
                }
            } else if proactive_cooldown {
                info!("[{}] Cooldown complete, reconnecting...", config.name);
            }
        }

        info!("[{}] Worker stopped", config.name);
    })
}
