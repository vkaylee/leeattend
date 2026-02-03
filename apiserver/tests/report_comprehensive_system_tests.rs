mod common;

use apiserver::queue::{JobMetadata, JobStatus, ReportJob, ReportQueue, ReportType, TaskType, DateRange, RedisPool};
use chrono::NaiveDate;
use common::db::TestDatabase;
use uuid::Uuid;

async fn setup_redis() -> (TestDatabase, RedisPool) {
    let test_db = TestDatabase::new().await;
    dotenvy::from_filename(".env.test").ok();
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let pool = RedisPool::new(&redis_url)
        .await
        .expect("Failed to connect to Redis");
    (test_db, pool)
}

fn create_test_meta(job_id: Uuid, workspace_id: Uuid) -> JobMetadata {
    let requester_id = Uuid::new_v4();
    let mut meta = JobMetadata::with_date_range(
        job_id,
        4,
        workspace_id,
        requester_id,
        ReportType::Summary,
        DateRange {
            start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        },
    );
    meta.status = JobStatus::Processing;
    meta
}

// REDIS TYPE SAFETY TESTS
#[tokio::test]
async fn test_job_creation_pipeline_type_safety() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let meta = create_test_meta(parent_job_id, workspace_id);

    let result = queue.create_job_metadata(&meta).await;
    assert!(result.is_ok(), "Job metadata creation failed");

    let retrieved = queue.get_job_metadata(parent_job_id).await;
    assert!(retrieved.is_ok(), "Failed to retrieve metadata");
    assert!(retrieved.unwrap().is_some(), "Metadata not found");
}

#[tokio::test]
async fn test_metadata_update_with_keepttl() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let mut meta = create_test_meta(parent_job_id, workspace_id);

    queue.create_job_metadata(&meta).await.unwrap();
    meta.completed_batches = 2;

    let result = queue.update_job_metadata(&meta).await;
    assert!(result.is_ok(), "Metadata update with KEEPTTL failed");

    let retrieved = queue.get_job_metadata(parent_job_id).await.unwrap().unwrap();
    assert_eq!(retrieved.completed_batches, 2);
}

#[tokio::test]
async fn test_merge_flag_setnx_returns_bool() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();

    let first_result = queue.try_trigger_merge(parent_job_id).await.unwrap();
    assert!(first_result, "First merge should return true");

    let second_result = queue.try_trigger_merge(parent_job_id).await.unwrap();
    assert!(!second_result, "Second merge should return false");
}

#[tokio::test]
async fn test_batch_result_storage_type_safety() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();
    let batch_data = r#"[{"employee_id": "123", "name": "John", "hours": 8.5}]"#;

    let result = queue.store_result(parent_job_id, 0, batch_data).await;
    assert!(result.is_ok(), "Failed to store batch result");

    let results = queue.get_all_results(parent_job_id).await.unwrap();
    assert!(!results.is_empty(), "No results retrieved");
    assert_eq!(results[0].1, batch_data);
}

#[tokio::test]
async fn test_completed_counter_incr_type_safety() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();

    let count1 = queue.increment_completed(parent_job_id).await.unwrap();
    assert_eq!(count1, 1);

    let count2 = queue.increment_completed(parent_job_id).await.unwrap();
    assert_eq!(count2, 2);

    let retrieved = queue.get_completed_count(parent_job_id).await.unwrap();
    assert_eq!(retrieved, 2);
}

#[tokio::test]
async fn test_base64_data_storage_pipeline() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();
    let excel_data = vec![0x50u8; 25000]; // 25KB
    let filename = "attendance_report_2026_01_01.xlsx";

    let result = queue.set_result_data(parent_job_id, &excel_data, filename).await;
    assert!(result.is_ok());

    let retrieved = queue.get_result_data(parent_job_id).await.unwrap();
    assert!(retrieved.is_some());

    let (data, fname) = retrieved.unwrap();
    assert_eq!(data.len(), excel_data.len());
    assert_eq!(fname, filename);
}

#[tokio::test]
async fn test_cleanup_deletion_pipeline() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let meta = create_test_meta(parent_job_id, workspace_id);

    queue.create_job_metadata(&meta).await.unwrap();
    queue.increment_completed(parent_job_id).await.unwrap();
    queue.store_result(parent_job_id, 0, "[]").await.unwrap();
    queue.set_result_data(parent_job_id, b"data", "file.xlsx").await.unwrap();

    let result = queue.cleanup_job(parent_job_id).await;
    assert!(result.is_ok());

    assert!(queue.get_job_metadata(parent_job_id).await.unwrap().is_none());
}

// BLPOP TIMEOUT TESTS
#[tokio::test]
async fn test_blpop_timeout_valid_5_seconds() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let result = queue.dequeue_job(5.0).await;
    assert!(result.is_ok(), "5 second timeout should be valid");
}

#[tokio::test]
async fn test_blpop_timeout_valid_30_seconds() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let result = queue.dequeue_job(30.0).await;
    assert!(result.is_ok(), "30 second timeout should be valid");
}

#[tokio::test]
async fn test_blpop_timeout_negative_invalid() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let result = queue.dequeue_job(-5.0).await;
    assert!(result.is_err(), "Negative timeout should be rejected");
}

#[tokio::test]
async fn test_blpop_timeout_nan_invalid() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let result = queue.dequeue_job(f64::NAN).await;
    assert!(result.is_err(), "NaN timeout should be rejected");
}

#[tokio::test]
async fn test_blpop_timeout_infinity_invalid() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let result = queue.dequeue_job(f64::INFINITY).await;
    assert!(result.is_err(), "Infinity timeout should be rejected");
}

#[tokio::test]
async fn test_blpop_with_enqueued_job() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();

    let job = ReportJob::new(
        parent_job_id,
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        vec![Uuid::new_v4()],
        DateRange {
            start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        },
        apiserver::queue::ReportConfig {
            report_type: ReportType::Summary,
            timezone: "UTC".to_string(),
            rounding_policy: "floor".to_string(),
            work_unit_minutes: 30,
        },
    );

    queue.enqueue_jobs(vec![job.clone()]).await.unwrap();

    let result = queue.dequeue_job(5.0).await.unwrap();
    assert!(result.is_some(), "Should get job immediately");

    let dequeued = result.unwrap();
    assert_eq!(dequeued.job_id, job.job_id);
}

// RACE CONDITION TESTS
#[tokio::test]
async fn test_single_worker_merge_trigger() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();

    let result = queue.try_trigger_merge(parent_job_id).await.unwrap();
    assert!(result, "Single worker should succeed");

    let result2 = queue.try_trigger_merge(parent_job_id).await.unwrap();
    assert!(!result2, "Subsequent trigger should fail");
}

#[tokio::test]
async fn test_concurrent_merge_triggers() {
    let (_test_db, redis_pool) = setup_redis().await;

    let parent_job_id = Uuid::new_v4();
    let mut handles = vec![];

    for _ in 0..4 {
        let redis_pool = redis_pool.clone();
        let job_id = parent_job_id;

        let handle = tokio::spawn(async move {
            let queue = ReportQueue::new(redis_pool);
            queue.try_trigger_merge(job_id).await.unwrap()
        });

        handles.push(handle);
    }

    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    let successes = results.iter().filter(|&&r| r).count();
    assert_eq!(successes, 1, "Exactly one merge should succeed");
}

#[tokio::test]
async fn test_completed_counter_atomic_increments() {
    let (_test_db, redis_pool) = setup_redis().await;

    let parent_job_id = Uuid::new_v4();
    let mut handles = vec![];

    for _ in 0..4 {
        let redis_pool = redis_pool.clone();
        let job_id = parent_job_id;

        let handle = tokio::spawn(async move {
            let queue = ReportQueue::new(redis_pool);
            queue.increment_completed(job_id).await.unwrap()
        });

        handles.push(handle);
    }

    let mut _results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        _results.push(result);
    }

    let final_count = ReportQueue::new(redis_pool.clone())
        .get_completed_count(parent_job_id)
        .await
        .unwrap();

    assert_eq!(final_count, 4);
}

// MERGE COMPLETION TESTS
#[tokio::test]
async fn test_merge_completion_full_flow() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();
    let workspace_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let meta = create_test_meta(parent_job_id, workspace_id);

    queue.create_job_metadata(&meta).await.unwrap();

    // Process 4 batches
    let batch_sizes = vec![200, 200, 200, 104];
    for (idx, size) in batch_sizes.iter().enumerate() {
        let employees = (0..*size)
            .map(|i| format!(r#"{{"employee_id": "emp_{:04}"}}"#, idx * 200 + i))
            .collect::<Vec<_>>()
            .join(",");

        let batch_json = format!("[{}]", employees);
        queue.store_result(parent_job_id, idx as u32, &batch_json).await.unwrap();

        let count = queue.increment_completed(parent_job_id).await.unwrap();
        assert_eq!(count as usize, idx + 1);
    }

    // Trigger merge
    let merge_triggered = queue.try_trigger_merge(parent_job_id).await.unwrap();
    assert!(merge_triggered);

    // Store Excel
    let excel_data = vec![0x50u8; 25000];
    queue.set_result_data(parent_job_id, &excel_data, "report.xlsx").await.unwrap();

    // Update status
    queue.update_job_status(parent_job_id, JobStatus::Completed).await.unwrap();

    let final_meta = queue.get_job_metadata(parent_job_id).await.unwrap().unwrap();
    assert_eq!(final_meta.status, JobStatus::Completed);
    assert!(final_meta.completed_at.is_some());

    // Verify download
    let downloaded = queue.get_result_data(parent_job_id).await.unwrap();
    assert!(downloaded.is_some());
}

#[tokio::test]
async fn test_batch_accumulation_before_merge() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let parent_job_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let meta = create_test_meta(parent_job_id, workspace_id);

    queue.create_job_metadata(&meta).await.unwrap();

    for i in 0..4 {
        let batch_size = if i < 3 { 200 } else { 104 };
        let employees: Vec<String> = (0..batch_size)
            .map(|j| format!(r#"{{"id": "emp_{}" }}"#, i * 200 + j))
            .collect();

        let batch_json = format!("[{}]", employees.join(","));
        queue.store_result(parent_job_id, i as u32, &batch_json).await.unwrap();

        let count = queue.increment_completed(parent_job_id).await.unwrap();
        assert_eq!(count as usize, i + 1);
    }

    let final_count = queue.get_completed_count(parent_job_id).await.unwrap();
    assert_eq!(final_count, 4);

    // Merge should only succeed once
    let first_merge = queue.try_trigger_merge(parent_job_id).await.unwrap();
    let second_merge = queue.try_trigger_merge(parent_job_id).await.unwrap();

    assert!(first_merge);
    assert!(!second_merge);
}

#[tokio::test]
async fn test_stats_accumulation() {
    let (_test_db, redis_pool) = setup_redis().await;
    let queue = ReportQueue::new(redis_pool);

    let count1 = queue.increment_daily_completed().await.unwrap();
    let count2 = queue.increment_daily_completed().await.unwrap();
    let count3 = queue.increment_daily_completed().await.unwrap();

    assert_eq!(count1 + 1, count2);
    assert_eq!(count2 + 1, count3);

    let failed1 = queue.increment_daily_failed().await.unwrap();
    let failed2 = queue.increment_daily_failed().await.unwrap();

    assert_eq!(failed1 + 1, failed2);
}
