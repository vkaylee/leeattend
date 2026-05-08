
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;

use apiserver::cache::CacheManager;
use apiserver::queue::{Job, JobMetadata, JobStatus, TaskType, ReportType, JobQueue, DateRange, JobPriority, JobPayload, ReportPayload, ReportConfig, RedisPool};

use apiserver::worker::handlers::AttendanceHandler;

use apiserver::services::ReportCoordinator;
use apiserver::worker::runner::WorkerConfig;

use apiserver::worker::runner::JobWorker;

/// Integration test to verify that the JobWorker ONLY uses ONE database connection per job.
/// By setting max_connections(1), this test will deadlock and timeout if any handler violates the rule.
#[tokio::test]
async fn test_worker_uses_only_one_connection_per_job() {
    let _ = tracing_subscriber::fmt::try_init();

    // 1. Create a DB pool with EXACTLY ONE connection.
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1) // STRICT 1 CONNECTION RULE
        .connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    // 2. Setup mock dependencies
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_pool = RedisPool::new(&redis_url).await.unwrap();
    
    let cache = Arc::new(CacheManager::new(redis_pool.clone(), pool.clone()));
    let queue = JobQueue::new(redis_pool.clone());

    let coordinator = Arc::new(ReportCoordinator::new(redis_pool.clone(), pool.clone()));
    let config = WorkerConfig {
        max_concurrent_jobs: 1,
        poll_timeout_secs: 1.0,
        max_retries: 0,
        job_timeout_secs: 5,
    };
    let worker = JobWorker::new(redis_pool.clone(), pool.clone(), config, coordinator, None);

    let workspace_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // 3. Define a list of test payloads for all task types
    let jobs = vec![
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::AttendanceReport,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![user_id],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "nearest".to_string(),
                    work_unit_minutes: 60,
                },
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::MergeResults,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::MergeResults(apiserver::queue::MergePayload {
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "nearest".to_string(),
                    work_unit_minutes: 60,
                },
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::LeaveAccrual,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::LeaveAccrual(apiserver::queue::AccrualPayload {
                date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                force: false,
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::LogIngestion,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::LogIngestion(apiserver::queue::LogIngestionPayload {
                workspace_id,
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2024-01-01 08:00:00".to_string(),
                device_id: None,
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "%Y-%m-%d".to_string(),
                time_fmt: "%H:%M:%S".to_string(),
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::ReprocessAttendance,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::ReprocessAttendance(apiserver::queue::ReprocessPayload {
                employee_id: user_id,
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
                },
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::ClaimStagedLogs,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::ClaimStagedLogs(apiserver::queue::ClaimStagedLogsPayload {
                workspace_id,
                employee_id: user_id,
                employee_id_number: "EMP001".to_string(),
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::LeaveImport,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::LeaveImport(apiserver::queue::LeaveImportPayload {
                file_path: "test.csv".to_string(),
                user_id,
                auto_approve: false,
                import_job_id: Uuid::new_v4(),
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::NightlyHousekeeping,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::NightlyHousekeeping(apiserver::queue::HousekeepingPayload {
                reference_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::EmployeeImport,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::EmployeeImport(apiserver::queue::EmployeeImportPayload {
                import_job_id: Uuid::new_v4(),
                user_id,
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::EmployeeImportChunk,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::EmployeeImportChunk(apiserver::queue::EmployeeImportChunkPayload {
                import_job_id: Uuid::new_v4(),
                user_id,
                file_path: "test.csv".to_string(),
                start_index: 0,
                end_index: 10,
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::LeaveImportChunk,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::LeaveImportChunk(apiserver::queue::LeaveImportChunkPayload {
                import_job_id: Uuid::new_v4(),
                user_id,
                file_path: "test.csv".to_string(),
                auto_approve: false,
                start_index: 0,
                end_index: 10,
            }),
        },
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            workspace_id,
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            task_type: TaskType::FinalizeImport,
            retry_count: 0,
            max_retries: 3,
            payload: JobPayload::FinalizeImport(apiserver::queue::FinalizeImportPayload {
                import_job_id: Uuid::new_v4(),
            }),
        },
    ];

    for job in jobs {
        let meta = JobMetadata {
            job_id: job.parent_job_id,
            workspace_id,
            total_batches: 1,
            completed_batches: 0,
            failed_batches: 0,
            requester_id: user_id,
            report_type: Some(ReportType::Summary),
            date_range: None,
            status: JobStatus::Pending,
            created_at: chrono::Utc::now(),
            completed_at: None,
            error_message: None,
        };
        
        let _ = queue.create_job_metadata(&meta).await;

        let handler_future = worker.process_job(job.clone());

        // 5 seconds is plenty of time for a single query to run on an empty test DB.
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), handler_future).await;

        match result {
            Ok(_) => {
                // Success! The handler completed without deadlocking.
            }
            Err(_) => {
                panic!("DEADLOCK DETECTED! TaskType {:?} timed out waiting for a database connection.", job.task_type);
            }
        }
    }
}
