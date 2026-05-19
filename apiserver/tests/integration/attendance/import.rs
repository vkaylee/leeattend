//! Attendance Import integration tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{enums::ImportTargetType, AttendanceImportJob, ImportJobStatus};
use apiserver::repositories::import_job_repository;
use apiserver::services::attendance_import_service::AttendanceImportService;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_import_csv_with_line_errors() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp_id_number = "ERR-001";
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    let csv_content = format!(
        "employee_code,full_name,punch_time\n{emp_id_number},User 1,14/01/2026 08:00\n{emp_id_number},User 1,not-a-date\n{emp_id_number},User 1,"
    );

    let file_path = format!("storage/test_imports/{}.csv", uuid::Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports")
        .await
        .unwrap();
    tokio::fs::write(&file_path, csv_content).await.unwrap();

    let te_repo = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    let import_repo: std::sync::Arc<
        dyn apiserver::repositories::import_job_repository::ImportJobRepository,
    > = std::sync::Arc::new(apiserver::repositories::DieselImportJobRepository::new(
        diesel_pools.clone(),
    ));
    let adj_repo: std::sync::Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = std::sync::Arc::new(apiserver::repositories::DieselAttendanceAdjustmentRepository::new(diesel_pools.clone()));
    let employee_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
    let job_id = AttendanceImportService::new(
        import_repo.clone(),
        adj_repo,
        employee_repo,
        te_repo,
        std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        )),
        std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        )),
        std::sync::Arc::new(apiserver::repositories::DieselStagingLogRepository::new(
            diesel_pools.clone(),
        )),
        apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone()),
        None, // log_svc
    )
    .batch_import_from_file(ws_id, user_id, file_path, ImportTargetType::Log)
    .await
    .unwrap();

    let mut job: Option<AttendanceImportJob> = None;
    for _ in 0..20 {
        sleep(Duration::from_millis(200)).await;
        if let Ok(Some(j)) = import_repo.get_job_by_id(ws_id, job_id).await {
            if j.status == ImportJobStatus::Completed || j.status == ImportJobStatus::Failed {
                job = Some(j);
                break;
            }
        }
    }

    let job = job.expect("Job timed out");
    let errors = job.error_log.expect("No error log found");

    assert!(
        errors.contains("Missing employee ID or timestamp")
            || errors.contains("Thiếu mã nhân viên hoặc thời gian")
            || errors.contains("Raw:")
            || errors.contains("not-a-date"),
        "Should identify recognition error, got: {errors}"
    );
}

#[tokio::test]
async fn test_import_timeout_guard_updates_db_on_drop() {
    use apiserver::repositories::import_job_repository::ImportJobRepository;
    use apiserver::worker::handlers::utils::ImportTimeoutGuard;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let _ = tracing_subscriber::fmt::try_init();
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let import_repo = Arc::new(apiserver::repositories::DieselImportJobRepository::new(
        diesel_pools.clone(),
    ));

    let job_id = uuid::Uuid::new_v4();
    let job = AttendanceImportJob {
        id: job_id,
        workspace_id: ws_id,
        user_id,
        target_type: ImportTargetType::Employee,
        file_path: Some("test.csv".to_string()),
        total_rows: 100,
        processed_rows: 50,
        status: ImportJobStatus::Processing,
        error_log: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    import_repo.create_job(&job).await.unwrap();

    let completed = Arc::new(AtomicBool::new(false));
    {
        let _guard =
            ImportTimeoutGuard::new(ws_id, job_id, Arc::clone(&completed), import_repo.clone());
        // Guard drops here as it goes out of scope (simulating a tokio timeout cancellation)
    }

    // The drop implementation spawns a tokio task, so we must wait slightly for it to complete
    sleep(Duration::from_millis(2000)).await;

    let updated_job = import_repo
        .get_job_by_id(ws_id, job_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated_job.status, ImportJobStatus::Failed);
    assert_eq!(
        updated_job.processed_rows, 50,
        "Should preserve processed_rows on timeout"
    );
    let error_log = updated_job
        .error_log
        .expect("Error log should be populated");
    assert!(
        error_log.contains("timeout"),
        "Error log should mention timeout"
    );
}

#[tokio::test]
async fn test_employee_import_chunking() {
    use crate::common::app::setup_app_tower;
    use apiserver::queue::{EmployeeImportPayload, Job, JobPayload, JobQueue, TaskType};
    use apiserver::repositories::import_job_repository::ImportJobRepository;
    use apiserver::worker::handlers::EmployeeImportHandler;
    use redis::AsyncCommands;
    let (test_db, factory, app_state, _router) = setup_app_tower().await;

    let pool = test_db.diesel_pools.clone().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    // Check if Redis is enabled
    let redis_pool = match app_state.infra.redis.clone() {
        Some(r) => r,
        None => {
            println!("Skipping test: REDIS_URL not set");
            return;
        }
    };
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let job_id = uuid::Uuid::new_v4();
    let import_repo = apiserver::repositories::DieselImportJobRepository::new(diesel_pools.clone());
    // Create a fake CSV file
    let mut csv_content = String::from("employee_code,full_name,hire_date,department_name\n");
    for i in 1..=1050 {
        csv_content.push_str(&format!("EMP{i},User {i},2023-01-01,IT\n"));
    }

    let file_path = format!("storage/test_imports/{}.csv", job_id);
    tokio::fs::create_dir_all("storage/test_imports")
        .await
        .unwrap();
    tokio::fs::write(&file_path, csv_content).await.unwrap();

    let job = AttendanceImportJob {
        id: job_id,
        workspace_id: ws_id,
        user_id,
        target_type: ImportTargetType::Employee,
        file_path: Some(file_path.clone()),
        total_rows: 1050,
        processed_rows: 0,
        status: ImportJobStatus::Processing,
        error_log: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    import_repo.create_job(&job).await.unwrap();

    let payload = EmployeeImportPayload {
        user_id,
        import_job_id: job_id,
    };

    let worker_job = Job {
        job_id: uuid::Uuid::new_v4(),
        parent_job_id: job_id,
        batch_index: 0,
        task_type: TaskType::EmployeeImport,
        payload: JobPayload::EmployeeImport(payload),
        priority: apiserver::queue::JobPriority::High,
        created_at: chrono::Utc::now(),
        workspace_id: ws_id,
        retry_count: 0,
        max_retries: 3,
    };

    let cache: std::sync::Arc<dyn apiserver::repositories::CacheRepository> =
        std::sync::Arc::new(apiserver::cache::RedisCache::new(redis_pool.clone()));
    let queue: std::sync::Arc<dyn apiserver::repositories::QueueRepository> = std::sync::Arc::new(
        apiserver::repositories::queue_repository::RedisQueueRepository::new(redis_pool.clone()),
    );
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let result = EmployeeImportHandler::handle(&worker_job, &cache, &queue, &deps).await;

    if let Err(e) = &result {
        println!("Handler failed with error: {}", e);
    }
    assert!(
        result.is_ok(),
        "Handler should succeed, got: {:?}",
        result.err()
    );

    // Verify Redis chunking keys
    let mut conn = redis_pool.conn();

    let total_chunks: u32 = redis::cmd("GET")
        .arg(format!("import_total_chunks:{}", job_id))
        .query_async(&mut conn)
        .await
        .unwrap_or(0);

    // 1050 / 500 = 2 remainder 50 -> 3 chunks
    assert_eq!(total_chunks, 3, "Should split 1050 rows into 3 chunks");
    // We do not pop from the queue to assert because `setup_app_tower()`
    // starts a `JobWorker` in the background which will actively consume these jobs,
    // leading to a race condition. Asserting `total_chunks` is sufficient.

    let _ = tokio::fs::remove_file(&file_path).await;
}

#[tokio::test]
async fn test_leave_import_chunking() {
    use crate::common::app::setup_app_tower;
    use apiserver::models::{enums::ImportTargetType, AttendanceImportJob, ImportJobStatus};
    use apiserver::queue::{Job, JobPayload, JobQueue, LeaveImportPayload, TaskType};
    use apiserver::repositories::import_job_repository::ImportJobRepository;
    use apiserver::worker::handlers::LeaveImportHandler;
    use redis::AsyncCommands;

    let (test_db, factory, app_state, _router) = setup_app_tower().await;
    let pool = test_db.diesel_pools.clone().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let redis_pool = match app_state.infra.redis.clone() {
        Some(r) => r,
        None => return,
    };

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let job_id = uuid::Uuid::new_v4();
    let import_repo = apiserver::repositories::DieselImportJobRepository::new(diesel_pools.clone());

    // Create a fake CSV file for leaves
    let mut csv_content =
        String::from("employee_code,leave_type_code,start_date,end_date,reason,duration_type\n");
    for i in 1..=600 {
        csv_content.push_str(&format!(
            "EMP{i},AL,2023-01-01,2023-01-02,Vacation,FullDay\n"
        ));
    }

    let file_path = format!("storage/test_imports/{}.csv", job_id);
    tokio::fs::create_dir_all("storage/test_imports")
        .await
        .unwrap();
    tokio::fs::write(&file_path, csv_content).await.unwrap();

    let job = AttendanceImportJob {
        id: job_id,
        workspace_id: ws_id,
        user_id,
        target_type: ImportTargetType::LeaveImport,
        file_path: Some(file_path.clone()),
        total_rows: 600,
        processed_rows: 0,
        status: ImportJobStatus::Processing,
        error_log: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    import_repo.create_job(&job).await.unwrap();

    let payload = LeaveImportPayload {
        user_id,
        import_job_id: job_id,
        file_path: file_path.clone(),
        auto_approve: true,
    };

    let worker_job = Job {
        job_id: uuid::Uuid::new_v4(),
        parent_job_id: job_id,
        batch_index: 0,
        task_type: TaskType::LeaveImport,
        payload: JobPayload::LeaveImport(payload),
        priority: apiserver::queue::JobPriority::High,
        created_at: chrono::Utc::now(),
        workspace_id: ws_id,
        retry_count: 0,
        max_retries: 3,
    };

    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let cache: std::sync::Arc<dyn apiserver::repositories::CacheRepository> =
        std::sync::Arc::new(apiserver::cache::RedisCache::new(redis_pool.clone()));
    let queue: std::sync::Arc<dyn apiserver::repositories::QueueRepository> = std::sync::Arc::new(
        apiserver::repositories::queue_repository::RedisQueueRepository::new(redis_pool.clone()),
    );
    let result = LeaveImportHandler::handle(&worker_job, &cache, &queue, &deps).await;

    if let Err(e) = &result {
        println!("Handler failed with error: {}", e);
    }
    assert!(
        result.is_ok(),
        "Handler should succeed, got: {:?}",
        result.err()
    );

    let mut conn = redis_pool.conn();
    let total_chunks: u32 = redis::cmd("GET")
        .arg(format!("import_total_chunks:{}", job_id))
        .query_async(&mut conn)
        .await
        .expect("import_total_chunks should be set");

    assert_eq!(total_chunks, 2, "Should split 600 rows into 2 chunks");

    // We do not pop from the queue to assert because `setup_app_tower()`
    // starts a `JobWorker` in the background which will actively consume these jobs,
    // leading to a race condition. Asserting `total_chunks` is sufficient.
}

#[tokio::test]
async fn test_employee_import_duplicate_department_prevention() {
    use apiserver::services::attendance_import_service::{EmployeeImportStrategy, PendingEmployee};
    use apiserver::services::import_messages::AttendanceImportMessages;
    use apiserver::services::import_pipeline::ImportStrategy;
    use std::collections::HashMap;

    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let job_id = uuid::Uuid::new_v4();
    let job = AttendanceImportJob {
        id: job_id,
        workspace_id: ws_id,
        user_id,
        target_type: ImportTargetType::Employee,
        file_path: None,
        total_rows: 2,
        processed_rows: 0,
        status: ImportJobStatus::Processing,
        error_log: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let employee_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));

    let msgs = AttendanceImportMessages::for_lang("vi");

    let department_repo: std::sync::Arc<
        dyn apiserver::repositories::department_repository::DepartmentRepository,
    > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
        diesel_pools.clone(),
    ));

    // Strategy 1 (simulating chunk 1)
    let mut strategy1 = EmployeeImportStrategy {
        msgs: msgs.clone(),
        employee_repo: std::sync::Arc::clone(&employee_repo),
        department_repo: std::sync::Arc::clone(&department_repo),
        staging_repo: std::sync::Arc::new(
            apiserver::repositories::DieselStagingLogRepository::new(diesel_pools.clone()),
        ),
        dept_cache: HashMap::new(), // Empty cache
    };

    // Strategy 2 (simulating chunk 2 running independently)
    let mut strategy2 = EmployeeImportStrategy {
        msgs,
        employee_repo: std::sync::Arc::clone(&employee_repo),
        department_repo: std::sync::Arc::clone(&department_repo),
        staging_repo: std::sync::Arc::new(
            apiserver::repositories::DieselStagingLogRepository::new(diesel_pools.clone()),
        ),
        dept_cache: HashMap::new(), // Empty cache
    };

    let dept_name = "Engineering".to_string();

    let emp1 = PendingEmployee {
        id_number: "E001".to_string(),
        full_name: "Alice".to_string(),
        hire_date: chrono::Utc::now().naive_utc().date(),
        department_name: Some(dept_name.clone()),
        line_number: 1,
    };

    let emp2 = PendingEmployee {
        id_number: "E002".to_string(),
        full_name: "Bob".to_string(),
        hire_date: chrono::Utc::now().naive_utc().date(),
        department_name: Some(dept_name.clone()),
        line_number: 2,
    };

    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    // Chunk 1 processes emp1. It should create the department.
    let res1 = strategy1.process_row(&deps, &job, &emp1).await;
    assert!(res1.is_ok(), "First process_row failed");

    // Chunk 2 processes emp2. Cache is empty, so it will miss cache.
    // Thanks to the DB double-check, it should NOT create a duplicate department.
    let res2 = strategy2.process_row(&deps, &job, &emp2).await;
    assert!(res2.is_ok(), "Second process_row failed");

    // Verify exactly one department exists in the database
    let departments = department_repo.get_departments(ws_id).await.unwrap();

    assert_eq!(
        departments.len(),
        1,
        "Should only create 1 department, found {}",
        departments.len()
    );
    assert_eq!(departments[0].name, dept_name);
}
