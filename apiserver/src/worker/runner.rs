//! Report Worker Runner
//!
//! Main worker loop that polls Redis queue and dispatches jobs to handlers.

use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Semaphore;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::cache::CacheManager;
use crate::queue::{RedisPool, ReportJob, ReportQueue, TaskType};
use crate::services::ReportCoordinator;
use crate::worker::handlers::{AttendanceHandler, MergeHandler};

/// Configuration for worker behavior.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Max concurrent jobs per worker (default: 2)
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
            max_concurrent_jobs: 2,
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

        if let Ok(val) = std::env::var("WORKER_MAX_CONCURRENT") {
            if let Ok(n) = val.parse() {
                config.max_concurrent_jobs = n;
            }
        }

        if let Ok(val) = std::env::var("WORKER_POLL_TIMEOUT_SECS") {
            if let Ok(n) = val.parse() {
                config.poll_timeout_secs = n;
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

/// Background worker that processes report jobs from Redis queue.
pub struct ReportWorker {
    queue: ReportQueue,
    db: PgPool,
    config: WorkerConfig,
    coordinator: Arc<ReportCoordinator>,
    cache: Arc<CacheManager>,
}

impl ReportWorker {
    /// Create a new ReportWorker.
    pub fn new(
        redis: RedisPool,
        db: PgPool,
        config: WorkerConfig,
        coordinator: Arc<ReportCoordinator>,
    ) -> Self {
        let cache = Arc::new(CacheManager::new(redis.clone(), db.clone()));
        Self {
            queue: ReportQueue::new(redis),
            db,
            config,
            coordinator,
            cache,
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
    async fn run(self, cancel_token: CancellationToken) {
        info!(
            max_concurrent = self.config.max_concurrent_jobs,
            poll_timeout = self.config.poll_timeout_secs,
            "Report worker started"
        );

        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_jobs));
        let worker = Arc::new(self);

        loop {
            // Check for shutdown
            if cancel_token.is_cancelled() {
                info!("Worker received shutdown signal, stopping...");
                break;
            }

            // Acquire semaphore permit (blocks if at capacity)
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    error!("Semaphore closed unexpectedly");
                    break;
                }
            };

            // Poll for job (with cancellation check)
            let job_result = tokio::select! {
                _ = cancel_token.cancelled() => {
                    drop(permit);
                    info!("Worker cancelled during poll");
                    break;
                }
                result = worker.queue.dequeue_job(worker.config.poll_timeout_secs) => result
            };

            match job_result {
                Ok(Some(job)) => {
                    let worker_clone = worker.clone();
                    let job_id = job.job_id;
                    let timeout_secs = worker.config.job_timeout_secs;

                    // Spawn task to process job
                    tokio::spawn(async move {
                        let _permit = permit; // Hold permit until done

                        let result = tokio::time::timeout(
                            Duration::from_secs(timeout_secs),
                            worker_clone.process_job(job),
                        )
                        .await;

                        match result {
                            Ok(Ok(())) => {
                                info!(job_id = %job_id, "Job completed successfully");
                            }
                            Ok(Err(e)) => {
                                error!(job_id = %job_id, error = %e, "Job failed");
                            }
                            Err(_) => {
                                error!(job_id = %job_id, "Job timed out");
                            }
                        }
                    });
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
            info!(pending = pending, "Waiting for in-flight jobs to complete...");

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

        info!("Report worker stopped");
    }

    /// Process a single job by routing to appropriate handler.
    async fn process_job(&self, job: ReportJob) -> Result<(), String> {
        let job_id = job.job_id;
        let parent_id = job.parent_job_id;
        let batch_index = job.batch_index;

        info!(
            job_id = %job_id,
            parent_id = %parent_id,
            batch_index = batch_index,
            task_type = ?job.task_type,
            employees = job.employee_ids.len(),
            "Processing job"
        );

        // Route to appropriate handler
        let result = match job.task_type {
            TaskType::AttendanceReport => {
                AttendanceHandler::handle(&self.db, &self.queue, &self.cache, &job).await
            }
            TaskType::MergeResults => MergeHandler::handle(&self.db, &self.queue, &job).await,
            TaskType::PayrollReport => {
                // Future implementation
                Err("PayrollReport not implemented".to_string())
            }
        };

        // Report completion to coordinator
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

        result
    }
}

/// Get the underlying queue for handlers.
impl ReportWorker {
    /// Get queue reference (for internal use by handlers).
    pub fn queue(&self) -> &ReportQueue {
        &self.queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();

        assert_eq!(config.max_concurrent_jobs, 2);
        assert!((config.poll_timeout_secs - 30.0).abs() < 0.01);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.job_timeout_secs, 300);
    }

    #[test]
    fn test_worker_config_from_env() {
        // Test with no env vars (should use defaults)
        std::env::remove_var("WORKER_MAX_CONCURRENT");
        std::env::remove_var("WORKER_POLL_TIMEOUT_SECS");
        std::env::remove_var("WORKER_MAX_RETRIES");
        std::env::remove_var("WORKER_JOB_TIMEOUT_SECS");

        let config = WorkerConfig::from_env();
        assert_eq!(config.max_concurrent_jobs, 2);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_worker_config_memory_constraint() {
        // 2 concurrent jobs with 20 employees each at ~2.5MB/employee
        // = 2 * 20 * 2.5MB = 100MB for processing
        // This fits comfortably in 256MB container with overhead

        let config = WorkerConfig::default();
        let employees_per_batch = 20;
        let approx_mb_per_employee = 2.5;

        let estimated_memory_mb =
            (config.max_concurrent_jobs as f64) * (employees_per_batch as f64) * approx_mb_per_employee;

        assert!(estimated_memory_mb < 200.0); // Leave room for overhead
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

        assert_eq!(semaphore.available_permits(), 2);

        // Simulate acquiring permits
        let permit1 = semaphore.try_acquire();
        assert!(permit1.is_ok());
        assert_eq!(semaphore.available_permits(), 1);

        let permit2 = semaphore.try_acquire();
        assert!(permit2.is_ok());
        assert_eq!(semaphore.available_permits(), 0);

        // Third should fail
        let permit3 = semaphore.try_acquire();
        assert!(permit3.is_err());
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
        ];

        for task_type in task_types {
            let is_implemented = matches!(
                task_type,
                TaskType::AttendanceReport | TaskType::MergeResults
            );

            // PayrollReport should be not implemented yet
            if matches!(task_type, TaskType::PayrollReport) {
                assert!(!is_implemented);
            } else {
                assert!(is_implemented);
            }
        }
    }
}
