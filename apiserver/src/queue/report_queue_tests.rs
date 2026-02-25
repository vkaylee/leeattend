#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{DateRange, JobStatus, RedisPool, ReportConfig, ReportType, TaskType};
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
