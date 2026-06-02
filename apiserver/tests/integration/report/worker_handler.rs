//! Worker handler tests for Reports (Attendance & Merge)

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::cache::CacheManager;
use apiserver::queue::{
    DateRange, Job, JobMetadata, JobPayload, JobPriority, JobQueue, JobStatus, MergePayload,
    RedisPool, ReportConfig, ReportPayload, ReportType, TaskType,
};
use apiserver::worker::handlers::{AttendanceHandler, MergeHandler};
use chrono::NaiveDate;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_attendance_and_merge_handlers() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let redis_pool = crate::common::app::create_test_redis_pool().await.unwrap();

    let redis_cache: Arc<dyn apiserver::repositories::CacheRepository> =
        Arc::new(apiserver::cache::RedisCache::new(redis_pool.clone()));
    let cache = Arc::new(CacheManager::new(
        redis_cache,
        Arc::clone(&deps.workspace_repo),
        Arc::clone(&deps.shift_repo),
    ));
    let queue = JobQueue::new(redis_pool.clone());
    let queue_repo: Arc<dyn apiserver::repositories::QueueRepository> = Arc::new(
        apiserver::repositories::RedisQueueRepository::new(redis_pool.clone()),
    );
    let publisher_repo: Arc<dyn apiserver::repositories::PublisherRepository> = Arc::new(
        apiserver::repositories::RedisPublisherRepository::new(redis_pool.clone()),
    );

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create 2 employees
    let emp1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EMP_A1")
        .create()
        .await;

    let emp2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EMP_A2")
        .create()
        .await;

    let parent_job_id_summary = Uuid::new_v4();
    let parent_job_id_detail = Uuid::new_v4();

    // Setup job metadata in Redis so AttendanceHandler and MergeHandler find it
    let metadata_summary = JobMetadata::new(
        parent_job_id_summary,
        1, // total_batches
        ws_id,
        user_id,
        Some(TaskType::AttendanceReport),
        Some(ReportType::Summary),
        None,
    );
    queue.update_job_metadata(&metadata_summary).await.unwrap();

    let metadata_detail = JobMetadata::new(
        parent_job_id_detail,
        1, // total_batches
        ws_id,
        user_id,
        Some(TaskType::AttendanceReport),
        Some(ReportType::Detail),
        None,
    );
    queue.update_job_metadata(&metadata_detail).await.unwrap();

    // 1. Test AttendanceHandler (Summary)
    let summary_job = Job::with_priority(
        parent_job_id_summary,
        0,
        TaskType::AttendanceReport,
        ws_id,
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![emp1, emp2],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(), // 3 days
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "Asia/Ho_Chi_Minh".to_string(),
                rounding_policy: "none".to_string(),
                work_unit_minutes: 60,
            },
        }),
        JobPriority::Normal,
    );

    let res = AttendanceHandler::handle(
        &queue_repo,
        &publisher_repo,
        &cache,
        &summary_job,
        None,
        &deps,
    )
    .await;
    res.expect("AttendanceHandler Summary should succeed");

    // 2. Test AttendanceHandler (Detail)
    let detail_job = Job::with_priority(
        parent_job_id_detail,
        0,
        TaskType::AttendanceReport,
        ws_id,
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![emp1],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Detail,
                timezone: "UTC".to_string(),
                rounding_policy: "nearest".to_string(),
                work_unit_minutes: 60,
            },
        }),
        JobPriority::Normal,
    );

    let res = AttendanceHandler::handle(
        &queue_repo,
        &publisher_repo,
        &cache,
        &detail_job,
        None,
        &deps,
    )
    .await;
    res.expect("AttendanceHandler Detail should succeed");

    // 3. Test MergeHandler
    let merge_job = Job::with_priority(
        parent_job_id_summary,
        0,
        TaskType::MergeResults,
        ws_id,
        JobPayload::MergeResults(MergePayload {
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: "none".to_string(),
                work_unit_minutes: 60,
            },
        }),
        JobPriority::High,
    );

    let res = MergeHandler::handle(&queue_repo, &publisher_repo, &merge_job, &deps).await;
    res.expect("MergeHandler should succeed");

    // Test MergeHandler for Detail
    let merge_detail_job = Job::with_priority(
        parent_job_id_detail,
        0,
        TaskType::MergeResults,
        ws_id,
        JobPayload::MergeResults(MergePayload {
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Detail,
                timezone: "UTC".to_string(),
                rounding_policy: "none".to_string(),
                work_unit_minutes: 60,
            },
        }),
        JobPriority::High,
    );

    let res = MergeHandler::handle(&queue_repo, &publisher_repo, &merge_detail_job, &deps).await;
    res.expect("MergeHandler Detail should succeed");

    // Direct tests for RedisPublisherRepository methods to ensure 100% coverage
    publisher_repo
        .publish_chunk(Uuid::new_v4(), 1, 5)
        .await
        .unwrap();
    publisher_repo
        .publish_done(Uuid::new_v4(), 5, 100)
        .await
        .unwrap();
    publisher_repo
        .publish_error(Uuid::new_v4(), "Test error message".to_string(), Some(2))
        .await
        .unwrap();
}
