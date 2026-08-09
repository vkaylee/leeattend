#[cfg(test)]
mod tests {
    use crate::queue::job_queue::{
        COUNTER_PREFIX, DLQ_KEY, JOB_TTL_SECS, LOW_QUEUE_WEIGHT, MAX_DLQ_LENGTH, MAX_QUEUE_LENGTH,
        MAX_RECENT_JOBS, MAX_SERIALIZED_JOB_BYTES, MERGE_FLAG_PREFIX, META_PREFIX, QUEUE_HIGH,
        QUEUE_LOW, QUEUE_NORMAL, RESULTS_PREFIX,
    };
    use crate::queue::{
        DateRange, Job, JobMetadata, JobPayload, JobQueue, JobStatus, MergePayload, RedisPool,
        ReportConfig, ReportPayload, ReportType, TaskType,
    };
    use chrono::NaiveDate;
    use redis::AsyncCommands;
    use uuid::Uuid;

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
            Ok(Ok(pool)) => {
                match tokio::time::timeout(tokio::time::Duration::from_secs(5), pool.health_check())
                    .await
                {
                    Ok(Ok(())) => Some(pool),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    macro_rules! setup_redis_or_skip {
        () => {{
            let Some(pool) = get_test_redis().await else {
                return;
            };
            pool
        }};
    }

    #[test]
    fn test_queue_key_constants() {
        assert_eq!(QUEUE_HIGH, "worker:jobs:high");
        assert_eq!(QUEUE_NORMAL, "worker:jobs:normal");
        assert_eq!(QUEUE_LOW, "worker:jobs:low");
        assert_eq!(LOW_QUEUE_WEIGHT, 10);
        assert_eq!(META_PREFIX, "report:meta:");
        assert_eq!(RESULTS_PREFIX, "report:results:");
        assert_eq!(COUNTER_PREFIX, "report:counter:");
        assert_eq!(DLQ_KEY, "report:dlq");
        assert_eq!(JOB_TTL_SECS, 86400); // 24 hours
    }

    #[test]
    fn test_redis_resource_limits_are_bounded() {
        const {
            assert!(MAX_QUEUE_LENGTH > 0);
            assert!(MAX_DLQ_LENGTH > 0);
            assert!(MAX_RECENT_JOBS > 0);
            assert!(MAX_SERIALIZED_JOB_BYTES >= 1024);
            assert!(MAX_RECENT_JOBS <= 1000);
        }
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

        assert_eq!(
            counter_key,
            "report:counter:550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(
            failed_key,
            "report:counter:550e8400-e29b-41d4-a716-446655440000:failed"
        );
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
        let mut sorted = fields;
        sorted.sort();

        assert_eq!(sorted[0], "batch_00000");
        assert_eq!(sorted[1], "batch_00001");
        assert_eq!(sorted[2], "batch_00009");
        assert_eq!(sorted[3], "batch_00010");
        assert_eq!(sorted[4], "batch_00100");
    }

    #[test]
    fn test_job_serialization_for_queue() {
        let job = Job::new(
            Uuid::new_v4(),
            5,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: crate::models::RoundingPolicy::Floor,
                    work_unit_minutes: 30,
                },
            }),
        );

        // Jobs should serialize to JSON for Redis (snake_case)
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("attendance_report"));
        assert!(json.contains("summary"));

        // Verify roundtrip
        let decoded: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(job.job_id, decoded.job_id);
        assert_eq!(job.batch_index, decoded.batch_index);
    }

    #[test]
    fn test_data_key_format() {
        let job_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let data_key = format!("{RESULTS_PREFIX}{job_id}:data");
        let filename_key = format!("{RESULTS_PREFIX}{job_id}:filename");
        assert_eq!(
            data_key,
            "report:results:550e8400-e29b-41d4-a716-446655440000:data"
        );
        assert_eq!(
            filename_key,
            "report:results:550e8400-e29b-41d4-a716-446655440000:filename"
        );
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
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
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
            report_type: Some(ReportType::Detail),
            date_range: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
            task_type: None,
            payload: None,
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
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
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
            report_type: Some(ReportType::Detail),
            date_range: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
            task_type: None,
            payload: None,
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
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
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
            report_type: Some(ReportType::Detail),
            date_range: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
            task_type: None,
            payload: None,
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
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
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
            report_type: Some(ReportType::Detail),
            date_range: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
            task_type: None,
            payload: None,
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

        assert!(ttl_after > 0, "TTL should be preserved after status update");
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
        let _test_ns = Uuid::new_v4().to_string();
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

    // ====== BLPOP HEAD-OF-LINE BLOCKING FIX TESTS ======
    //
    // These tests verify the dual-connection architecture that prevents
    // BLPOP from blocking other Redis operations.
    //
    // Background: ConnectionManager in redis-rs is a single multiplexed TCP connection.
    // BLPOP blocks the entire pipeline for its timeout duration (up to 30s),
    // causing all concurrent HSET/PUBLISH/GET operations to wait.
    //
    // Fix: RedisPool now has two ConnectionManagers:
    //   - conn()          → non-blocking operations (HSET, GET, PUBLISH, etc.)
    //   - blocking_conn() → blocking operations (BLPOP only)

    #[tokio::test]
    async fn test_redis_pool_has_dual_connections() {
        // Test: RedisPool should successfully create both connection managers
        let pool = setup_redis_or_skip!();
        let _test_ns = Uuid::new_v4().to_string();

        // Both connections should be usable independently
        let mut conn = pool.conn();
        let mut blocking = pool.blocking_conn();

        // PING should work on both connections
        let pong1: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .expect("conn() PING failed");
        assert_eq!(pong1, "PONG");

        let pong2: String = redis::cmd("PING")
            .query_async(&mut blocking)
            .await
            .expect("blocking_conn() PING failed");
        assert_eq!(pong2, "PONG");
    }

    #[tokio::test]
    async fn test_conn_and_blocking_conn_are_independent() {
        // Test: Operations on conn() and blocking_conn() should not interfere
        let pool = setup_redis_or_skip!();
        let _test_ns = Uuid::new_v4().to_string();
        let test_key_1 = format!("test:dual:conn:{}", Uuid::new_v4());
        let test_key_2 = format!("test:dual:blocking:{}", Uuid::new_v4());

        let mut conn = pool.conn();
        let mut blocking = pool.blocking_conn();

        // Write via conn(), read via blocking_conn() — both see the same Redis instance
        let _: () = redis::cmd("SET")
            .arg(&test_key_1)
            .arg("from_conn")
            .arg("EX")
            .arg(10)
            .query_async(&mut conn)
            .await
            .expect("SET via conn() failed");

        let _: () = redis::cmd("SET")
            .arg(&test_key_2)
            .arg("from_blocking")
            .arg("EX")
            .arg(10)
            .query_async(&mut blocking)
            .await
            .expect("SET via blocking_conn() failed");

        // Cross-read: both connections see each other's writes
        let val1: String = redis::cmd("GET")
            .arg(&test_key_1)
            .query_async(&mut blocking)
            .await
            .expect("GET via blocking_conn() failed");
        assert_eq!(val1, "from_conn");

        let val2: String = redis::cmd("GET")
            .arg(&test_key_2)
            .query_async(&mut conn)
            .await
            .expect("GET via conn() failed");
        assert_eq!(val2, "from_blocking");

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(&test_key_1)
            .arg(&test_key_2)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_dequeue_job_uses_blocking_conn() {
        // Test: dequeue_job should return None on timeout without blocking conn()
        // This is an integration test that proves BLPOP uses the dedicated connection.
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);

        // Ensure the queue is empty (use a very short timeout)
        let result = queue
            .dequeue_job(1.0) // 1 second timeout
            .await
            .expect("dequeue_job should not error on empty queue");

        assert!(
            result.is_none(),
            "dequeue_job should return None on empty queue"
        );
    }

    #[tokio::test]
    async fn test_blpop_does_not_block_concurrent_operations() {
        // CRITICAL TEST: This is the regression test for the BLPOP head-of-line blocking bug.
        //
        // Before the fix, this test would take ~2 seconds because BLPOP on the shared
        // connection would block the concurrent SET operation.
        // After the fix, SET completes in <10ms while BLPOP waits on its own connection.
        let pool = setup_redis_or_skip!();
        let _test_ns = Uuid::new_v4().to_string();
        let test_key = format!("test:blpop_nonblock:{}", Uuid::new_v4());

        // Start BLPOP on blocking_conn (will wait 2 seconds)
        let blocking_pool = pool.clone();
        let blpop_handle = tokio::spawn(async move {
            let mut blocking = blocking_pool.blocking_conn();
            let dummy_key = format!("test:blpop_queue:{}", Uuid::new_v4());
            let _: Option<(String, String)> = redis::cmd("BLPOP")
                .arg(&dummy_key)
                .arg(2) // 2 second timeout
                .query_async(&mut blocking)
                .await
                .unwrap_or(None);
        });

        // Give BLPOP a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Now try a SET on conn() — this should complete INSTANTLY, not wait for BLPOP
        let start = std::time::Instant::now();
        let mut conn = pool.conn();
        let _: () = redis::cmd("SET")
            .arg(&test_key)
            .arg("not_blocked")
            .arg("EX")
            .arg(10)
            .query_async(&mut conn)
            .await
            .expect("SET should succeed while BLPOP is running on blocking_conn");

        let elapsed = start.elapsed();

        // The SET should complete in well under 1 second.
        // Before the fix, it would take ~2 seconds (waiting for BLPOP timeout).
        assert!(
            elapsed.as_millis() < 500,
            "SET on conn() should NOT be blocked by BLPOP on blocking_conn(). \
             Elapsed: {}ms (should be <500ms). \
             If this fails, BLPOP is likely sharing conn() — check dequeue_job().",
            elapsed.as_millis()
        );

        // Verify the SET actually worked
        let val: String = redis::cmd("GET")
            .arg(&test_key)
            .query_async(&mut conn)
            .await
            .expect("GET should succeed");
        assert_eq!(val, "not_blocked");

        // Wait for BLPOP to finish
        blpop_handle.await.ok();

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(&test_key)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_concurrent_store_while_dequeue_running() {
        // Test: store_result should complete quickly even while dequeue_job is waiting.
        // This simulates the real-world scenario where 6 workers run concurrently,
        // some storing results while the main loop dequeues new jobs.
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let job_id = Uuid::new_v4();

        // Start dequeue on a separate task (will wait 2 seconds on empty queue)
        let queue_for_dequeue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let dequeue_handle = tokio::spawn(async move {
            let _ = queue_for_dequeue.dequeue_job(2.0).await;
        });

        // Give dequeue a moment to start BLPOP
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // store_result should complete instantly (uses conn(), not blocking_conn())
        let start = std::time::Instant::now();
        queue
            .store_result(job_id, 0, r#"{"test": "data"}"#)
            .await
            .expect("store_result should work while dequeue is running");
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 500,
            "store_result should NOT be blocked by concurrent dequeue_job. \
             Elapsed: {}ms (should be <500ms)",
            elapsed.as_millis()
        );

        // Wait for dequeue to finish
        dequeue_handle.await.ok();

        // Cleanup
        let mut conn = pool.conn();
        let results_key = format!("{RESULTS_PREFIX}{job_id}");
        let _: () = redis::cmd("DEL")
            .arg(&results_key)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    // ====== PRIORITY QUEUE ROUTING TESTS ======

    #[tokio::test]
    async fn test_enqueue_rejects_oversized_serialized_job_without_write() {
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();
        let key = queue.queue_low();
        let oversized = "x".repeat(MAX_SERIALIZED_JOB_BYTES + 1);
        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: None,
                raw_data: Some(serde_json::json!({ "oversized": oversized })),
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
                msg_id: None,
                agent_id: None,
                source_timestamp: None,
                source_timezone: None,
                timestamp_kind: None,
            })),
            crate::queue::JobPriority::Low,
        );

        let result = queue.enqueue_single(job).await;
        assert!(result.is_err(), "oversized job must be rejected");
        let length: usize = conn.llen(&key).await.unwrap_or(0);
        assert_eq!(length, 0, "oversized job must not be written");
        let _: () = conn.del(&key).await.unwrap_or(());
    }

    #[tokio::test]
    async fn test_enqueue_rejects_when_injected_capacity_is_full() {
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_capacity(pool.clone(), &test_ns, 2);
        let mut conn = pool.conn();
        let key = queue.queue_low();
        let _: () = conn.del(&key).await.unwrap_or(());

        for _ in 0..2 {
            let job = Job::with_priority(
                Uuid::new_v4(),
                0,
                TaskType::LogIngestion,
                Uuid::new_v4(),
                JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                    workspace_id: Uuid::new_v4(),
                    employee_id_number: "EMP001".to_string(),
                    punch_time_raw: "2026-01-26 09:00:00".to_string(),
                    device_id: None,
                    raw_data: None,
                    timezone: "UTC".to_string(),
                    date_fmt: "YYYY-MM-DD".to_string(),
                    time_fmt: "24h".to_string(),
                    msg_id: None,
                    agent_id: None,
                    source_timestamp: None,
                    source_timezone: None,
                    timestamp_kind: None,
                })),
                crate::queue::JobPriority::Low,
            );
            queue
                .enqueue_single(job)
                .await
                .expect("first two jobs should fit");
        }

        let third = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: None,
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
                msg_id: None,
                agent_id: None,
                source_timestamp: None,
                source_timezone: None,
                timestamp_kind: None,
            })),
            crate::queue::JobPriority::Low,
        );
        assert!(
            queue.enqueue_single(third).await.is_err(),
            "third job must be rejected"
        );
        let length: usize = conn.llen(&key).await.unwrap_or(0);
        assert_eq!(length, 2);
        let _: () = conn.del(&key).await.unwrap_or(());
    }

    #[tokio::test]
    async fn test_dlq_write_is_bounded() {
        let pool = setup_redis_or_skip!();
        let mut conn = pool.conn();
        let _: () = conn.del(DLQ_KEY).await.unwrap_or(());
        let queue = JobQueue::new(pool.clone());
        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: None,
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
                msg_id: None,
                agent_id: None,
                source_timestamp: None,
                source_timezone: None,
                timestamp_kind: None,
            })),
            crate::queue::JobPriority::Low,
        );

        for _ in 0..=MAX_DLQ_LENGTH {
            queue
                .move_to_dlq(&job, "test failure")
                .await
                .expect("DLQ write should succeed");
        }
        let length: usize = conn.llen(DLQ_KEY).await.unwrap_or(0);
        assert_eq!(length, MAX_DLQ_LENGTH);
        let _: () = conn.del(DLQ_KEY).await.unwrap_or(());
    }

    #[tokio::test]
    async fn test_enqueue_routes_high_priority_to_high_queue() {
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();

        // Clean up before test
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());

        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: crate::models::RoundingPolicy::Floor,
                    work_unit_minutes: 30,
                },
            }),
            crate::queue::JobPriority::High,
        );

        queue
            .enqueue_jobs(vec![job])
            .await
            .expect("enqueue should succeed");

        let high_len: usize = conn.llen(queue.queue_high()).await.unwrap_or(0);
        let normal_len: usize = conn.llen(queue.queue_normal()).await.unwrap_or(0);
        let low_len: usize = conn.llen(queue.queue_low()).await.unwrap_or(0);

        assert!(high_len >= 1, "High-priority job should be in high queue");
        assert_eq!(normal_len, 0, "Normal queue should be empty");
        assert_eq!(low_len, 0, "Low queue should be empty");

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_enqueue_routes_low_priority_to_low_queue() {
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();

        // Clean up before test
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());

        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: None,
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
                msg_id: None,
                agent_id: None,
                source_timestamp: None,
                source_timezone: None,
                timestamp_kind: None,
            })),
            crate::queue::JobPriority::Low,
        );

        queue
            .enqueue_single(job)
            .await
            .expect("enqueue should succeed");

        let high_len: usize = conn.llen(queue.queue_high()).await.unwrap_or(0);
        let normal_len: usize = conn.llen(queue.queue_normal()).await.unwrap_or(0);
        let low_len: usize = conn.llen(queue.queue_low()).await.unwrap_or(0);

        assert_eq!(high_len, 0, "High queue should be empty");
        assert_eq!(normal_len, 0, "Normal queue should be empty");
        assert!(low_len >= 1, "Low-priority job should be in low queue");

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_dequeue_serves_high_priority_first() {
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();

        // Clean up before test
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());

        // Enqueue LOW job first
        let low_job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "LOW_EMP".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: None,
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
                msg_id: None,
                agent_id: None,
                source_timestamp: None,
                source_timezone: None,
                timestamp_kind: None,
            })),
            crate::queue::JobPriority::Low,
        );
        queue.enqueue_single(low_job).await.unwrap();

        // Then enqueue HIGH job
        let high_job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: crate::models::RoundingPolicy::Floor,
                    work_unit_minutes: 30,
                },
            }),
            crate::queue::JobPriority::High,
        );
        queue.enqueue_single(high_job.clone()).await.unwrap();

        // Dequeue should return HIGH first, even though LOW was enqueued first
        // (BLPOP checks high→normal→low, so High is found before Low)
        let first = queue
            .dequeue_job(1.0)
            .await
            .unwrap()
            .expect("should dequeue");
        assert_eq!(
            first.task_type,
            TaskType::AttendanceReport,
            "First dequeued job should be HIGH priority (AttendanceReport)"
        );
        assert_eq!(first.job_id, high_job.job_id);

        // Second dequeue: high+normal are empty, so low is returned via BLPOP fallback
        let second = queue
            .dequeue_job(1.0)
            .await
            .unwrap()
            .expect("should dequeue");
        assert_eq!(
            second.task_type,
            TaskType::LogIngestion,
            "Second dequeued job should be LOW priority (LogIngestion)"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_queue_length_sums_all_queues() {
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();

        // Clean up before test
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());

        // Enqueue 1 high, 1 normal, 2 low
        let make_high = || {
            Job::with_priority(
                Uuid::new_v4(),
                0,
                TaskType::AttendanceReport,
                Uuid::new_v4(),
                JobPayload::AttendanceReport(ReportPayload {
                    employee_ids: vec![],
                    date_range: DateRange {
                        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                        end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                    },
                    config: ReportConfig {
                        report_type: ReportType::Summary,
                        timezone: "UTC".to_string(),
                        rounding_policy: crate::models::RoundingPolicy::Floor,
                        work_unit_minutes: 30,
                    },
                }),
                crate::queue::JobPriority::High,
            )
        };

        let make_normal = || {
            Job::with_priority(
                Uuid::new_v4(),
                0,
                TaskType::LeaveAccrual,
                Uuid::new_v4(),
                JobPayload::LeaveAccrual(crate::queue::AccrualPayload {
                    date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                    force: false,
                    employee_ids: None,
                }),
                crate::queue::JobPriority::Normal,
            )
        };

        let make_low = || {
            Job::with_priority(
                Uuid::new_v4(),
                0,
                TaskType::LogIngestion,
                Uuid::new_v4(),
                JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                    workspace_id: Uuid::new_v4(),
                    employee_id_number: "E".to_string(),
                    punch_time_raw: "2026-01-01 00:00:00".to_string(),
                    device_id: None,
                    raw_data: None,
                    timezone: "UTC".to_string(),
                    date_fmt: "YYYY-MM-DD".to_string(),
                    time_fmt: "24h".to_string(),
                    msg_id: None,
                    agent_id: None,
                    source_timestamp: None,
                    source_timezone: None,
                    timestamp_kind: None,
                })),
                crate::queue::JobPriority::Low,
            )
        };

        queue.enqueue_single(make_high()).await.unwrap();
        queue.enqueue_single(make_normal()).await.unwrap();
        queue.enqueue_single(make_low()).await.unwrap();
        queue.enqueue_single(make_low()).await.unwrap();

        let total = queue.queue_length().await.unwrap();
        assert_eq!(total, 4, "Total queue length should sum all three queues");

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_normal_priority_goes_to_normal_queue() {
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();

        // Clean up before test
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());

        // Normal priority Job (via Job::new which defaults to Normal)
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::LeaveAccrual,
            Uuid::new_v4(),
            JobPayload::LeaveAccrual(crate::queue::AccrualPayload {
                date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                force: false,
                employee_ids: None,
            }),
        );

        // priority should be Normal
        assert_eq!(job.priority, crate::queue::JobPriority::Normal);

        queue.enqueue_single(job).await.unwrap();

        let high_len: usize = conn.llen(queue.queue_high()).await.unwrap_or(0);
        let normal_len: usize = conn.llen(queue.queue_normal()).await.unwrap_or(0);
        let low_len: usize = conn.llen(queue.queue_low()).await.unwrap_or(0);

        assert_eq!(high_len, 0, "High queue should remain empty");
        assert!(
            normal_len >= 1,
            "Normal priority should route to normal queue"
        );
        assert_eq!(low_len, 0, "Low queue should remain empty");

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_dequeue_high_before_normal() {
        // High and Normal are separate queues; BLPOP must prefer High first.
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();

        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());

        // Enqueue NORMAL first (would be first-in if same queue)
        let normal_job_id = Uuid::new_v4();
        queue
            .enqueue_single(Job::with_priority(
                normal_job_id,
                0,
                TaskType::LeaveAccrual,
                Uuid::new_v4(),
                JobPayload::LeaveAccrual(crate::queue::AccrualPayload {
                    date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                    force: false,
                    employee_ids: None,
                }),
                crate::queue::JobPriority::Normal,
            ))
            .await
            .unwrap();

        // Then enqueue HIGH
        let high_job_id = Uuid::new_v4();
        queue
            .enqueue_single(Job::with_priority(
                high_job_id,
                0,
                TaskType::AttendanceReport,
                Uuid::new_v4(),
                JobPayload::AttendanceReport(ReportPayload {
                    employee_ids: vec![],
                    date_range: DateRange {
                        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                        end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                    },
                    config: ReportConfig {
                        report_type: ReportType::Summary,
                        timezone: "UTC".to_string(),
                        rounding_policy: crate::models::RoundingPolicy::Floor,
                        work_unit_minutes: 30,
                    },
                }),
                crate::queue::JobPriority::High,
            ))
            .await
            .unwrap();

        // First dequeue: must be HIGH, not NORMAL (BLPOP checks high first)
        let first = queue
            .dequeue_job(1.0)
            .await
            .unwrap()
            .expect("should dequeue");
        assert_eq!(
            first.job_id, high_job_id,
            "High must dequeue before Normal (BLPOP order: high→normal)"
        );

        // Second dequeue: should be Normal
        let second = queue
            .dequeue_job(1.0)
            .await
            .unwrap()
            .expect("should dequeue");
        assert_eq!(
            second.job_id, normal_job_id,
            "Normal should dequeue after High"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_dequeue_normal_before_low_on_regular_turn() {
        // On non-low-turn dequeues, BLPOP checks high→normal→low,
        // so Normal must be returned before Low.
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();

        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());

        // Enqueue LOW first
        let low_job_id = Uuid::new_v4();
        queue
            .enqueue_single(Job::with_priority(
                low_job_id,
                0,
                TaskType::LogIngestion,
                Uuid::new_v4(),
                JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                    workspace_id: Uuid::new_v4(),
                    employee_id_number: "ORDER_TEST".to_string(),
                    punch_time_raw: "2026-01-01 00:00:00".to_string(),
                    device_id: None,
                    raw_data: None,
                    timezone: "UTC".to_string(),
                    date_fmt: "YYYY-MM-DD".to_string(),
                    time_fmt: "24h".to_string(),
                    msg_id: None,
                    agent_id: None,
                    source_timestamp: None,
                    source_timezone: None,
                    timestamp_kind: None,
                })),
                crate::queue::JobPriority::Low,
            ))
            .await
            .unwrap();

        // Then enqueue NORMAL
        let normal_job_id = Uuid::new_v4();
        queue
            .enqueue_single(Job::with_priority(
                normal_job_id,
                0,
                TaskType::MergeResults,
                Uuid::new_v4(),
                JobPayload::MergeResults(MergePayload {
                    date_range: DateRange {
                        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                        end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                    },
                    config: ReportConfig {
                        report_type: ReportType::Summary,
                        timezone: "UTC".to_string(),
                        rounding_policy: crate::models::RoundingPolicy::Floor,
                        work_unit_minutes: 30,
                    },
                }),
                crate::queue::JobPriority::Normal,
            ))
            .await
            .unwrap();

        // Drain until we get a non-low-turn dequeue.
        // On a regular turn, BLPOP(high→normal→low) should return Normal first.
        // On a low-turn, LPOP from low would take the low job first —
        // that's expected behavior. We wait for either outcome.
        let mut normal_first = false;
        let mut low_first = false;
        let low_job_id_captured = low_job_id;
        let normal_job_id_captured = normal_job_id;

        for _ in 0..5 {
            let job = queue
                .dequeue_job(0.5)
                .await
                .unwrap()
                .expect("should dequeue");
            if job.job_id == normal_job_id_captured {
                normal_first = true;
                // Next dequeue should be low
                let next = queue
                    .dequeue_job(0.5)
                    .await
                    .unwrap()
                    .expect("should dequeue");
                assert_eq!(
                    next.job_id, low_job_id_captured,
                    "After Normal, Low should follow"
                );
                break;
            }
            if job.job_id == low_job_id_captured {
                low_first = true;
                // This was a low-turn — the next dequeue should be normal
                let next = queue
                    .dequeue_job(0.5)
                    .await
                    .unwrap()
                    .expect("should dequeue");
                assert_eq!(
                    next.job_id, normal_job_id_captured,
                    "After Low (low-turn), Normal should follow"
                );
                break;
            }
        }

        assert!(
            normal_first || low_first,
            "Either Normal or Low should have been dequeued"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_dequeue_fairness_low_gets_slot() {
        // Verify weighted fairness: when high/normal queues are full,
        // every LOW_QUEUE_WEIGHT-th dequeue polls low first.
        let pool = setup_redis_or_skip!();
        let test_ns = Uuid::new_v4().to_string();
        let queue = JobQueue::with_namespace(pool.clone(), &test_ns);
        let mut conn = pool.conn();

        // Clean up before test
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());

        // Enqueue lots of high-priority jobs (always dequeued first)
        let make_high = || {
            Job::with_priority(
                Uuid::new_v4(),
                0,
                TaskType::AttendanceReport,
                Uuid::new_v4(),
                JobPayload::AttendanceReport(ReportPayload {
                    employee_ids: vec![],
                    date_range: DateRange {
                        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                        end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                    },
                    config: ReportConfig {
                        report_type: ReportType::Summary,
                        timezone: "UTC".to_string(),
                        rounding_policy: crate::models::RoundingPolicy::Floor,
                        work_unit_minutes: 30,
                    },
                }),
                crate::queue::JobPriority::High,
            )
        };

        // Enqueue one low job
        let low_job_id = Uuid::new_v4();
        let low_job = Job::with_priority(
            low_job_id,
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "FAIR_TEST".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: None,
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
                msg_id: None,
                agent_id: None,
                source_timestamp: None,
                source_timezone: None,
                timestamp_kind: None,
            })),
            crate::queue::JobPriority::Low,
        );
        queue.enqueue_single(low_job).await.unwrap();

        // Enqueue enough high jobs to keep the queue busy
        for _ in 0..20 {
            queue.enqueue_single(make_high()).await.unwrap();
        }

        // Drain until we find the low job — it MUST appear within
        // 20 dequeue attempts (LOW_QUEUE_WEIGHT=10 means at least 2 low turns)
        let mut found_low = false;
        for _attempt in 0..20 {
            let job = queue
                .dequeue_job(0.5)
                .await
                .unwrap()
                .expect("should dequeue a job");
            if job.job_id == low_job_id {
                found_low = true;
                break;
            }
        }

        assert!(
            found_low,
            "Low-priority job should be dequeued within 20 attempts; \
             weighted fairness (LOW_QUEUE_WEIGHT={LOW_QUEUE_WEIGHT}) should guarantee it"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_high())
            .arg(queue.queue_normal())
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }

    /// When only low-priority jobs are in the queue, every 10th dequeue
    /// triggers a non-blocking LPOP from `queue:low`. Before the fix,
    /// LPOP returned `Option<String>` but the code tried to deserialize as
    /// `Option<(String, String)>`, causing a Redis `TypeError`.
    ///
    /// This test enqueues 12 low-priority jobs and dequeues them all,
    /// proving that LPOP (hit at least once in 12 attempts when
    /// LOW_QUEUE_WEIGHT=10) succeeds with the corrected return type.
    #[tokio::test]
    async fn test_dequeue_low_priority_lpop_type_fix() {
        let pool = setup_redis_or_skip!();
        let queue = JobQueue::with_namespace(pool.clone(), &Uuid::new_v4().to_string());

        let mut conn = pool.conn();
        let ws = Uuid::new_v4();

        // Enqueue 12 low-priority jobs — enough to guarantee LPOP is hit
        // at least once (LOW_QUEUE_WEIGHT=10, so dequeue #11 triggers LPOP).
        let mut job_ids = Vec::new();
        for i in 0..12 {
            let id = Uuid::new_v4();
            job_ids.push(id);
            let job = Job::with_priority(
                id,
                0,
                TaskType::LogIngestion,
                ws,
                JobPayload::LogIngestion(Box::new(crate::queue::LogIngestionPayload {
                    workspace_id: ws,
                    employee_id_number: format!("EMP{i:03}"),
                    punch_time_raw: "2026-01-26 09:00:00".to_string(),
                    device_id: None,
                    raw_data: None,
                    timezone: "UTC".to_string(),
                    date_fmt: "YYYY-MM-DD".to_string(),
                    time_fmt: "24h".to_string(),
                    msg_id: None,
                    agent_id: None,
                    source_timestamp: None,
                    source_timezone: None,
                    timestamp_kind: None,
                })),
                crate::queue::JobPriority::Low,
            );
            queue.enqueue_single(job).await.unwrap();
        }

        // Dequeue all 12 jobs — must succeed without Redis TypeError.
        let mut dequeued = 0u32;
        for _ in 0..12 {
            let result = queue.dequeue_job(0.5).await;
            assert!(
                result.is_ok(),
                "dequeue_job must not fail with Redis error; got {:?}",
                result.err()
            );
            let job = result.unwrap().expect("should dequeue a job");
            assert!(
                job_ids.contains(&job.job_id),
                "dequeued unexpected job {}",
                job.job_id
            );
            dequeued += 1;
        }

        assert_eq!(dequeued, 12, "all 12 low-priority jobs must be dequeued");

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg(queue.queue_low())
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or(());
    }
}
