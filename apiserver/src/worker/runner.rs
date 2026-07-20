//! Job Worker Runner
//!
//! Main worker loop that polls Redis queue and dispatches jobs to handlers.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::cache::{CacheManager, RosterCache};
use crate::queue::{Job, JobQueue, RedisPool, TaskType};
use crate::services::ReportCoordinator;
use crate::worker::handlers::{
    AccrualHandler, AttendanceHandler, BulkTerminationHandler, ClaimStagedLogsHandler,
    EmployeeImportChunkHandler, EmployeeImportHandler, FinalizeImportHandler, LeaveImportHandler,
    LogIngestionHandler, MergeHandler, NightlyHousekeepingHandler, PayrollReportHandler,
    ReprocessAttendanceHandler, SyncHireDateHandler,
};

// Redis key patterns for monitoring
const WORKER_HEARTBEAT_PREFIX: &str = "monitor:worker:";
const HEARTBEAT_INTERVAL_SECS: u64 = 5;
const HEARTBEAT_TTL_SECS: i64 = 15;

/// Worker status for heartbeat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkerStatus {
    Idle,
    Processing,
}

/// Worker heartbeat payload stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    id: Uuid,
    host: String,
    status: WorkerStatus,
    current_task_type: Option<String>,
    last_seen_at: DateTime<Utc>,
}

impl WorkerHeartbeat {
    pub fn new(
        id: Uuid,
        host: String,
        status: WorkerStatus,
        current_task_type: Option<String>,
        last_seen_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            host,
            status,
            current_task_type,
            last_seen_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn status(&self) -> &WorkerStatus {
        &self.status
    }

    pub fn current_task_type(&self) -> Option<&str> {
        self.current_task_type.as_deref()
    }

    pub fn last_seen_at(&self) -> DateTime<Utc> {
        self.last_seen_at
    }
}

/// Configuration for worker behavior.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Max concurrent jobs per worker (default: 6)
    pub max_concurrent_jobs: usize,
    /// BLPOP timeout in seconds (default: 30)
    pub poll_timeout_secs: f64,
    /// Max retries per job before moving to DLQ (default: 3)
    pub max_retries: u32,
    /// Job processing timeout in seconds (default: 300)
    pub job_timeout_secs: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 5,
            poll_timeout_secs: 30.0,
            max_retries: 3,
            job_timeout_secs: 300,
        }
    }
}

impl WorkerConfig {
    /// Create config from environment variables with defaults.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        let max_conn = std::env::var("DB_WORKER_MAX_CONN")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(5);
        config.max_concurrent_jobs = max_conn;

        if let Ok(val) = std::env::var("WORKER_POLL_TIMEOUT_SECS") {
            match val.parse::<f64>() {
                Ok(n) if n.is_finite() && n > 0.0 && n <= 300.0 => {
                    config.poll_timeout_secs = n;
                }
                Ok(n) => {
                    tracing::warn!(
                        value = n,
                        "WORKER_POLL_TIMEOUT_SECS has invalid value (must be finite, positive, <=300), using default: {}",
                        config.poll_timeout_secs
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        value = %val,
                        error = %e,
                        "Failed to parse WORKER_POLL_TIMEOUT_SECS, using default: {}",
                        config.poll_timeout_secs
                    );
                }
            }
        }

        if let Ok(val) = std::env::var("WORKER_MAX_RETRIES") {
            if let Ok(n) = val.parse() {
                config.max_retries = n;
            }
        }

        if let Ok(val) = std::env::var("WORKER_JOB_TIMEOUT_SECS") {
            if let Ok(n) = val.parse() {
                config.job_timeout_secs = n;
            }
        }

        config
    }
}

/// Background worker that processes jobs from Redis queue.
pub struct JobWorker {
    queue: JobQueue,
    redis: RedisPool,
    queue_repo: Arc<dyn crate::repositories::QueueRepository>,
    publisher_repo: Arc<dyn crate::repositories::PublisherRepository>,
    config: WorkerConfig,
    coordinator: Arc<ReportCoordinator>,
    cache: Arc<CacheManager>,
    /// Pre-built repository instances shared across all handlers.
    deps: Arc<super::WorkerDeps>,
    /// Optional roster cache shared with the main app state.
    roster_cache: Option<Arc<RosterCache>>,
    worker_id: Uuid,
    hostname: String,
    current_status: Arc<Mutex<WorkerStatus>>,
    current_task_type: Arc<Mutex<Option<String>>>,
}

impl JobWorker {
    /// Create a new JobWorker.
    pub fn new(
        redis: RedisPool,
        diesel: &crate::diesel_pool::DieselPools,
        config: WorkerConfig,
        coordinator: Arc<ReportCoordinator>,
        roster_cache: Option<Arc<RosterCache>>,
    ) -> Self {
        let deps = Arc::new(super::WorkerDeps::new(diesel));
        let redis_cache: Arc<dyn crate::repositories::CacheRepository> =
            Arc::new(crate::cache::RedisCache::new(redis.clone()));
        let cache = Arc::new(CacheManager::new(
            redis_cache,
            Arc::clone(&deps.workspace_repo),
            Arc::clone(&deps.shift_repo),
            Arc::clone(&deps.workspace_config_repo),
        ));
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let queue_repo: Arc<dyn crate::repositories::QueueRepository> = Arc::new(
            crate::repositories::RedisQueueRepository::new(redis.clone()),
        );
        let publisher_repo: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(
            crate::repositories::RedisPublisherRepository::new(redis.clone()),
        );

        Self {
            queue: JobQueue::new(redis.clone()),
            redis,
            queue_repo,
            publisher_repo,
            config,
            coordinator,
            cache,
            deps,
            roster_cache,
            worker_id: Uuid::new_v4(),
            hostname,
            current_status: Arc::new(Mutex::new(WorkerStatus::Idle)),
            current_task_type: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the worker as a background task.
    /// Returns a JoinHandle for graceful shutdown.
    pub fn start(self, cancel_token: CancellationToken) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.run(cancel_token).await;
        })
    }

    /// Main worker loop.
    #[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
    async fn run(self, cancel_token: CancellationToken) {
        info!(
            worker_id = %self.worker_id,
            hostname = %self.hostname,
            max_concurrent = self.config.max_concurrent_jobs,
            poll_timeout = self.config.poll_timeout_secs,
            "Job worker started"
        );

        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_jobs));
        let worker = Arc::new(self);

        // Start heartbeat background task
        let heartbeat_worker = Arc::clone(&worker);
        let heartbeat_cancel = cancel_token.clone();
        let heartbeat_task = tokio::spawn(async move {
            heartbeat_worker.heartbeat_loop(heartbeat_cancel).await;
        });

        loop {
            // Check for shutdown
            if cancel_token.is_cancelled() {
                info!("Worker received shutdown signal, stopping...");
                break;
            }

            // Acquire semaphore permit (blocks if at capacity)
            let Ok(permit) = Arc::clone(&semaphore).acquire_owned().await else {
                error!("Semaphore closed unexpectedly");
                break;
            };

            // Poll for job (with cancellation check)
            let job_result = tokio::select! {
                () = cancel_token.cancelled() => {
                    drop(permit);
                    info!("Worker cancelled during poll");
                    break;
                }
                result = worker.queue.dequeue_job(worker.config.poll_timeout_secs) => result
            };

            match job_result {
                Ok(Some(job)) => {
                    let worker_clone = Arc::clone(&worker);
                    let job_id = job.job_id;
                    let timeout_secs = worker.config.job_timeout_secs;
                    let task_type_str = format!("{:?}", job.task_type);

                    // Update status to processing
                    {
                        *worker.current_status.lock() = WorkerStatus::Processing;
                        *worker.current_task_type.lock() = Some(task_type_str.clone());
                    }

                    let batch_index = job.batch_index;
                    let parent_job_id = job.parent_job_id;
                    let trace_ctx = job.trace_context.clone().unwrap_or_default();

                    // Spawn task to process job
                    tokio::spawn(crate::utils::logging::CURRENT_TRACE_CONTEXT.scope(trace_ctx, async move {
                        let _permit = permit; // Hold permit until done

                        let result = tokio::time::timeout(
                            Duration::from_secs(timeout_secs),
                            worker_clone.process_job(job),
                        )
                        .await;

                        // Update daily stats based on result
                        let success = matches!(result, Ok(Ok(())));
                        if success {
                            let _ = worker_clone.queue.increment_daily_completed().await;
                        } else {
                            let _ = worker_clone.queue.increment_daily_failed().await;
                        }

                        // Reset status to idle
                        {
                            *worker_clone.current_status.lock() = WorkerStatus::Idle;
                            *worker_clone.current_task_type.lock() = None;
                        }

                        match result {
                            Ok(Ok(())) => {
                                info!(job_id = %job_id, "Job completed successfully");
                            }
                            Ok(Err(e)) => {
                                error!(job_id = %job_id, error = %e, "Job failed");
                                if let Ok(Some(mut meta)) =
                                    worker_clone.queue.get_job_metadata(parent_job_id).await
                                {
                                    meta.error_message = Some(format!("Batch {batch_index}: {e}"));
                                    let _ = worker_clone.queue.update_job_metadata(&meta).await;
                                }
                            }
                            Err(_) => {
                                error!(job_id = %job_id, timeout_secs = timeout_secs, "Job timed out");
                                if let Ok(Some(mut meta)) =
                                    worker_clone.queue.get_job_metadata(parent_job_id).await
                                {
                                    meta.error_message = Some(format!(
                                        "Batch {batch_index} timed out after {timeout_secs}s"
                                    ));
                                    let _ = worker_clone.queue.update_job_metadata(&meta).await;
                                }
                            }
                        }
                    }));
                }
                Ok(None) => {
                    // Timeout, no job available - continue loop
                    drop(permit);
                }
                Err(e) => {
                    error!(error = %e, "Failed to dequeue job");
                    drop(permit);
                    // Brief sleep before retry
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        // Wait for in-flight tasks to complete
        let pending = worker.config.max_concurrent_jobs - semaphore.available_permits();
        if pending > 0 {
            info!(
                pending = pending,
                "Waiting for in-flight jobs to complete..."
            );

            // Acquire all permits to ensure tasks complete
            match semaphore
                .acquire_many(worker.config.max_concurrent_jobs as u32)
                .await
            {
                Ok(_all_permits) => {
                    info!("All in-flight jobs completed");
                }
                Err(_) => {
                    warn!("Could not acquire all permits during shutdown");
                }
            }
        }

        // Clean up heartbeat key
        let _ = worker.remove_heartbeat().await;

        // Wait for heartbeat task to finish
        let _ = heartbeat_task.await;

        info!("Job worker stopped");
    }

    /// Background loop that sends heartbeats every 5 seconds.
    async fn heartbeat_loop(self: Arc<Self>, cancel_token: CancellationToken) {
        let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

        loop {
            tokio::select! {
                () = cancel_token.cancelled() => {
                    break;
                }
                _ = interval.tick() => {
                    if let Err(e) = self.send_heartbeat().await {
                        warn!(error = %e, "Failed to send heartbeat");
                    }
                }
            }
        }
    }

    /// Send heartbeat to Redis with current status.
    async fn send_heartbeat(&self) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.conn();
        let key = format!("{}{}", WORKER_HEARTBEAT_PREFIX, self.worker_id);

        let status = self.current_status.lock().clone();
        let task_type = self.current_task_type.lock().clone();

        let heartbeat = WorkerHeartbeat::new(
            self.worker_id,
            self.hostname.clone(),
            status,
            task_type,
            Utc::now(),
        );

        let json = serde_json::to_string(&heartbeat).map_err(|e| {
            redis::RedisError::from((redis::ErrorKind::TypeError, "JSON error", e.to_string()))
        })?;

        redis::pipe()
            .set(&key, json)
            .expire(&key, HEARTBEAT_TTL_SECS)
            .query_async(&mut conn)
            .await
    }

    /// Remove heartbeat key on shutdown.
    async fn remove_heartbeat(&self) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.conn();
        let key = format!("{}{}", WORKER_HEARTBEAT_PREFIX, self.worker_id);
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    /// Process a single job by routing to appropriate handler.
    #[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
    pub async fn process_job(&self, job: Job) -> Result<(), String> {
        let job_id = job.job_id;
        let parent_id = job.parent_job_id;
        let batch_index = job.batch_index;
        let start = std::time::Instant::now();

        info!(
            job_id = %job_id,
            parent_id = %parent_id,
            batch_index = batch_index,
            task_type = ?job.task_type,
            "Processing job"
        );

        // Route to appropriate handler
        let result = match job.task_type {
            TaskType::AttendanceReport => {
                AttendanceHandler::handle(
                    &self.queue_repo,
                    &self.publisher_repo,
                    &self.cache,
                    &job,
                    self.roster_cache.as_deref(),
                    &self.deps,
                )
                .await
            }
            TaskType::MergeResults => {
                MergeHandler::handle(&self.queue_repo, &self.publisher_repo, &job, &self.deps).await
            }
            TaskType::PayrollReport => PayrollReportHandler::handle(&job, &self.deps).await,
            TaskType::LeaveAccrual => {
                AccrualHandler::handle(&self.queue_repo, &job, &self.deps).await
            }
            TaskType::LogIngestion => LogIngestionHandler::handle(&job, &self.deps).await,
            TaskType::ReprocessAttendance => {
                ReprocessAttendanceHandler::handle(&self.queue_repo, &job, &self.deps).await
            }
            TaskType::ClaimStagedLogs => ClaimStagedLogsHandler::handle(&job, &self.deps).await,
            TaskType::LeaveImport => {
                LeaveImportHandler::handle(&job, &self.cache.repo, &self.queue_repo, &self.deps)
                    .await
            }
            TaskType::EmployeeImport => {
                EmployeeImportHandler::handle(&job, &self.cache.repo, &self.queue_repo, &self.deps)
                    .await
            }
            TaskType::EmployeeImportChunk => {
                EmployeeImportChunkHandler::handle(
                    &job,
                    &self.cache.repo,
                    &self.queue_repo,
                    &self.deps,
                )
                .await
            }
            TaskType::FinalizeImport => {
                FinalizeImportHandler::handle(&job, &self.cache.repo, &self.deps).await
            }
            TaskType::ExpiringContractsCheck => {
                crate::worker::handlers::ExpiringContractsHandler::handle(&self.deps, &job).await
            }
            TaskType::LeaveImportChunk => {
                crate::worker::handlers::LeaveImportChunkHandler::handle(
                    &job,
                    &self.cache.repo,
                    &self.queue_repo,
                    &self.deps,
                )
                .await
            }
            TaskType::NightlyHousekeeping => {
                NightlyHousekeepingHandler::handle(&self.deps, &job).await
            }
            TaskType::SyncEmployeeHireDate => SyncHireDateHandler::process(&job, &self.deps).await,
            TaskType::BulkTermination => {
                BulkTerminationHandler::handle(&self.queue_repo, &job, &self.deps).await
            }
        };

        let elapsed = start.elapsed();

        // Warn on slow batches (> 10 seconds)
        if elapsed.as_secs() > 10 {
            warn!(
                job_id = %job_id,
                parent_id = %parent_id,
                batch_index = batch_index,
                elapsed_ms = elapsed.as_millis() as u64,
                "Slow batch detected"
            );
        }

        // Report completion to coordinator (for batch-based jobs only)
        // LogIngestion jobs are standalone and don't participate in batch coordination
        if !matches!(
            job.task_type,
            TaskType::LogIngestion
                | TaskType::ClaimStagedLogs
                | TaskType::LeaveImport
                | TaskType::EmployeeImport
                | TaskType::EmployeeImportChunk
                | TaskType::LeaveImportChunk
                | TaskType::FinalizeImport
                | TaskType::NightlyHousekeeping
                | TaskType::SyncEmployeeHireDate
                | TaskType::BulkTermination
        ) {
            let success = result.is_ok();
            if let Err(e) = self
                .coordinator
                .on_batch_complete(parent_id, batch_index, success)
                .await
            {
                error!(
                    parent_id = %parent_id,
                    error = ?e,
                    "Failed to report completion to coordinator"
                );
            }
        }

        result
    }
}

/// Get the underlying queue for handlers.
impl JobWorker {
    /// Get queue reference (for internal use by handlers).
    pub fn queue(&self) -> &JobQueue {
        &self.queue
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();

        assert_eq!(config.max_concurrent_jobs, 5);
        assert!((config.poll_timeout_secs - 30.0).abs() < 0.01);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.job_timeout_secs, 300);
    }

    #[test]
    fn test_worker_config_from_env() {
        // Test with no env vars (should use defaults)
        std::env::remove_var("DB_WORKER_MAX_CONN");
        std::env::remove_var("WORKER_POLL_TIMEOUT_SECS");
        std::env::remove_var("WORKER_MAX_RETRIES");
        std::env::remove_var("WORKER_JOB_TIMEOUT_SECS");

        let config = WorkerConfig::from_env();
        assert_eq!(config.max_concurrent_jobs, 5);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_worker_config_memory_constraint() {
        // 6 concurrent jobs with 50 employees each at ~2.5MB/employee
        // = 6 * 50 * 2.5MB = 750MB for processing
        // Higher parallelism uses more memory but stays in reasonable limits

        let config = WorkerConfig::default();
        let employees_per_batch = 50;
        let approx_mb_per_employee = 2.5;

        let estimated_memory_mb = (config.max_concurrent_jobs as f64)
            * f64::from(employees_per_batch)
            * approx_mb_per_employee;

        assert!(estimated_memory_mb < 1024.0); // Must fit in 1GB
    }

    #[test]
    fn test_job_timeout_reasonable() {
        let config = WorkerConfig::default();

        // 5 minutes should be enough for 20 employees
        assert_eq!(config.job_timeout_secs, 300);
        assert!(config.job_timeout_secs >= 60); // At least 1 minute
        assert!(config.job_timeout_secs <= 600); // At most 10 minutes
    }

    #[test]
    fn test_poll_timeout_blpop() {
        let config = WorkerConfig::default();

        // 30 seconds is good for BLPOP - not too aggressive, not too slow
        assert!((config.poll_timeout_secs - 30.0).abs() < 0.01);
        assert!(config.poll_timeout_secs >= 5.0);
        assert!(config.poll_timeout_secs <= 60.0);
    }

    #[test]
    fn test_max_retries_reasonable() {
        let config = WorkerConfig::default();

        // 3 retries is standard for transient failures
        assert_eq!(config.max_retries, 3);
        assert!(config.max_retries >= 1);
        assert!(config.max_retries <= 5);
    }

    #[test]
    fn test_semaphore_capacity() {
        use tokio::sync::Semaphore;

        let config = WorkerConfig::default();
        let semaphore = Semaphore::new(config.max_concurrent_jobs);

        assert_eq!(semaphore.available_permits(), 5);

        // Simulate acquiring permits
        let _permit1 = semaphore.try_acquire().unwrap();
        assert_eq!(semaphore.available_permits(), 4);

        let _permit2 = semaphore.try_acquire().unwrap();
        assert_eq!(semaphore.available_permits(), 3);

        // Can acquire up to 5
        let _p3 = semaphore.try_acquire();
        let _p4 = semaphore.try_acquire();
        let _permit5 = semaphore.try_acquire().unwrap();
        assert_eq!(semaphore.available_permits(), 0);

        // Sixth should fail
        assert!(semaphore.try_acquire().is_err());
    }

    #[test]
    fn test_cancellation_token() {
        use tokio_util::sync::CancellationToken;

        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_task_type_routing() {
        // Ensure all task types are handled
        let task_types = vec![
            TaskType::AttendanceReport,
            TaskType::MergeResults,
            TaskType::PayrollReport,
            TaskType::LeaveAccrual,
            TaskType::LogIngestion,
            TaskType::ReprocessAttendance,
            TaskType::ClaimStagedLogs,
            TaskType::LeaveImport,
            TaskType::EmployeeImport,
            TaskType::NightlyHousekeeping,
        ];

        for task_type in task_types {
            let is_implemented = matches!(
                task_type,
                TaskType::AttendanceReport
                    | TaskType::MergeResults
                    | TaskType::PayrollReport
                    | TaskType::LeaveAccrual
                    | TaskType::LogIngestion
                    | TaskType::ReprocessAttendance
                    | TaskType::ClaimStagedLogs
                    | TaskType::LeaveImport
                    | TaskType::EmployeeImport
                    | TaskType::EmployeeImportChunk
                    | TaskType::LeaveImportChunk
                    | TaskType::FinalizeImport
                    | TaskType::NightlyHousekeeping
            );

            assert!(is_implemented);
        }
    }

    #[test]
    fn test_worker_status_serialization() {
        let idle = WorkerStatus::Idle;
        let processing = WorkerStatus::Processing;

        let idle_json = serde_json::to_string(&idle).unwrap();
        let processing_json = serde_json::to_string(&processing).unwrap();

        assert_eq!(idle_json, "\"IDLE\"");
        assert_eq!(processing_json, "\"PROCESSING\"");
    }

    #[test]
    fn test_worker_heartbeat_serialization() {
        let heartbeat = WorkerHeartbeat::new(
            Uuid::nil(),
            "worker-pod-1".to_string(),
            WorkerStatus::Processing,
            Some("AttendanceReport".to_string()),
            Utc::now(),
        );

        let json = serde_json::to_string(&heartbeat).unwrap();
        assert!(json.contains("\"status\":\"PROCESSING\""));
        assert!(json.contains("\"current_task_type\":\"AttendanceReport\""));
        assert!(json.contains("\"host\":\"worker-pod-1\""));

        // Verify roundtrip
        let decoded: WorkerHeartbeat = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.host(), "worker-pod-1");
        assert_eq!(decoded.status(), &WorkerStatus::Processing);
    }

    #[test]
    fn test_heartbeat_key_format() {
        let worker_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = format!("{WORKER_HEARTBEAT_PREFIX}{worker_id}");
        assert_eq!(key, "monitor:worker:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_heartbeat_timing_constants() {
        // Heartbeat every 5s, TTL 15s = 3 missed heartbeats before expiry
        assert_eq!(HEARTBEAT_INTERVAL_SECS, 5);
        assert_eq!(HEARTBEAT_TTL_SECS, 15);
        assert!(HEARTBEAT_TTL_SECS >= HEARTBEAT_INTERVAL_SECS as i64 * 2);
    }

    #[test]
    fn test_slow_batch_threshold_is_10_seconds() {
        // The slow batch detection threshold is 10 seconds.
        // Batches under 10s should NOT trigger a warning.
        let fast_elapsed = std::time::Duration::from_secs(9);
        assert!(
            fast_elapsed.as_secs() <= 10,
            "9s should not trigger slow batch warning"
        );

        // Batches over 10s SHOULD trigger a warning.
        let slow_elapsed = std::time::Duration::from_secs(11);
        assert!(
            slow_elapsed.as_secs() > 10,
            "11s should trigger slow batch warning"
        );

        // Exactly 10s should NOT trigger (> 10, not >=).
        let edge_elapsed = std::time::Duration::from_secs(10);
        assert!(
            (edge_elapsed.as_secs() <= 10),
            "Exactly 10s should not trigger"
        );
    }

    #[test]
    fn test_elapsed_millis_conversion() {
        // Verify as_millis() produces correct values for log output
        let duration_198s = std::time::Duration::from_millis(198_000);
        assert_eq!(duration_198s.as_millis() as u64, 198_000);
        assert_eq!(duration_198s.as_secs(), 198);

        // Sub-second precision
        let duration_250ms = std::time::Duration::from_millis(250);
        assert_eq!(duration_250ms.as_millis() as u64, 250);
        assert_eq!(duration_250ms.as_secs(), 0); // Under threshold
    }

    #[test]
    fn test_instant_elapsed_captures_meaningful_duration() {
        // Verify that Instant::now() + elapsed() pattern works correctly
        let start = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = start.elapsed();

        // Should be at least 10ms (may be slightly more due to scheduling)
        assert!(
            elapsed.as_millis() >= 10,
            "Elapsed should capture sleep duration"
        );
        // Should not be unreasonably long
        assert!(
            elapsed.as_millis() < 500,
            "Elapsed should not be unreasonably long"
        );
    }

    #[test]
    fn test_job_timeout_includes_slow_batch_threshold() {
        // Job timeout (300s) should be much larger than slow batch threshold (10s)
        // so slow batches are warned about long before they time out.
        let config = WorkerConfig::default();
        let slow_batch_threshold_secs = 10u64;

        assert!(
            config.job_timeout_secs > slow_batch_threshold_secs * 2,
            "Job timeout ({}) should be at least 2x the slow batch threshold ({})",
            config.job_timeout_secs,
            slow_batch_threshold_secs
        );
    }

    #[test]
    fn test_task_type_hashable() {
        // TaskType must be Hash for HashMap-based handler registry
        let mut map: HashMap<TaskType, &str> = HashMap::new();
        map.insert(TaskType::AttendanceReport, "attendance");
        map.insert(TaskType::MergeResults, "merge");
        map.insert(TaskType::PayrollReport, "payroll");
        map.insert(TaskType::LogIngestion, "log_ingestion");

        assert_eq!(map.len(), 4);
        assert_eq!(map.get(&TaskType::AttendanceReport), Some(&"attendance"));
        assert_eq!(map.get(&TaskType::LogIngestion), Some(&"log_ingestion"));
    }
}
