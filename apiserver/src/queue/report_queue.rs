use redis::AsyncCommands;
use tracing::{debug, error};
use uuid::Uuid;

use crate::queue::{JobMetadata, RedisPool, ReportJob};

const QUEUE_KEY: &str = "report:jobs:attendance";
const META_PREFIX: &str = "report:meta:";
const RESULTS_PREFIX: &str = "report:results:";
const COUNTER_PREFIX: &str = "report:counter:";
const MERGE_FLAG_PREFIX: &str = "report:merge_triggered:";
const DLQ_KEY: &str = "report:dlq";
const JOB_TTL_SECS: i64 = 86400; // 24 hours

// Global daily stats keys
const STATS_COMPLETED_PREFIX: &str = "stats:completed:";
const STATS_FAILED_PREFIX: &str = "stats:failed:";
const STATS_TTL_SECS: i64 = 604800; // 7 days retention

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
    /// * `timeout_secs` - Maximum seconds to wait for a job (must be finite, positive)
    ///
    /// # Returns
    /// * `Ok(Some(job))` - A job was dequeued
    /// * `Ok(None)` - Timeout reached, no job available
    /// * `Err(e)` - Redis error
    pub async fn dequeue_job(
        &self,
        timeout_secs: f64,
    ) -> Result<Option<ReportJob>, redis::RedisError> {
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

        let mut conn = self.pool.conn();

        // Use redis::cmd instead of blpop method to ensure correct type serialization
        let result: Option<(String, String)> = redis::cmd("BLPOP")
            .arg(QUEUE_KEY)
            .arg(timeout_int)
            .query_async(&mut conn)
            .await?;

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
        let json =
            serde_json::to_string(meta).expect("JobMetadata serialization should not fail");

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
    /// The data is base64 encoded and stored with a 1-hour TTL.
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
        use base64::{engine::general_purpose::STANDARD, Engine as _};

        let mut conn = self.pool.conn();
        let data_key = format!("{RESULTS_PREFIX}{job_id}:data");
        let filename_key = format!("{RESULTS_PREFIX}{job_id}:filename");

        // Encode data as base64 for Redis storage
        let encoded = STANDARD.encode(data);

        debug!(
            job_id = %job_id,
            data_size = data.len(),
            filename = %filename,
            "Storing result data in Redis"
        );

        // Store both data and filename with 1-hour TTL
        // Use pipeline for atomic operation, but with reasonable size limits
        let encoded_size = encoded.len();
        debug!(
            job_id = %job_id,
            encoded_size = encoded_size,
            "Writing data to Redis pipeline"
        );

        redis::pipe()
            .set(&data_key, &encoded)
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
                    encoded_size = encoded_size,
                    "Failed to store result data in Redis (timeout or error)"
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
        use base64::{engine::general_purpose::STANDARD, Engine as _};

        let mut conn = self.pool.conn();
        let data_key = format!("{RESULTS_PREFIX}{job_id}:data");
        let filename_key = format!("{RESULTS_PREFIX}{job_id}:filename");

        let encoded: Option<String> = conn.get(&data_key).await?;
        let filename: Option<String> = conn.get(&filename_key).await?;

        match (encoded, filename) {
            (Some(enc), Some(fname)) => {
                let data = STANDARD.decode(&enc).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "Base64 decode error",
                        e.to_string(),
                    ))
                })?;
                Ok(Some((data, fname)))
            }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{DateRange, ReportConfig, ReportType, TaskType, JobStatus, RedisPool};
    use chrono::NaiveDate;

    // Helper to create test Redis pool with 5s timeout (avoids 60s+ hang when Redis unavailable)
    async fn get_test_redis() -> Option<RedisPool> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            RedisPool::new(&redis_url),
        )
        .await
        {
            Ok(Ok(pool)) => match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                pool.health_check(),
            )
            .await
            {
                Ok(Ok(())) => Some(pool),
                _ => None,
            },
            _ => None,
        }
    }

    macro_rules! setup_redis_or_skip {
        () => {{
            match get_test_redis().await {
                Some(pool) => pool,
                None => {
                    println!("SKIPPED: Redis not available (5s timeout)");
                    return;
                }
            }
        }};
    }

    #[test]
    fn test_queue_key_constants() {
        assert_eq!(QUEUE_KEY, "report:jobs:attendance");
        assert_eq!(META_PREFIX, "report:meta:");
        assert_eq!(RESULTS_PREFIX, "report:results:");
        assert_eq!(COUNTER_PREFIX, "report:counter:");
        assert_eq!(DLQ_KEY, "report:dlq");
        assert_eq!(JOB_TTL_SECS, 86400); // 24 hours
    }

    #[test]
    fn test_meta_key_format() {
        let job_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = format!("{META_PREFIX}{job_id}");
        assert_eq!(key, "report:meta:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_results_key_format() {
        let job_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = format!("{RESULTS_PREFIX}{job_id}");
        assert_eq!(key, "report:results:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_counter_key_format() {
        let job_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let counter_key = format!("{COUNTER_PREFIX}{job_id}");
        let failed_key = format!("{COUNTER_PREFIX}{job_id}:failed");

        assert_eq!(counter_key, "report:counter:550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(failed_key, "report:counter:550e8400-e29b-41d4-a716-446655440000:failed");
    }

    #[test]
    fn test_batch_field_format() {
        let batch_index = 42u32;
        let field = format!("batch_{batch_index:05}");
        assert_eq!(field, "batch_00042");

        // Test sorting order
        let fields: Vec<String> = vec![9, 0, 100, 1, 10]
            .into_iter()
            .map(|i| format!("batch_{i:05}"))
            .collect();
        let mut sorted = fields.clone();
        sorted.sort();

        assert_eq!(sorted[0], "batch_00000");
        assert_eq!(sorted[1], "batch_00001");
        assert_eq!(sorted[2], "batch_00009");
        assert_eq!(sorted[3], "batch_00010");
        assert_eq!(sorted[4], "batch_00100");
    }

    #[test]
    fn test_job_serialization_for_queue() {
        let job = crate::queue::ReportJob::new(
            Uuid::new_v4(),
            5,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            vec![Uuid::new_v4(), Uuid::new_v4()],
            DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: "floor".to_string(),
                work_unit_minutes: 30,
            },
        );

        // Jobs should serialize to JSON for Redis (snake_case)
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("attendance_report"));
        assert!(json.contains("summary"));

        // Verify roundtrip
        let decoded: crate::queue::ReportJob = serde_json::from_str(&json).unwrap();
        assert_eq!(job.job_id, decoded.job_id);
        assert_eq!(job.batch_index, decoded.batch_index);
    }

    #[test]
    fn test_data_key_format() {
        let job_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let data_key = format!("{RESULTS_PREFIX}{job_id}:data");
        let filename_key = format!("{RESULTS_PREFIX}{job_id}:filename");
        assert_eq!(data_key, "report:results:550e8400-e29b-41d4-a716-446655440000:data");
        assert_eq!(filename_key, "report:results:550e8400-e29b-41d4-a716-446655440000:filename");
    }

    #[test]
    fn test_cleanup_key_list() {
        let job_id = Uuid::new_v4();

        let meta_key = format!("{META_PREFIX}{job_id}");
        let counter_key = format!("{COUNTER_PREFIX}{job_id}");
        let failed_key = format!("{COUNTER_PREFIX}{job_id}:failed");
        let results_key = format!("{RESULTS_PREFIX}{job_id}");
        let data_key = format!("{RESULTS_PREFIX}{job_id}:data");
        let filename_key = format!("{RESULTS_PREFIX}{job_id}:filename");
        let merge_flag_key = format!("{MERGE_FLAG_PREFIX}{job_id}");

        // All keys should contain the job_id
        assert!(meta_key.contains(&job_id.to_string()));
        assert!(counter_key.contains(&job_id.to_string()));
        assert!(failed_key.contains(&job_id.to_string()));
        assert!(results_key.contains(&job_id.to_string()));
        assert!(data_key.contains(&job_id.to_string()));
        assert!(filename_key.contains(&job_id.to_string()));
        assert!(merge_flag_key.contains(&job_id.to_string()));
    }

    #[test]
    fn test_merge_flag_key_format() {
        let job_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let merge_flag_key = format!("{MERGE_FLAG_PREFIX}{job_id}");
        assert_eq!(
            merge_flag_key,
            "report:merge_triggered:550e8400-e29b-41d4-a716-446655440000"
        );
    }

    // ====== REDIS TTL PRESERVATION TESTS (Redis 3.0+ compatibility) ======
    // These tests verify the fix for KEEPTTL compatibility issue
    // KEEPTTL requires Redis 6.0+, but our fix uses TTL + expire for compatibility

    #[tokio::test]
    async fn test_update_job_metadata_preserves_existing_ttl() {
        // Test: When updating metadata with existing TTL, preserve that TTL
        let pool = setup_redis_or_skip!();
        let queue = ReportQueue::new(pool.clone());
        let job_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        // Create metadata with 24h TTL
        let mut meta = JobMetadata {
            job_id,
            status: JobStatus::Pending,
            total_batches: 4,
            completed_batches: 0,
            failed_batches: 0,
            workspace_id,
            requester_id: Uuid::new_v4(),
            report_type: ReportType::Detail,
            date_range: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        queue
            .create_job_metadata(&meta)
            .await
            .expect("Failed to create metadata");

        // Get TTL before update
        let mut conn = pool.conn();
        let key = format!("{META_PREFIX}{job_id}");
        let ttl_before: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        assert!(ttl_before > 0, "Initial TTL should be > 0");

        // Update metadata
        meta.status = JobStatus::Processing;
        meta.completed_batches = 1;
        queue
            .update_job_metadata(&meta)
            .await
            .expect("Failed to update metadata");

        // Get TTL after update
        let ttl_after: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        // TTL should be preserved (within reasonable tolerance)
        assert!(
            ttl_after > 0,
            "TTL should be preserved after update, got: {ttl_after}"
        );
        assert!(
            ttl_after <= ttl_before,
            "TTL should not increase after update"
        );

        // Verify metadata was updated
        let retrieved = queue
            .get_job_metadata(job_id)
            .await
            .expect("Failed to retrieve metadata")
            .expect("Metadata should exist");
        assert_eq!(retrieved.completed_batches, 1);
        assert_eq!(retrieved.status, JobStatus::Processing);

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_update_job_metadata_applies_default_ttl_when_missing() {
        // Test: When updating metadata without TTL (new key), apply default 24h TTL
        let pool = setup_redis_or_skip!();
        let queue = ReportQueue::new(pool.clone());
        let job_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        let meta = JobMetadata {
            job_id,
            status: JobStatus::Pending,
            total_batches: 4,
            completed_batches: 0,
            failed_batches: 0,
            workspace_id,
            requester_id: Uuid::new_v4(),
            report_type: ReportType::Detail,
            date_range: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        // Update metadata directly (without creating first)
        queue
            .update_job_metadata(&meta)
            .await
            .expect("Failed to update metadata");

        // Check that TTL was set to default
        let mut conn = pool.conn();
        let key = format!("{META_PREFIX}{job_id}");
        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        assert!(
            ttl > 0 && ttl <= JOB_TTL_SECS,
            "TTL should be set to default ({JOB_TTL_SECS}s), got: {ttl}s"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_multiple_updates_preserve_ttl() {
        // Test: Multiple updates should preserve and not reset TTL
        let pool = setup_redis_or_skip!();
        let queue = ReportQueue::new(pool.clone());
        let job_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        let mut meta = JobMetadata {
            job_id,
            status: JobStatus::Pending,
            total_batches: 4,
            completed_batches: 0,
            failed_batches: 0,
            workspace_id,
            requester_id: Uuid::new_v4(),
            report_type: ReportType::Detail,
            date_range: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        queue
            .create_job_metadata(&meta)
            .await
            .expect("Failed to create metadata");

        let mut conn = pool.conn();
        let key = format!("{META_PREFIX}{job_id}");

        // Sleep briefly to ensure TTL decreases naturally
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // First update
        meta.completed_batches = 1;
        queue
            .update_job_metadata(&meta)
            .await
            .expect("First update failed");

        let ttl_after_first: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Second update
        meta.completed_batches = 2;
        queue
            .update_job_metadata(&meta)
            .await
            .expect("Second update failed");

        let ttl_after_second: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        // TTLs should be similar (within 1 second tolerance)
        assert!(
            ttl_after_first > 0 && ttl_after_second > 0,
            "TTL should remain positive after updates"
        );
        assert!(
            (ttl_after_first - ttl_after_second).abs() <= 2,
            "TTL should not increase significantly: {ttl_after_first} vs {ttl_after_second}"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_update_job_status_preserves_ttl() {
        // Test: update_job_status (which calls update_job_metadata) preserves TTL
        let pool = setup_redis_or_skip!();
        let queue = ReportQueue::new(pool.clone());
        let job_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        let meta = JobMetadata {
            job_id,
            status: JobStatus::Pending,
            total_batches: 4,
            completed_batches: 0,
            failed_batches: 0,
            workspace_id,
            requester_id: Uuid::new_v4(),
            report_type: ReportType::Detail,
            date_range: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        queue
            .create_job_metadata(&meta)
            .await
            .expect("Failed to create metadata");

        let mut conn = pool.conn();
        let key = format!("{META_PREFIX}{job_id}");

        // Get initial TTL
        let ttl_before: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        // Update status (this internally calls update_job_metadata)
        queue
            .update_job_status(job_id, JobStatus::Processing)
            .await
            .expect("Failed to update job status");

        // Get TTL after status update
        let ttl_after: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(-1);

        assert!(
            ttl_after > 0,
            "TTL should be preserved after status update"
        );
        assert!(
            ttl_after <= ttl_before,
            "TTL should not increase after status update"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_redis_compatibility_ttl_method() {
        // Test: Verify TTL command works (Redis 3.0+ compatible)
        let pool = setup_redis_or_skip!();
        let mut conn = pool.conn();
        let test_key = format!("test:ttl:{}", Uuid::new_v4());

        // Set key with TTL
        let _: () = redis::cmd("SET")
            .arg(&test_key)
            .arg("test_value")
            .arg("EX")
            .arg(100)
            .query_async(&mut conn)
            .await
            .expect("Failed to set key");

        // Get TTL
        let ttl: i64 = redis::cmd("TTL")
            .arg(&test_key)
            .query_async(&mut conn)
            .await
            .expect("Failed to get TTL");

        // TTL should be positive (not -1 or -2)
        assert!(ttl > 0, "TTL should be positive, got: {ttl}");
        assert!(ttl <= 100, "TTL should not exceed set time, got: {ttl}");

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(&test_key)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }
}
