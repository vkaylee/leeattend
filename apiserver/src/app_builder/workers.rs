#![allow(clippy::clone_on_ref_ptr)]
use super::repositories::Repositories;
use super::services::Services;
use super::AppWorkers;
use crate::diesel_pool::DieselPools;
use crate::queue::RedisPool;
use crate::services::ReportCoordinator;
use crate::worker::{HousekeepingScheduler, JobWorker, WorkerConfig};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[allow(clippy::needless_pass_by_value)]
pub fn build_workers(
    diesel_pools: &DieselPools,
    redis_pool: &Option<RedisPool>,
    repos: &Repositories,
    services: &Services,
    log_worker_handle: tokio::task::JoinHandle<()>,
    roster_cache: Option<Arc<crate::cache::RosterCache>>,
) -> (Option<Arc<ReportCoordinator>>, AppWorkers) {
    let worker_cancel_token = CancellationToken::new();
    let (coordinator, worker_handle, scheduler_handle, outbox_sweep_handle) =
        redis_pool.as_ref().map_or_else(
            || (None, None, None, None),
            |redis| {
                let worker_config = WorkerConfig::from_env();
                tracing::info!(
                    max_concurrent = worker_config.max_concurrent_jobs,
                    poll_timeout = worker_config.poll_timeout_secs,
                    "Initializing job worker"
                );

                let queue_repo: Arc<dyn crate::repositories::queue_repository::QueueRepository> =
                    Arc::new(
                        crate::repositories::queue_repository::RedisQueueRepository::new(
                            redis.clone(),
                        ),
                    );

                let coordinator = Arc::new(ReportCoordinator::new(
                    queue_repo.clone(),
                    Arc::clone(&repos.employee),
                    Arc::clone(&repos.workspace),
                    Arc::clone(&services.outbox),
                ));

                let worker = JobWorker::new(
                    redis.clone(),
                    diesel_pools,
                    worker_config,
                    Arc::clone(&coordinator),
                    roster_cache.clone(),
                );

                let scheduler = HousekeepingScheduler::new(redis.clone());

                let outbox_sweep = crate::worker::OutboxSweepScheduler::new(
                    Arc::clone(&services.outbox),
                    Arc::clone(&coordinator),
                );

                (
                    Some(coordinator),
                    Some(worker.start(worker_cancel_token.clone())),
                    Some(scheduler.start(worker_cancel_token.clone())),
                    Some(outbox_sweep.start(worker_cancel_token.clone())),
                )
            },
        );

    let file_scan_scheduler = crate::worker::file_scan_scheduler::FileScanScheduler::new(
        Arc::clone(&repos.file),
        Arc::clone(&services.file),
        Arc::clone(&services.scan),
        Arc::clone(&repos.workspace),
    );
    let file_scan_scheduler_handle = Some(file_scan_scheduler.start(worker_cancel_token.clone()));

    (
        coordinator,
        AppWorkers {
            log_worker_handle,
            job_worker_handle: worker_handle,
            scheduler_handle,
            outbox_sweep_handle,
            file_scan_scheduler_handle,
            job_worker_token: worker_cancel_token,
        },
    )
}
