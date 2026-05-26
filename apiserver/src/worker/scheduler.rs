use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use uuid::Uuid;

use crate::queue::{
    HousekeepingPayload, Job, JobPayload, JobPriority, JobQueue, RedisPool, TaskType,
};

pub struct HousekeepingScheduler {
    queue: JobQueue,
    redis: RedisPool,
}

impl HousekeepingScheduler {
    pub fn new(redis: RedisPool) -> Self {
        Self {
            queue: JobQueue::new(redis.clone()),
            redis,
        }
    }

    pub fn start(self, cancel_token: CancellationToken) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Housekeeping scheduler started");
            // Check every hour
            let mut tick = interval(Duration::from_secs(3600));

            loop {
                tokio::select! {
                    () = cancel_token.cancelled() => break,
                    _ = tick.tick() => {
                        let today = chrono::Utc::now().date_naive();
                        let lock_key = format!("lock:housekeeping:{today}");

                        let mut conn = self.redis.conn();

                        // Try to acquire daily lock using SET NX EX
                        let acquired: Result<Option<String>, _> = redis::cmd("SET")
                            .arg(&lock_key)
                            .arg("1")
                            .arg("NX")
                            .arg("EX")
                            .arg(86400) // 24 hours
                            .query_async(&mut conn)
                            .await;

                        if let Ok(Some(_)) = acquired {
                            info!("Acquired daily housekeeping lock, enqueuing job");
                            let job = Job::with_priority(
                                Uuid::nil(),
                                0,
                                TaskType::NightlyHousekeeping,
                                Uuid::nil(), // No specific workspace
                                JobPayload::NightlyHousekeeping(HousekeepingPayload {
                                    reference_date: today,
                                }),
                                JobPriority::Low,
                            );

                            if let Err(e) = self.queue.enqueue_single(job).await {
                                error!(error = %e, "Failed to enqueue nightly housekeeping job");
                            }

                            // Enqueue expiring contracts check
                            let contracts_job = Job::with_priority(
                                Uuid::nil(),
                                0,
                                TaskType::ExpiringContractsCheck,
                                Uuid::nil(),
                                JobPayload::ExpiringContractsCheck(crate::queue::ExpiringContractsPayload {
                                    reference_date: today,
                                }),
                                JobPriority::Low,
                            );

                            if let Err(e) = self.queue.enqueue_single(contracts_job).await {
                                error!(error = %e, "Failed to enqueue expiring contracts check job");
                            }
                        }
                    }
                }
            }
            info!("Housekeeping scheduler stopped");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    async fn get_test_redis() -> Option<RedisPool> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        match tokio::time::timeout(Duration::from_secs(5), RedisPool::new(&redis_url)).await {
            Ok(Ok(pool)) => {
                match tokio::time::timeout(Duration::from_secs(5), pool.health_check()).await {
                    Ok(Ok(())) => Some(pool),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    #[tokio::test]
    async fn test_scheduler_enqueues_housekeeping_job() {
        let Some(pool) = get_test_redis().await else {
            println!("SKIPPED: Redis not available");
            return;
        };

        // Ensure clean state for today's lock
        let today = chrono::Utc::now().date_naive();
        // Since the scheduler uses the global lock, we append a test suffix to avoid conflicts if possible.
        // But the scheduler hardcodes `lock:housekeeping:{today}`. We just delete it.
        let lock_key = format!("lock:housekeeping:{today}");
        let mut conn = pool.conn();
        let _: () = redis::cmd("DEL")
            .arg(&lock_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        let cancel_token = CancellationToken::new();
        // Create scheduler
        let scheduler = HousekeepingScheduler::new(pool.clone());
        let queue = JobQueue::new(pool.clone());

        // Drain any existing jobs in the default low queue to prevent false positives
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        let handle = scheduler.start(cancel_token.clone());

        // Wait a little bit for the first tick to execute
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check if job was enqueued
        let mut block_conn = pool.blocking_conn();
        let result: Option<(String, String)> = redis::cmd("BLPOP")
            .arg(queue.queue_low())
            .arg(2) // 2 second timeout
            .query_async(&mut block_conn)
            .await
            .unwrap();

        assert!(result.is_some(), "Job was not enqueued");
        let (_, json) = result.unwrap();

        let job: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(job.task_type, TaskType::NightlyHousekeeping);

        if let JobPayload::NightlyHousekeeping(payload) = job.payload {
            assert_eq!(payload.reference_date, today);
        } else {
            panic!("Expected NightlyHousekeeping payload");
        }

        // Cancel scheduler
        cancel_token.cancel();
        handle.await.unwrap();

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(&lock_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(());
    }
}
