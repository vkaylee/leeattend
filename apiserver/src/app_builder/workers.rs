#![allow(clippy::clone_on_ref_ptr)]
use super::repositories::Repositories;
use super::services::Services;
use super::AppWorkers;
use crate::diesel_pool::DieselPools;
use crate::queue::RedisPool;
use crate::services::ReportCoordinator;
use crate::worker::pairing_scheduler::{PairingNotification, PairingNotifier};
use crate::worker::{HousekeepingScheduler, JobWorker, PairingScheduler, WorkerConfig};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[allow(clippy::needless_pass_by_value)]
pub fn build_workers(
    diesel_pools: &DieselPools,
    redis_pool: &Option<RedisPool>,
    repos: &Repositories,
    services: &Services,
    roster_cache: Option<Arc<crate::cache::RosterCache>>,
    pairing_rx: mpsc::UnboundedReceiver<PairingNotification>,
    pairing_notifier_for_worker: Option<Arc<PairingNotifier>>,
    registry: &crate::utils::env_registry::EnvRegistry,
) -> (Option<Arc<ReportCoordinator>>, AppWorkers) {
    let worker_cancel_token = CancellationToken::new();
    let (coordinator, worker_handle, scheduler_handle, outbox_sweep_handle) =
        redis_pool.as_ref().map_or_else(
            || (None, None, None, None),
            |redis| {
                let worker_config = WorkerConfig::from_registry(registry);
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
                    pairing_notifier_for_worker.clone(),
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

    // PairingScheduler: drains the notification channel every 2 s and calls
    // reprocess_daily_attendance once per (employment, date) group, eliminating
    // the pg_advisory_xact_lock contention from the per-log hot path.
    let pairing_scheduler = PairingScheduler::new(
        pairing_rx,
        Arc::clone(&services.pairing),
        Arc::clone(&repos.employee),
        registry,
    );
    let pairing_scheduler_handle = Some(pairing_scheduler.start(worker_cancel_token.clone()));

    (
        coordinator,
        AppWorkers {
            job_worker_handle: worker_handle,
            scheduler_handle,
            outbox_sweep_handle,
            file_scan_scheduler_handle,
            pairing_scheduler_handle,
            job_worker_token: worker_cancel_token,
        },
    )
}
