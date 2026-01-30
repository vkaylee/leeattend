use redis::AsyncCommands;
use uuid::Uuid;

use crate::queue::{JobMetadata, RedisPool, ReportJob};

const QUEUE_KEY: &str = "report:jobs:attendance";
const META_PREFIX: &str = "report:meta:";
const RESULTS_PREFIX: &str = "report:results:";
const COUNTER_PREFIX: &str = "report:counter:";
const DLQ_KEY: &str = "report:dlq";
const JOB_TTL_SECS: i64 = 86400; // 24 hours

/// Queue for report processing jobs.
/// Uses Redis LIST for FIFO job distribution and HASH for result storage.
pub struct ReportQueue {
    pool: RedisPool,
}

impl ReportQueue {
    /// Create a new ReportQueue with the given Redis pool.
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// Enqueue multiple jobs atomically using RPUSH.
    /// Jobs are appended to the queue in order for FIFO processing.
    ///
    /// # Arguments
    /// * `jobs` - Vector of jobs to enqueue
    ///
    /// # Errors
    /// Returns `RedisError` if the operation fails.
    pub async fn enqueue_jobs(&self, jobs: Vec<ReportJob>) -> Result<(), redis::RedisError> {
        if jobs.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.conn();
        let serialized: Vec<String> = jobs
            .iter()
            .map(|j| serde_json::to_string(j).expect("Job serialization should not fail"))
            .collect();

        // RPUSH for FIFO ordering (workers use BLPOP from left)
        let _: () = conn.rpush(QUEUE_KEY, &serialized).await?;
        Ok(())
    }

    /// Blocking dequeue with timeout (for workers).
    /// Uses BLPOP for efficient waiting without polling.
    ///
    /// # Arguments
    /// * `timeout_secs` - Maximum seconds to wait for a job
    ///
    /// # Returns
    /// * `Ok(Some(job))` - A job was dequeued
    /// * `Ok(None)` - Timeout reached, no job available
    /// * `Err(e)` - Redis error
    pub async fn dequeue_job(
        &self,
        timeout_secs: f64,
    ) -> Result<Option<ReportJob>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let result: Option<(String, String)> = conn.blpop(QUEUE_KEY, timeout_secs).await?;

        match result {
            Some((_, json)) => {
                let job: ReportJob = serde_json::from_str(&json).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "JSON parse error",
                        e.to_string(),
                    ))
                })?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// Create job metadata entry for tracking overall progress.
    /// Sets TTL for automatic cleanup.
    ///
    /// # Arguments
    /// * `meta` - Job metadata to store
    pub async fn create_job_metadata(&self, meta: &JobMetadata) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", META_PREFIX, meta.job_id);
        let json =
            serde_json::to_string(meta).expect("JobMetadata serialization should not fail");

        redis::pipe()
            .set(&key, json)
            .expire(&key, JOB_TTL_SECS)
            .query_async(&mut conn)
            .await
    }

    /// Get job metadata for status polling.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID to query
    ///
    /// # Returns
    /// * `Ok(Some(meta))` - Metadata found
    /// * `Ok(None)` - Job not found (possibly expired)
    pub async fn get_job_metadata(
        &self,
        job_id: Uuid,
    ) -> Result<Option<JobMetadata>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", META_PREFIX, job_id);
        let json: Option<String> = conn.get(&key).await?;

        match json {
            Some(s) => {
                let meta: JobMetadata = serde_json::from_str(&s).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "JSON parse error",
                        e.to_string(),
                    ))
                })?;
                Ok(Some(meta))
            }
            None => Ok(None),
        }
    }

    /// Update job metadata (for status changes).
    /// Preserves existing TTL.
    pub async fn update_job_metadata(&self, meta: &JobMetadata) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", META_PREFIX, meta.job_id);
        let json =
            serde_json::to_string(meta).expect("JobMetadata serialization should not fail");

        // SET with KEEPTTL to preserve existing expiration
        redis::cmd("SET")
            .arg(&key)
            .arg(json)
            .arg("KEEPTTL")
            .query_async(&mut conn)
            .await
    }

    /// Atomically increment completed batch count.
    /// Uses a separate counter key for atomic operations.
    ///
    /// # Arguments
    /// * `parent_job_id` - Parent job to update
    ///
    /// # Returns
    /// The new completed count after increment.
    pub async fn increment_completed(&self, parent_job_id: Uuid) -> Result<u32, redis::RedisError> {
        let mut conn = self.pool.conn();
        let counter_key = format!("{}{}", COUNTER_PREFIX, parent_job_id);

        // Set TTL on first increment (counter may not exist yet)
        let count: u32 = redis::pipe()
            .incr(&counter_key, 1)
            .expire(&counter_key, JOB_TTL_SECS)
            .ignore()
            .query_async(&mut conn)
            .await?;

        Ok(count)
    }

    /// Atomically increment failed batch count.
    /// Uses a separate counter key for failed jobs.
    ///
    /// # Arguments
    /// * `parent_job_id` - Parent job to update
    ///
    /// # Returns
    /// The new failed count after increment.
    pub async fn increment_failed(&self, parent_job_id: Uuid) -> Result<u32, redis::RedisError> {
        let mut conn = self.pool.conn();
        let failed_key = format!("{}{}:failed", COUNTER_PREFIX, parent_job_id);

        let count: u32 = redis::pipe()
            .incr(&failed_key, 1)
            .expire(&failed_key, JOB_TTL_SECS)
            .ignore()
            .query_async(&mut conn)
            .await?;

        Ok(count)
    }

    /// Get current completed count for a job.
    pub async fn get_completed_count(&self, parent_job_id: Uuid) -> Result<u32, redis::RedisError> {
        let mut conn = self.pool.conn();
        let counter_key = format!("{}{}", COUNTER_PREFIX, parent_job_id);
        let count: Option<u32> = conn.get(&counter_key).await?;
        Ok(count.unwrap_or(0))
    }

    /// Store a batch result in the results hash.
    /// Results are keyed by batch index for ordered retrieval.
    ///
    /// # Arguments
    /// * `parent_job_id` - Parent job ID
    /// * `batch_index` - Batch number (0-based)
    /// * `result` - JSON-serialized result data
    pub async fn store_result(
        &self,
        parent_job_id: Uuid,
        batch_index: u32,
        result: &str,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", RESULTS_PREFIX, parent_job_id);
        let field = format!("batch_{:05}", batch_index);

        redis::pipe()
            .hset(&key, &field, result)
            .expire(&key, JOB_TTL_SECS)
            .query_async(&mut conn)
            .await
    }

    /// Get all batch results for aggregation.
    /// Returns results as (field_name, json_data) pairs sorted by batch index.
    ///
    /// # Arguments
    /// * `parent_job_id` - Parent job ID
    ///
    /// # Returns
    /// Vector of (batch_key, result_json) pairs.
    pub async fn get_all_results(
        &self,
        parent_job_id: Uuid,
    ) -> Result<Vec<(String, String)>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", RESULTS_PREFIX, parent_job_id);
        let results: Vec<(String, String)> = conn.hgetall(&key).await?;

        // Sort by batch key for consistent ordering
        let mut sorted = results;
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(sorted)
    }

    /// Move a failed job to the dead letter queue for inspection.
    ///
    /// # Arguments
    /// * `job` - The failed job
    /// * `error` - Error message describing the failure
    pub async fn move_to_dlq(&self, job: &ReportJob, error: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();

        #[derive(serde::Serialize)]
        struct DlqEntry<'a> {
            job: &'a ReportJob,
            error: &'a str,
            failed_at: chrono::DateTime<chrono::Utc>,
        }

        let entry = DlqEntry {
            job,
            error,
            failed_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&entry).expect("DLQ serialization should not fail");
        let _: () = conn.rpush(DLQ_KEY, json).await?;
        Ok(())
    }

    /// Get queue length (pending jobs count).
    pub async fn queue_length(&self) -> Result<usize, redis::RedisError> {
        let mut conn = self.pool.conn();
        let len: usize = conn.llen(QUEUE_KEY).await?;
        Ok(len)
    }

    /// Clean up all data for a job (metadata, counter, results).
    /// Used after final result is delivered.
    pub async fn cleanup_job(&self, job_id: Uuid) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();

        let meta_key = format!("{}{}", META_PREFIX, job_id);
        let counter_key = format!("{}{}", COUNTER_PREFIX, job_id);
        let failed_key = format!("{}{}:failed", COUNTER_PREFIX, job_id);
        let results_key = format!("{}{}", RESULTS_PREFIX, job_id);
        let url_key = format!("{}{}:url", RESULTS_PREFIX, job_id);

        let _: () = redis::pipe()
            .del(&meta_key)
            .del(&counter_key)
            .del(&failed_key)
            .del(&results_key)
            .del(&url_key)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    /// Store the final result URL for a completed job.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID
    /// * `url` - Download URL for the generated report
    pub async fn set_result_url(&self, job_id: Uuid, url: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}:url", RESULTS_PREFIX, job_id);

        redis::pipe()
            .set(&key, url)
            .expire(&key, JOB_TTL_SECS)
            .query_async(&mut conn)
            .await
    }

    /// Get the result URL for a completed job.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID
    ///
    /// # Returns
    /// * `Ok(Some(url))` - URL found
    /// * `Ok(None)` - URL not set (job may not be complete)
    pub async fn get_result_url(&self, job_id: Uuid) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}:url", RESULTS_PREFIX, job_id);
        conn.get(&key).await
    }

    /// Delete batch results after merge completes.
    /// This frees memory in Redis by removing the results hash.
    ///
    /// # Arguments
    /// * `parent_job_id` - Parent job ID whose results should be deleted
    pub async fn delete_results(&self, parent_job_id: Uuid) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", RESULTS_PREFIX, parent_job_id);
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    /// Update job status atomically.
    /// Also sets completed_at timestamp for terminal states.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID
    /// * `status` - New status to set
    pub async fn update_job_status(
        &self,
        job_id: Uuid,
        status: crate::queue::JobStatus,
    ) -> Result<(), redis::RedisError> {
        let mut meta = self.get_job_metadata(job_id).await?.ok_or_else(|| {
            redis::RedisError::from((redis::ErrorKind::TypeError, "Job not found"))
        })?;

        meta.status = status.clone();
        if matches!(
            status,
            crate::queue::JobStatus::Completed | crate::queue::JobStatus::Failed
        ) {
            meta.completed_at = Some(chrono::Utc::now());
        }

        self.update_job_metadata(&meta).await
    }
}
