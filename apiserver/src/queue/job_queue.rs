//! Job Queue — Redis-backed FIFO queue for distributed job processing.
//!
//! Uses Redis LIST (`worker:jobs`) for job distribution via BLPOP,
//! HASH for batch result storage, and STRING keys for metadata/counters.
//!
//! Key patterns:
//! - `worker:jobs` — main job queue (LIST)
//! - `report:meta:{job_id}` — job metadata (STRING, JSON)
//! - `report:results:{job_id}` — batch results (HASH, `batch_NNNNN` → JSON)
//! - `report:counter:{job_id}` — completed batch counter (STRING, atomic INCR)
//! - `report:merge_triggered:{job_id}` — merge dedup flag (STRING, SETNX)
//! - `report:dlq` — dead letter queue for failed jobs (LIST)

use redis::AsyncCommands;
use tracing::{debug, error};
use uuid::Uuid;

use crate::queue::{Job, JobMetadata, JobPriority, RedisPool};

/// High-priority queue key (reports, merge, accrual).
pub(crate) const QUEUE_HIGH: &str = "worker:jobs:high";
/// Low-priority queue key (log ingestion, background tasks).
pub(crate) const QUEUE_LOW: &str = "worker:jobs:low";
pub(crate) const META_PREFIX: &str = "report:meta:";
pub(crate) const RESULTS_PREFIX: &str = "report:results:";
pub(crate) const COUNTER_PREFIX: &str = "report:counter:";
pub(crate) const MERGE_FLAG_PREFIX: &str = "report:merge_triggered:";
pub(crate) const DLQ_KEY: &str = "report:dlq";
pub(crate) const JOB_TTL_SECS: i64 = 86400; // 24 hours

// Global daily stats keys
const STATS_COMPLETED_PREFIX: &str = "stats:completed:";
const STATS_FAILED_PREFIX: &str = "stats:failed:";
const STATS_TTL_SECS: i64 = 604800; // 7 days retention

/// Queue for worker job processing.
/// Uses Redis LIST for FIFO job distribution and HASH for result storage.
pub struct JobQueue {
    pool: RedisPool,
    namespace: String,
}

impl JobQueue {
    /// Create a new JobQueue with the given Redis pool.
    pub fn new(pool: RedisPool) -> Self {
        Self { pool, namespace: "".to_string() }
    }

    /// Create a new JobQueue with a specific namespace (mainly for isolated testing).
    pub fn with_namespace(pool: RedisPool, namespace: &str) -> Self {
        Self { pool, namespace: namespace.to_string() }
    }

    /// Get the high priority queue key.
    pub fn queue_high(&self) -> String {
        if self.namespace.is_empty() {
            QUEUE_HIGH.to_string()
        } else {
            format!("{}:{}", self.namespace, QUEUE_HIGH)
        }
    }

    /// Get the low priority queue key.
    pub fn queue_low(&self) -> String {
        if self.namespace.is_empty() {
            QUEUE_LOW.to_string()
        } else {
            format!("{}:{}", self.namespace, QUEUE_LOW)
        }
    }

    /// Enqueue multiple jobs atomically using RPUSH.
    /// Jobs are routed to the appropriate priority queue.
    ///
    /// # Arguments
    /// * `jobs` - Vector of jobs to enqueue
    ///
    /// # Errors
    /// Returns `RedisError` if the operation fails.
    pub async fn enqueue_jobs(&self, jobs: Vec<Job>) -> Result<(), redis::RedisError> {
        if jobs.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.conn();

        for job in jobs {
            let json = serde_json::to_string(&job).expect("Job serialization should not fail");
            let key = match job.priority {
                JobPriority::Low => self.queue_low(),
                _ => self.queue_high(),
            };
            tracing::debug!(job_id = %job.job_id, queue = key, "Pushing job to queue");
            let _: () = conn.rpush(key, json).await?;
        }

        Ok(())
    }

    /// Enqueue a single job. Convenience method for log ingestion.
    pub async fn enqueue_single(&self, job: Job) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let json = serde_json::to_string(&job).expect("Job serialization should not fail");
        let key = match job.priority {
            JobPriority::Low => self.queue_low(),
            _ => self.queue_high(),
        };
        let _: () = conn.rpush(key, json).await?;
        Ok(())
    }

    /// Blocking dequeue with timeout (for workers).
    /// Uses BLPOP for efficient waiting without polling.
    ///
    /// # Arguments
    /// * `timeout_secs` - Maximum seconds to wait for a job (must be finite, positive)
    ///
    /// # Returns
    /// * `Ok(Some(job))` - A job was dequeued
    /// * `Ok(None)` - Timeout reached, no job available
    /// * `Err(e)` - Redis error
    pub async fn dequeue_job(
        &self,
        timeout_secs: f64,
    ) -> Result<Option<Job>, redis::RedisError> {
        // Validate timeout value (Redis protocol requires valid positive number)
        if !timeout_secs.is_finite() || timeout_secs < 0.0 {
            return Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Invalid timeout",
                format!("timeout must be finite and non-negative, got: {timeout_secs}"),
            )));
        }

        // Convert timeout to integer seconds for Redis BLPOP
        // BLPOP expects timeout as integer in Redis protocol
        let timeout_int = timeout_secs.round() as i64;

        // Use dedicated blocking connection to avoid head-of-line blocking.
        // BLPOP blocks for up to timeout_secs; using the shared conn() would
        // stall all concurrent HSET/PUBLISH/GET operations from other workers.
        let mut conn = self.pool.blocking_conn();

        // Use redis::cmd instead of blpop method to ensure correct type serialization
        // BLPOP checks keys left-to-right: high-priority queue is checked first
        let result: Option<(String, String)> = redis::cmd("BLPOP")
            .arg(self.queue_high())
            .arg(self.queue_low())
            .arg(timeout_int)
            .query_async(&mut conn)
            .await?;

        match result {
            Some((_, json)) => {
                let job: Job = serde_json::from_str(&json).map_err(|e| {
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
        let json = serde_json::to_string(meta).expect("JobMetadata serialization should not fail");

        redis::pipe()
            .set(&key, json)
            .expire(&key, JOB_TTL_SECS)
            .query_async::<()>(&mut conn)
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
        let key = format!("{META_PREFIX}{job_id}");
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
        let json = serde_json::to_string(meta).expect("JobMetadata serialization should not fail");

        // Get current TTL before update (compatible with Redis 3.0+)
        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        // If key has TTL (>0), preserve it; otherwise use default
        if ttl > 0 {
            redis::pipe()
                .set(&key, &json)
                .expire(&key, ttl as i64)
                .query_async::<()>(&mut conn)
                .await
        } else {
            // Key doesn't exist or has no TTL, use default
            redis::pipe()
                .set(&key, &json)
                .expire(&key, JOB_TTL_SECS)
                .query_async::<()>(&mut conn)
                .await
        }
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
        let counter_key = format!("{COUNTER_PREFIX}{parent_job_id}");

        // Set TTL on first increment (counter may not exist yet)
        // Use tuple destructuring to handle pipe response (incr returns u32, expire returns i32)
        let (count, _): (u32, i32) = redis::pipe()
            .incr(&counter_key, 1)
            .expire(&counter_key, JOB_TTL_SECS)
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
        let failed_key = format!("{COUNTER_PREFIX}{parent_job_id}:failed");

        // Use tuple destructuring to handle pipe response (incr returns u32, expire returns i32)
        let (count, _): (u32, i32) = redis::pipe()
            .incr(&failed_key, 1)
            .expire(&failed_key, JOB_TTL_SECS)
            .query_async(&mut conn)
            .await?;

        Ok(count)
    }

    /// Get current completed count for a job.
    pub async fn get_completed_count(&self, parent_job_id: Uuid) -> Result<u32, redis::RedisError> {
        let mut conn = self.pool.conn();
        let counter_key = format!("{COUNTER_PREFIX}{parent_job_id}");
        let count: Option<u32> = conn.get(&counter_key).await?;
        Ok(count.unwrap_or(0))
    }

    /// Atomically try to set merge triggered flag.
    /// Returns true if this call successfully set the flag (first caller wins).
    /// Returns false if flag was already set by another worker (race condition prevented).
    ///
    /// # Arguments
    /// * `parent_job_id` - Parent job ID
    ///
    /// # Returns
    /// * `Ok(true)` - This worker should trigger merge
    /// * `Ok(false)` - Another worker already triggered merge
    pub async fn try_trigger_merge(&self, parent_job_id: Uuid) -> Result<bool, redis::RedisError> {
        let mut conn = self.pool.conn();
        let flag_key = format!("{MERGE_FLAG_PREFIX}{parent_job_id}");

        // SETNX: Set key only if it doesn't exist (atomic operation)
        // Returns 1 if key was set, 0 if key already exists
        let was_set: bool = redis::cmd("SET")
            .arg(&flag_key)
            .arg("triggered")
            .arg("NX") // Only set if not exists
            .arg("EX") // Set expiration
            .arg(JOB_TTL_SECS)
            .query_async::<bool>(&mut conn)
            .await?;

        debug!(
            job_id = %parent_job_id,
            flag_set = was_set,
            "Attempted to set merge trigger flag"
        );

        Ok(was_set)
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
        let key = format!("{RESULTS_PREFIX}{parent_job_id}");
        let field = format!("batch_{batch_index:05}");

        redis::pipe()
            .hset(&key, &field, result)
            .expire(&key, JOB_TTL_SECS)
            .query_async::<()>(&mut conn)
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
        let key = format!("{RESULTS_PREFIX}{parent_job_id}");
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
    pub async fn move_to_dlq(&self, job: &Job, error: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();

        #[derive(serde::Serialize)]
        struct DlqEntry<'a> {
            job: &'a Job,
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

    /// Get queue length (pending jobs count across both priority queues).
    pub async fn queue_length(&self) -> Result<usize, redis::RedisError> {
        let mut conn = self.pool.conn();
        let high: usize = conn.llen(self.queue_high()).await?;
        let low: usize = conn.llen(self.queue_low()).await?;
        Ok(high + low)
    }

    /// Clean up all data for a job (metadata, counter, results).
    /// Used after final result is delivered.
    pub async fn cleanup_job(&self, job_id: Uuid) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();

        let meta_key = format!("{META_PREFIX}{job_id}");
        let counter_key = format!("{COUNTER_PREFIX}{job_id}");
        let failed_key = format!("{COUNTER_PREFIX}{job_id}:failed");
        let results_key = format!("{RESULTS_PREFIX}{job_id}");
        let data_key = format!("{RESULTS_PREFIX}{job_id}:data");
        let filename_key = format!("{RESULTS_PREFIX}{job_id}:filename");
        let merge_flag_key = format!("{MERGE_FLAG_PREFIX}{job_id}");

        let _: () = redis::pipe()
            .del(&meta_key)
            .del(&counter_key)
            .del(&failed_key)
            .del(&results_key)
            .del(&data_key)
            .del(&filename_key)
            .del(&merge_flag_key)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    /// Store the final Excel data directly in Redis for client download.
    /// The data is stored directly as binary bytes with a 1-hour TTL.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID
    /// * `data` - Raw Excel bytes
    /// * `filename` - Original filename for Content-Disposition header
    pub async fn set_result_data(
        &self,
        job_id: Uuid,
        data: &[u8],
        filename: &str,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let data_key = format!("{RESULTS_PREFIX}{job_id}:data");
        let filename_key = format!("{RESULTS_PREFIX}{job_id}:filename");

        debug!(
            job_id = %job_id,
            data_size = data.len(),
            filename = %filename,
            "Storing result data in Redis"
        );

        // Store both data and filename with 1-hour TTL
        redis::pipe()
            .set(&data_key, data)
            .expire(&data_key, 3600) // 1 hour TTL
            .set(&filename_key, filename)
            .expire(&filename_key, 3600)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| {
                error!(
                    job_id = %job_id,
                    error = %e,
                    data_size = data.len(),
                    "Failed to store result data in Redis"
                );
                e
            })?;

        debug!(
            job_id = %job_id,
            "Successfully stored result data in Redis"
        );

        Ok(())
    }

    /// Get the Excel data for a completed job.
    ///
    /// # Returns
    /// * `Ok(Some((data, filename)))` - Data and filename found
    /// * `Ok(None)` - Data not found (job may not be complete or TTL expired)
    pub async fn get_result_data(
        &self,
        job_id: Uuid,
    ) -> Result<Option<(Vec<u8>, String)>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let data_key = format!("{RESULTS_PREFIX}{job_id}:data");
        let filename_key = format!("{RESULTS_PREFIX}{job_id}:filename");

        let data: Option<Vec<u8>> = conn.get(&data_key).await?;
        let filename: Option<String> = conn.get(&filename_key).await?;

        match (data, filename) {
            (Some(bytes), Some(fname)) => Ok(Some((bytes, fname))),
            _ => Ok(None),
        }
    }

    /// Delete batch results after merge completes.
    /// This frees memory in Redis by removing the results hash.
    ///
    /// # Arguments
    /// * `parent_job_id` - Parent job ID whose results should be deleted
    pub async fn delete_results(&self, parent_job_id: Uuid) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{RESULTS_PREFIX}{parent_job_id}");
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    /// Increment global daily completed jobs counter.
    /// Uses date-based key for daily aggregation with 7-day TTL.
    pub async fn increment_daily_completed(&self) -> Result<u64, redis::RedisError> {
        let mut conn = self.pool.conn();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let key = format!("{STATS_COMPLETED_PREFIX}{today}");

        // Use tuple destructuring to handle pipe response (incr returns u64, expire returns i32)
        let (count, _): (u64, i32) = redis::pipe()
            .incr(&key, 1)
            .expire(&key, STATS_TTL_SECS)
            .query_async(&mut conn)
            .await?;

        Ok(count)
    }

    /// Increment global daily failed jobs counter.
    /// Uses date-based key for daily aggregation with 7-day TTL.
    pub async fn increment_daily_failed(&self) -> Result<u64, redis::RedisError> {
        let mut conn = self.pool.conn();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let key = format!("{STATS_FAILED_PREFIX}{today}");

        // Use tuple destructuring to handle pipe response (incr returns u64, expire returns i32)
        let (count, _): (u64, i32) = redis::pipe()
            .incr(&key, 1)
            .expire(&key, STATS_TTL_SECS)
            .query_async(&mut conn)
            .await?;

        Ok(count)
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

    /// List recent jobs by scanning all metadata keys in Redis.
    /// Returns up to `limit` most recent jobs, sorted by `created_at` descending.
    pub async fn list_recent_jobs(&self, limit: usize) -> Result<Vec<JobMetadata>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let pattern = format!("{META_PREFIX}*");

        // Scan for all metadata keys
        let mut keys: Vec<String> = Vec::new();
        let mut cursor: u64 = 0;
        loop {
            let (next_cursor, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;

            keys.extend(batch);
            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // MGET all values
        let values: Vec<Option<String>> = conn.mget(&keys).await?;

        let mut jobs: Vec<JobMetadata> = values
            .into_iter()
            .filter_map(|v| v.and_then(|s| serde_json::from_str::<JobMetadata>(&s).ok()))
            .collect();

        // Sort by created_at descending (newest first)
        jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        jobs.truncate(limit);

        Ok(jobs)
    }
}

#[cfg(test)]
#[path = "job_queue_tests.rs"]
mod job_queue_tests;

