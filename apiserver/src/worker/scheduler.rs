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
                    _ = cancel_token.cancelled() => break,
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
                        }
                    }
                }
            }
            info!("Housekeeping scheduler stopped");
        })
    }
}
