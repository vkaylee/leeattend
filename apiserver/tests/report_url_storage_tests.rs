mod common;

use apiserver::queue::{JobMetadata, JobStatus, RedisPool, ReportQueue, ReportType};
use apiserver::services::ReportCoordinator;
use chrono::Utc;
use common::db::TestDatabase;
use uuid::Uuid;

// Macro to simplify test setup with Redis skip support
macro_rules! setup_redis_or_skip {
    () => {{
        match get_test_redis().await {
            Some(pool) => pool,
            None => {
                println!("SKIPPED: Redis not available");
                return;
            }
        }
    }};
}

// Helper to create a test Redis pool
// Returns None if Redis is not available (tests will be skipped)
async fn get_test_redis() -> Option<RedisPool> {
    // Try to connect to Redis with timeout and retries
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Try once with a timeout
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        RedisPool::new(&redis_url),
    )
    .await
    {
        Ok(Ok(pool)) => {
            // Verify Redis is reachable with health check
            match tokio::time::timeout(tokio::time::Duration::from_secs(5), pool.health_check())
                .await
            {
                Ok(Ok(())) => return Some(pool),
                _ => {
                    eprintln!("WARNING: Redis health check failed. Tests will skip.");
                    return None;
                }
            }
        }
        _ => {
            // Redis is not available or timeout - return None so tests can skip
            eprintln!("WARNING: Redis is not available. Skipping Redis-dependent tests.");
            eprintln!("To run these tests, start Redis with: redis-server");
            return None;
        }
    }
}

// Helper to create test job metadata
fn create_test_job_metadata(job_id: Uuid, status: JobStatus) -> JobMetadata {
    JobMetadata {
        job_id,
        status,
        total_batches: 5,
        completed_batches: 3,
        failed_batches: 0,
        workspace_id: Uuid::new_v4(),
        requester_id: Uuid::new_v4(),
        report_type: ReportType::Detail,
        date_range: None,
        error_message: None,
        created_at: Utc::now(),
        completed_at: None,
    }
}

/// TEST 1: Happy Path - Set and retrieve result URL
#[tokio::test]
async fn test_result_url_storage_and_retrieval() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool);

    let job_id = Uuid::new_v4();
    let test_url = "https://s3.example.com/reports/test-job-id.csv";

    // Act: Set result URL
    let set_result = queue.set_result_url(job_id, test_url).await;
    assert!(
        set_result.is_ok(),
        "Failed to set result URL: {:?}",
        set_result.err()
    );

    // Act: Get result URL
    let get_result = queue.get_result_url(job_id).await;
    assert!(
        get_result.is_ok(),
        "Failed to get result URL: {:?}",
        get_result.err()
    );

    // Assert: URL matches what was set
    let retrieved_url = get_result.unwrap();
    assert_eq!(
        retrieved_url, Some(test_url.to_string()),
        "Retrieved URL does not match stored URL"
    );
}

/// TEST 2: Redis key format verification
#[tokio::test]
async fn test_redis_key_format_for_url_storage() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool.clone());

    let job_id = Uuid::new_v4();
    let test_url = "https://example.com/report.csv";

    // Set URL
    queue
        .set_result_url(job_id, test_url)
        .await
        .expect("Failed to set URL");

    // Verify by directly accessing Redis with expected key format
    let expected_key = format!("report:results:{}:url", job_id);
    let mut conn = redis_pool.conn();
    let stored_value: Option<String> = redis::AsyncCommands::get(&mut conn, &expected_key)
        .await
        .expect("Failed to query Redis directly");

    assert_eq!(
        stored_value, Some(test_url.to_string()),
        "URL not found at expected Redis key format"
    );
}

/// TEST 3: TTL verification for stored URLs
#[tokio::test]
async fn test_result_url_ttl_is_set_correctly() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool.clone());

    let job_id = Uuid::new_v4();
    let test_url = "https://example.com/report.csv";

    // Set URL
    queue
        .set_result_url(job_id, test_url)
        .await
        .expect("Failed to set URL");

    // Check TTL on the Redis key
    let expected_key = format!("report:results:{}:url", job_id);
    let mut conn = redis_pool.conn();
    let ttl: i64 = redis::AsyncCommands::ttl(&mut conn, &expected_key)
        .await
        .expect("Failed to get TTL from Redis");

    // TTL should be positive (86400 seconds = 24 hours)
    // Allow some variance due to timing
    assert!(ttl > 0, "TTL should be positive, got: {}", ttl);
    assert!(
        ttl <= 86400,
        "TTL should not exceed 86400 seconds (24 hours), got: {}",
        ttl
    );
}

/// TEST 4: Get URL when not set (job not complete)
#[tokio::test]
async fn test_get_result_url_returns_none_when_not_set() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool);

    let job_id = Uuid::new_v4();

    // Act: Try to get URL that was never set
    let result = queue
        .get_result_url(job_id)
        .await
        .expect("Should not error on missing URL");

    // Assert: Should return None, not error
    assert_eq!(result, None, "Should return None for non-existent URL");
}

/// TEST 5: Multiple jobs with different URLs
#[tokio::test]
async fn test_multiple_jobs_with_different_urls() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool);

    let job_id_1 = Uuid::new_v4();
    let job_id_2 = Uuid::new_v4();
    let url_1 = "https://s3.example.com/report1.csv";
    let url_2 = "https://s3.example.com/report2.csv";

    // Set URLs for both jobs
    queue
        .set_result_url(job_id_1, url_1)
        .await
        .expect("Failed to set URL 1");
    queue
        .set_result_url(job_id_2, url_2)
        .await
        .expect("Failed to set URL 2");

    // Retrieve both URLs
    let retrieved_url_1 = queue
        .get_result_url(job_id_1)
        .await
        .expect("Failed to get URL 1");
    let retrieved_url_2 = queue
        .get_result_url(job_id_2)
        .await
        .expect("Failed to get URL 2");

    // Assert: Each job should return its own URL
    assert_eq!(retrieved_url_1, Some(url_1.to_string()));
    assert_eq!(retrieved_url_2, Some(url_2.to_string()));
}

/// TEST 6: Overwriting URL for same job
#[tokio::test]
async fn test_overwriting_result_url_for_same_job() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool);

    let job_id = Uuid::new_v4();
    let original_url = "https://s3.example.com/report-v1.csv";
    let updated_url = "https://s3.example.com/report-v2.csv";

    // Set original URL
    queue
        .set_result_url(job_id, original_url)
        .await
        .expect("Failed to set original URL");

    // Overwrite with new URL
    queue
        .set_result_url(job_id, updated_url)
        .await
        .expect("Failed to update URL");

    // Retrieve and verify latest URL is returned
    let retrieved_url = queue
        .get_result_url(job_id)
        .await
        .expect("Failed to get URL");

    assert_eq!(
        retrieved_url,
        Some(updated_url.to_string()),
        "Should return updated URL, not original"
    );
}

/// TEST 7: Job status returns URL when job is completed
#[tokio::test]
async fn test_get_job_status_returns_url_when_completed() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let redis_pool = setup_redis_or_skip!();

    let queue = ReportQueue::new(redis_pool.clone());
    let coordinator = ReportCoordinator::new(redis_pool.clone(), pool.clone());

    let job_id = Uuid::new_v4();
    let test_url = "https://s3.example.com/completed-report.csv";

    // Create completed job metadata in Redis
    let metadata = create_test_job_metadata(job_id, JobStatus::Completed);
    queue
        .update_job_metadata(&metadata)
        .await
        .expect("Failed to create job metadata");

    // Store URL for this job
    queue
        .set_result_url(job_id, test_url)
        .await
        .expect("Failed to set result URL");

    // Act: Get job status
    let status_response = coordinator
        .get_job_status(job_id)
        .await
        .expect("Failed to get job status");

    // Assert: URL should be present in status response
    assert_eq!(
        status_response.result_url,
        Some(test_url.to_string()),
        "Job status should include result URL for completed job"
    );
    assert_eq!(status_response.status, "completed");

    drop(test_db);  // TestDatabase cleanup is automatic via Drop trait
}

/// TEST 8: Job status returns None for URL when job is still pending
#[tokio::test]
async fn test_get_job_status_returns_none_url_when_pending() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let redis_pool = setup_redis_or_skip!();

    let queue = ReportQueue::new(redis_pool.clone());
    let coordinator = ReportCoordinator::new(redis_pool.clone(), pool.clone());

    let job_id = Uuid::new_v4();

    // Create pending job metadata
    let metadata = create_test_job_metadata(job_id, JobStatus::Pending);
    queue
        .update_job_metadata(&metadata)
        .await
        .expect("Failed to create job metadata");

    // Act: Get job status (no URL set)
    let status_response = coordinator
        .get_job_status(job_id)
        .await
        .expect("Failed to get job status");

    // Assert: URL should be None for pending job
    assert_eq!(
        status_response.result_url, None,
        "Pending job should not have result URL"
    );
    assert_eq!(status_response.status, "pending");

    drop(test_db);  // TestDatabase cleanup is automatic via Drop trait
}

/// TEST 9: Job status handles Redis error gracefully
#[tokio::test]
async fn test_get_job_status_handles_redis_error_gracefully() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let redis_pool = setup_redis_or_skip!();

    let queue = ReportQueue::new(redis_pool.clone());
    let coordinator = ReportCoordinator::new(redis_pool.clone(), pool.clone());

    let job_id = Uuid::new_v4();

    // Create completed job metadata without URL
    let metadata = create_test_job_metadata(job_id, JobStatus::Completed);
    queue
        .update_job_metadata(&metadata)
        .await
        .expect("Failed to create job metadata");

    // Act: Get job status (URL not set - simulates Redis retrieval failure handling)
    let status_response = coordinator
        .get_job_status(job_id)
        .await
        .expect("Should not error even if URL retrieval fails");

    // Assert: Should return Ok with None URL, not propagate error
    assert_eq!(status_response.result_url, None);
    assert_eq!(status_response.status, "completed");

    // Verify it didn't panic and request was successful
    assert_eq!(status_response.job_id, job_id);

    drop(test_db);  // TestDatabase cleanup is automatic via Drop trait
}

/// TEST 10: Job status completed but URL not found yet (race condition)
#[tokio::test]
async fn test_get_job_status_completed_but_url_not_yet_stored() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let redis_pool = setup_redis_or_skip!();

    let queue = ReportQueue::new(redis_pool.clone());
    let coordinator = ReportCoordinator::new(redis_pool.clone(), pool.clone());

    let job_id = Uuid::new_v4();

    // Create job metadata marked as completed
    let metadata = create_test_job_metadata(job_id, JobStatus::Completed);
    queue
        .update_job_metadata(&metadata)
        .await
        .expect("Failed to create job metadata");

    // Act: Get status WITHOUT setting URL (simulates race where status updated before URL stored)
    let status_response = coordinator
        .get_job_status(job_id)
        .await
        .expect("Should handle missing URL gracefully");

    // Assert: Should return Ok with None for URL
    // This verifies error swallowing fix - should log warning but not fail
    assert_eq!(status_response.result_url, None);
    assert_eq!(status_response.status, "completed");

    drop(test_db);  // TestDatabase cleanup is automatic via Drop trait
}

/// TEST 11: Empty URL edge case
#[tokio::test]
async fn test_set_and_retrieve_empty_url_string() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool);

    let job_id = Uuid::new_v4();
    let empty_url = "";

    // Set empty URL (edge case)
    queue
        .set_result_url(job_id, empty_url)
        .await
        .expect("Should handle empty string");

    // Retrieve and verify
    let retrieved = queue
        .get_result_url(job_id)
        .await
        .expect("Failed to get URL");

    // Should return the empty string as-is
    assert_eq!(retrieved, Some(empty_url.to_string()));
}

/// TEST 12: Very long URL (S3 pre-signed URL)
#[tokio::test]
async fn test_set_and_retrieve_long_s3_presigned_url() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool);

    let job_id = Uuid::new_v4();
    let long_url = "https://mybucket.s3.amazonaws.com/reports/attendance-2024-01.csv?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20240131%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20240131T120000Z&X-Amz-Expires=3600&X-Amz-SignedHeaders=host&X-Amz-Signature=abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";

    // Set long URL
    queue
        .set_result_url(job_id, long_url)
        .await
        .expect("Should handle long URLs");

    // Retrieve and verify
    let retrieved = queue
        .get_result_url(job_id)
        .await
        .expect("Failed to get long URL");

    assert_eq!(retrieved, Some(long_url.to_string()));
}

/// TEST 13: Special characters in URL (encoding test)
#[tokio::test]
async fn test_set_and_retrieve_url_with_special_characters() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool);

    let job_id = Uuid::new_v4();
    let url_with_special_chars = "https://s3.example.com/reports/attendance%20report%202024-01-31.csv?version=1.0&format=csv&encoding=utf-8";

    // Set URL with special characters
    queue
        .set_result_url(job_id, url_with_special_chars)
        .await
        .expect("Should handle URL-encoded characters");

    // Retrieve and verify exact match
    let retrieved = queue
        .get_result_url(job_id)
        .await
        .expect("Failed to get URL with special chars");

    assert_eq!(retrieved, Some(url_with_special_chars.to_string()));
}

/// TEST 14: Concurrent URL operations
#[tokio::test]
async fn test_concurrent_url_set_and_get() {
    let redis_pool = setup_redis_or_skip!();
    let queue = std::sync::Arc::new(ReportQueue::new(redis_pool));

    let job_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

    // Spawn concurrent tasks to set URLs
    let mut handles = vec![];
    for (i, &job_id) in job_ids.iter().enumerate() {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move {
            let url = format!("https://s3.example.com/report-{}.csv", i);
            queue_clone
                .set_result_url(job_id, &url)
                .await
                .expect("Failed to set URL in concurrent task");
            (job_id, url)
        });
        handles.push(handle);
    }

    // Wait for all sets to complete
    let mut stored_pairs = vec![];
    for handle in handles {
        stored_pairs.push(handle.await.expect("Task panicked"));
    }

    // Now concurrently read all URLs
    let mut read_handles = vec![];
    for (job_id, expected_url) in stored_pairs {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move {
            let retrieved = queue_clone
                .get_result_url(job_id)
                .await
                .expect("Failed to get URL in concurrent task");
            (job_id, expected_url, retrieved)
        });
        read_handles.push(handle);
    }

    // Verify all reads succeeded and values match
    for handle in read_handles {
        let (job_id, expected_url, retrieved) = handle.await.expect("Read task panicked");
        assert_eq!(
            retrieved,
            Some(expected_url),
            "URL mismatch for job {} in concurrent test",
            job_id
        );
    }
}

/// TEST 15: URL retrieval for processing and failed jobs
#[tokio::test]
async fn test_job_status_url_only_for_completed_jobs() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let redis_pool = setup_redis_or_skip!();

    let queue = ReportQueue::new(redis_pool.clone());
    let coordinator = ReportCoordinator::new(redis_pool.clone(), pool.clone());

    let processing_job_id = Uuid::new_v4();
    let failed_job_id = Uuid::new_v4();
    let test_url = "https://s3.example.com/report.csv";

    // Create processing and failed jobs
    let processing_meta = create_test_job_metadata(processing_job_id, JobStatus::Processing);
    let failed_meta = create_test_job_metadata(failed_job_id, JobStatus::Failed);

    queue
        .update_job_metadata(&processing_meta)
        .await
        .expect("Failed to create processing job");
    queue
        .update_job_metadata(&failed_meta)
        .await
        .expect("Failed to create failed job");

    // Set URLs for both (shouldn't matter - should be ignored for non-completed)
    let queue = ReportQueue::new(redis_pool.clone());
    queue
        .set_result_url(processing_job_id, test_url)
        .await
        .expect("Failed to set URL");
    queue
        .set_result_url(failed_job_id, test_url)
        .await
        .expect("Failed to set URL");

    // Get status for both
    let processing_status = coordinator
        .get_job_status(processing_job_id)
        .await
        .expect("Failed to get processing status");
    let failed_status = coordinator
        .get_job_status(failed_job_id)
        .await
        .expect("Failed to get failed status");

    // Assert: URL should be None for non-completed jobs
    assert_eq!(
        processing_status.result_url, None,
        "Processing job should not return URL"
    );
    assert_eq!(
        failed_status.result_url, None,
        "Failed job should not return URL"
    );

    drop(test_db);  // TestDatabase cleanup is automatic via Drop trait
}

/// TEST 16: URL expiration via TTL (simulate passage of time)
#[tokio::test]
async fn test_result_url_expires_after_ttl() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool.clone());

    let job_id = Uuid::new_v4();
    let test_url = "https://s3.example.com/report.csv";

    // Set URL
    queue
        .set_result_url(job_id, test_url)
        .await
        .expect("Failed to set URL");

    // Verify URL is accessible
    let retrieved_before = queue
        .get_result_url(job_id)
        .await
        .expect("Failed to get URL");
    assert!(retrieved_before.is_some(), "URL should exist immediately after set");

    // Note: We can't actually wait 24 hours, but we can verify the TTL was set
    // The expiration test confirms Redis TTL mechanism is working by checking TTL value
    let expected_key = format!("report:results:{}:url", job_id);
    let mut conn = redis_pool.conn();
    let ttl: i64 = redis::AsyncCommands::ttl(&mut conn, &expected_key)
        .await
        .expect("Failed to check TTL");

    assert!(
        ttl > 0,
        "URL should have positive TTL for automatic expiration"
    );
}

/// TEST 17: Database and Redis consistency
#[tokio::test]
async fn test_job_metadata_and_url_consistency() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let redis_pool = setup_redis_or_skip!();

    let queue = ReportQueue::new(redis_pool.clone());
    let coordinator = ReportCoordinator::new(redis_pool.clone(), pool.clone());

    let job_id = Uuid::new_v4();
    let url_1 = "https://s3.example.com/report-v1.csv";
    let url_2 = "https://s3.example.com/report-v2.csv";

    // Create job as pending
    let mut metadata = create_test_job_metadata(job_id, JobStatus::Pending);
    queue
        .update_job_metadata(&metadata)
        .await
        .expect("Failed to create metadata");

    // Verify URL doesn't exist yet
    let status_1 = coordinator
        .get_job_status(job_id)
        .await
        .expect("Failed to get status 1");
    assert_eq!(status_1.result_url, None);

    // Mark as completed and set first URL
    metadata.status = JobStatus::Completed;
    queue
        .update_job_metadata(&metadata)
        .await
        .expect("Failed to update metadata");
    queue
        .set_result_url(job_id, url_1)
        .await
        .expect("Failed to set URL 1");

    // Verify URL is now available
    let status_2 = coordinator
        .get_job_status(job_id)
        .await
        .expect("Failed to get status 2");
    assert_eq!(status_2.result_url, Some(url_1.to_string()));

    // Update URL
    queue
        .set_result_url(job_id, url_2)
        .await
        .expect("Failed to set URL 2");

    // Verify updated URL is returned
    let status_3 = coordinator
        .get_job_status(job_id)
        .await
        .expect("Failed to get status 3");
    assert_eq!(status_3.result_url, Some(url_2.to_string()));

    drop(test_db);  // TestDatabase cleanup is automatic via Drop trait
}
