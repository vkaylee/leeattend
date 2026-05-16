use async_trait::async_trait;
use uuid::Uuid;
use crate::queue::{Job, JobMetadata, JobStatus};

/// Abstract interface for Queue and distributed progress tracking operations.
///
/// This allows Dependency Injection to decouple services (like ReportCoordinator
/// and ImportPipeline) from concrete Redis connections.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QueueRepository: Send + Sync {
    /// Enqueue multiple jobs atomically
    async fn enqueue_jobs(&self, jobs: Vec<Job>) -> Result<(), String>;

    /// Create metadata for a parent job
    async fn create_job_metadata(&self, meta: &JobMetadata) -> Result<(), String>;

    /// Get metadata for a parent job
    async fn get_job_metadata(&self, job_id: Uuid) -> Result<Option<JobMetadata>, String>;

    /// Update metadata for a parent job
    async fn update_job_metadata(&self, meta: &JobMetadata) -> Result<(), String>;

    /// Update status directly
    async fn update_job_status(&self, job_id: Uuid, status: JobStatus) -> Result<(), String>;

    /// Atomically increment the completed batches counter
    async fn increment_completed(&self, job_id: Uuid) -> Result<u32, String>;

    /// Atomically increment the failed batches counter
    async fn increment_failed(&self, job_id: Uuid) -> Result<u32, String>;

    /// Try to trigger a merge job, returns true if caller should trigger it
    async fn try_trigger_merge(&self, job_id: Uuid) -> Result<bool, String>;

    /// Increment a general progress counter (e.g. for import jobs)
    async fn increment_progress(&self, job_id: Uuid, amount: u32) -> Result<(), String>;

    /// Store an error string for a specific chunk/task (e.g. for import jobs)
    async fn store_import_error(&self, job_id: Uuid, chunk_id: &str, error: &str) -> Result<(), String>;
}

use crate::queue::{JobQueue, RedisPool};

pub struct RedisQueueRepository {
    queue: JobQueue,
    redis: RedisPool,
}

impl RedisQueueRepository {
    pub fn new(redis: RedisPool) -> Self {
        Self {
            queue: JobQueue::new(redis.clone()),
            redis,
        }
    }
}

#[async_trait]
impl QueueRepository for RedisQueueRepository {
    async fn enqueue_jobs(&self, jobs: Vec<Job>) -> Result<(), String> {
        self.queue.enqueue_jobs(jobs).await.map_err(|e| e.to_string())
    }

    async fn create_job_metadata(&self, meta: &JobMetadata) -> Result<(), String> {
        self.queue.create_job_metadata(meta).await.map_err(|e| e.to_string())
    }

    async fn get_job_metadata(&self, job_id: Uuid) -> Result<Option<JobMetadata>, String> {
        self.queue.get_job_metadata(job_id).await.map_err(|e| e.to_string())
    }

    async fn update_job_metadata(&self, meta: &JobMetadata) -> Result<(), String> {
        self.queue.update_job_metadata(meta).await.map_err(|e| e.to_string())
    }

    async fn update_job_status(&self, job_id: Uuid, status: JobStatus) -> Result<(), String> {
        self.queue.update_job_status(job_id, status).await.map_err(|e| e.to_string())
    }

    async fn increment_completed(&self, job_id: Uuid) -> Result<u32, String> {
        self.queue.increment_completed(job_id).await.map_err(|e| e.to_string())
    }

    async fn increment_failed(&self, job_id: Uuid) -> Result<u32, String> {
        self.queue.increment_failed(job_id).await.map_err(|e| e.to_string())
    }

    async fn try_trigger_merge(&self, job_id: Uuid) -> Result<bool, String> {
        self.queue.try_trigger_merge(job_id).await.map_err(|e| e.to_string())
    }

    async fn increment_progress(&self, job_id: Uuid, amount: u32) -> Result<(), String> {
        let mut conn = self.redis.conn();
        let key = format!("import_progress:{}", job_id);
        let _: () = redis::cmd("INCRBY")
            .arg(&key)
            .arg(amount)
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn store_import_error(&self, job_id: Uuid, chunk_id: &str, error: &str) -> Result<(), String> {
        let mut conn = self.redis.conn();
        let key = format!("import_errors:{}", job_id);
        let _: () = redis::cmd("HSET")
            .arg(&key)
            .arg(chunk_id)
            .arg(error)
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

