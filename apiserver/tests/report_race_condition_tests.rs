mod common;

use apiserver::queue::{JobMetadata, JobStatus, RedisPool, ReportQueue, ReportType};
use chrono::Utc;
use common::db::TestDatabase;
use std::sync::Arc;
use tokio::task::JoinSet;
use uuid::Uuid;

// Helper to create test Redis pool or skip tests
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
            _ => {
                eprintln!("WARNING: Redis health check failed. Tests will skip.");
                None
            }
        },
        _ => {
            eprintln!("WARNING: Redis is not available. Skipping Redis-dependent tests.");
            None
        }
    }
}

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

/// TEST 1: Verify merge flag prevents double merge trigger
/// Simulates race condition where multiple workers complete batches concurrently
#[tokio::test]
async fn test_merge_flag_prevents_double_trigger() {
    let redis_pool = setup_redis_or_skip!();
    let queue = Arc::new(ReportQueue::new(redis_pool.clone()));

    let job_id = Uuid::new_v4();

    // Setup: Create job metadata
    let metadata = JobMetadata {
        job_id,
        status: JobStatus::Processing,
        total_batches: 4,
        completed_batches: 0,
        failed_batches: 0,
        workspace_id: Uuid::new_v4(),
        requester_id: Uuid::new_v4(),
        report_type: ReportType::Detail,
        date_range: None,
        error_message: None,
        created_at: Utc::now(),
        completed_at: None,
    };

    queue
        .create_job_metadata(&metadata)
        .await
        .expect("Failed to create job metadata");

    // Simulate concurrent workers trying to trigger merge
    let mut handles = JoinSet::new();

    for worker_id in 0..5 {
        let queue_clone = queue.clone();
        let job_id_clone = job_id;

        handles.spawn(async move {
            // Each worker tries to set merge trigger flag
            let result = queue_clone
                .try_trigger_merge(job_id_clone)
                .await
                .expect("Failed to try trigger merge");

            (worker_id, result)
        });
    }

    // Collect results
    let mut success_count = 0;
    let mut results = Vec::new();

    while let Some(result) = handles.join_next().await {
        let (worker_id, was_set) = result.expect("Task panicked");
        results.push((worker_id, was_set));
        if was_set {
            success_count += 1;
        }
    }

    // Assert: Only ONE worker should successfully set the flag
    assert_eq!(
        success_count, 1,
        "Expected exactly 1 worker to trigger merge, got {}. Results: {:?}",
        success_count, results
    );

    // Verify flag is set in Redis
    let flag_set_again = queue
        .try_trigger_merge(job_id)
        .await
        .expect("Failed to check flag again");
    assert!(!flag_set_again, "Flag should remain set after first trigger");

    // Cleanup
    queue.cleanup_job(job_id).await.ok();
}

/// TEST 2: Verify merge flag is cleared during cleanup
#[tokio::test]
async fn test_merge_flag_cleanup() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool);

    let job_id = Uuid::new_v4();

    // Set merge flag
    let was_set = queue
        .try_trigger_merge(job_id)
        .await
        .expect("Failed to set flag");
    assert!(was_set, "Should successfully set merge flag");

    // Clean up job
    queue
        .cleanup_job(job_id)
        .await
        .expect("Failed to cleanup job");

    // Try setting flag again - should succeed since it was cleaned up
    let was_set_after_cleanup = queue
        .try_trigger_merge(job_id)
        .await
        .expect("Failed to set flag after cleanup");
    assert!(
        was_set_after_cleanup,
        "Should be able to set flag again after cleanup"
    );

    // Cleanup again
    queue.cleanup_job(job_id).await.ok();
}

/// TEST 3: Simulate realistic race condition with batch completion
#[tokio::test]
async fn test_concurrent_batch_completion_no_double_merge() {
    let test_db = TestDatabase::new().await;
    let _pool = test_db.pool();
    let redis_pool = setup_redis_or_skip!();

    let queue = Arc::new(ReportQueue::new(redis_pool.clone()));
    let job_id = Uuid::new_v4();
    let total_batches = 4u32;

    // Setup: Create job metadata
    let metadata = JobMetadata {
        job_id,
        status: JobStatus::Processing,
        total_batches,
        completed_batches: 0,
        failed_batches: 0,
        workspace_id: Uuid::new_v4(),
        requester_id: Uuid::new_v4(),
        report_type: ReportType::Summary,
        date_range: None,
        error_message: None,
        created_at: Utc::now(),
        completed_at: None,
    };

    queue
        .create_job_metadata(&metadata)
        .await
        .expect("Failed to create job metadata");

    // Simulate concurrent batch completions
    let mut handles = JoinSet::new();
    let trigger_count = Arc::new(std::sync::atomic::AtomicU32::new(0));

    for batch_index in 0..total_batches {
        let queue_clone = queue.clone();
        let trigger_count_clone = trigger_count.clone();

        handles.spawn(async move {
            // Increment completed counter
            let completed = queue_clone
                .increment_completed(job_id)
                .await
                .expect("Failed to increment");

            // Check if all done
            if completed >= total_batches {
                // Try to trigger merge (atomic operation)
                let should_trigger = queue_clone
                    .try_trigger_merge(job_id)
                    .await
                    .expect("Failed to try trigger");

                if should_trigger {
                    trigger_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }

            (batch_index, completed)
        });
    }

    // Wait for all workers
    let mut completions = Vec::new();
    while let Some(result) = handles.join_next().await {
        let (batch_index, completed) = result.expect("Task panicked");
        completions.push((batch_index, completed));
    }

    // Assert: Merge should be triggered exactly once
    let final_trigger_count = trigger_count.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        final_trigger_count, 1,
        "Merge should be triggered exactly once, got {}. Completions: {:?}",
        final_trigger_count, completions
    );

    // Verify final counter state
    let final_count = queue
        .get_completed_count(job_id)
        .await
        .expect("Failed to get count");
    assert_eq!(
        final_count, total_batches,
        "Final count should equal total batches"
    );

    // Cleanup
    queue.cleanup_job(job_id).await.ok();
    drop(test_db);
}

/// TEST 4: Verify flag expiration (TTL)
#[tokio::test]
async fn test_merge_flag_has_ttl() {
    let redis_pool = setup_redis_or_skip!();
    let queue = ReportQueue::new(redis_pool.clone());

    let job_id = Uuid::new_v4();

    // Set merge flag
    queue
        .try_trigger_merge(job_id)
        .await
        .expect("Failed to set flag");

    // Check TTL on the Redis key
    let flag_key = format!("report:merge_triggered:{}", job_id);
    let mut conn = redis_pool.conn();
    let ttl: i64 = redis::AsyncCommands::ttl(&mut conn, &flag_key)
        .await
        .expect("Failed to get TTL from Redis");

    // TTL should be positive and match JOB_TTL_SECS (86400)
    assert!(ttl > 0, "TTL should be positive, got: {}", ttl);
    assert!(
        ttl <= 86400,
        "TTL should not exceed 86400 seconds (24 hours), got: {}",
        ttl
    );

    // Cleanup
    queue.cleanup_job(job_id).await.ok();
}
