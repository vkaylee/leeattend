//! Worker handler tests for Employee Import

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::ImportJobStatus;
use apiserver::queue::{
    EmployeeImportChunkPayload, EmployeeImportPayload, FinalizeImportPayload, Job, JobPayload,
    JobPriority, RedisPool, TaskType,
};
use apiserver::worker::handlers::{
    EmployeeImportChunkHandler, EmployeeImportHandler, FinalizeImportHandler,
};
use uuid::Uuid;

#[tokio::test]
async fn test_employee_import_worker_handlers() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let redis_pool = crate::common::app::create_test_redis_pool().await.unwrap();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Create a dummy XLSX file
    let file_path = format!("/tmp/test_emp_import_{}.xlsx", Uuid::new_v4());
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    let headers = ["employee_code", "full_name", "hire_date", "department_name"];
    for (col, h) in headers.iter().enumerate() {
        sheet.write_string(0, col as u16, *h).unwrap();
    }
    sheet.write_string(1, 0, "EMP_W1").unwrap();
    sheet.write_string(1, 1, "Worker 1").unwrap();
    sheet.write_string(1, 2, "2026-01-01").unwrap();
    sheet.write_string(1, 3, "").unwrap();
    sheet.write_string(2, 0, "EMP_W2").unwrap();
    sheet.write_string(2, 1, "Worker 2").unwrap();
    sheet.write_string(2, 2, "2026-01-15").unwrap();
    sheet.write_string(2, 3, "").unwrap();
    workbook.save(&file_path).unwrap();

    // 2. Create Import Job in DB
    let import_job_id = Uuid::new_v4();
    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, target_type, status, file_path, total_rows, processed_rows)
         VALUES ($1, $2, $3, 'employee', 'pending', $4, 0, 0)",
        diesel::sql_types::Uuid, import_job_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, user_id,
        diesel::sql_types::Text, &file_path
    );

    // 3. Test Master Handler
    let master_job = Job::with_priority(
        Uuid::new_v4(),
        0,
        TaskType::EmployeeImport,
        ws_id,
        JobPayload::EmployeeImport(EmployeeImportPayload {
            import_job_id,
            user_id,
        }),
        JobPriority::Normal,
    );

    let cache: std::sync::Arc<dyn apiserver::repositories::CacheRepository> =
        std::sync::Arc::new(apiserver::cache::RedisCache::new(redis_pool.clone()));
    let queue: std::sync::Arc<dyn apiserver::repositories::QueueRepository> = std::sync::Arc::new(
        apiserver::repositories::queue_repository::RedisQueueRepository::new(redis_pool.clone()),
    );
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let res = EmployeeImportHandler::handle(&master_job, &cache, &queue, &deps).await;
    res.expect("Master handler should succeed");

    // Check DB status is processing or completed
    let status: String = crate::test_query_scalar_string!(
        &pool.api,
        "SELECT status::text AS val FROM attendance_import_jobs WHERE id = $1",
        diesel::sql_types::Uuid,
        import_job_id
    );
    // It actually sets status to Processing, then enqueues chunks.
    assert_eq!(status, "processing");

    // 4. Test Chunk Handler
    let chunk_job = Job::with_priority(
        import_job_id,
        0,
        TaskType::EmployeeImportChunk,
        ws_id,
        JobPayload::EmployeeImportChunk(EmployeeImportChunkPayload {
            import_job_id,
            user_id,
            file_path: file_path.clone(),
            start_index: 1, // Skip header
            end_index: 2,   // Read 2 lines
        }),
        JobPriority::High,
    );

    let res = EmployeeImportChunkHandler::handle(&chunk_job, &cache, &queue, &deps).await;
    res.expect("Chunk handler should succeed");

    // 5. Test Finalize Handler
    let finalize_job = Job::with_priority(
        import_job_id,
        0,
        TaskType::FinalizeImport,
        ws_id,
        JobPayload::FinalizeImport(FinalizeImportPayload { import_job_id }),
        JobPriority::High,
    );

    let res = FinalizeImportHandler::handle(&finalize_job, &cache, &deps).await;
    res.expect("Finalize handler should succeed");

    // Verify status is completed
    let final_status: String = crate::test_query_scalar_string!(
        &pool.api,
        "SELECT status::text AS val FROM attendance_import_jobs WHERE id = $1",
        diesel::sql_types::Uuid,
        import_job_id
    );
    assert_eq!(final_status, "completed");

    // Verify employee was inserted
    let count: i64 = crate::test_query_scalar_i64!(
        &pool.api,
        "SELECT count(*) AS count FROM employees WHERE employee_id_number = '1'"
    );
    assert_eq!(count, 1);

    let _ = std::fs::remove_file(file_path);
}

#[tokio::test]
async fn test_employee_import_handler_empty_file() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let redis_pool = crate::common::app::create_test_redis_pool().await.unwrap();
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let file_path = format!("/tmp/test_emp_import_empty_{}.xlsx", Uuid::new_v4());
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.write_string(0, 0, "employee_code").unwrap();
    sheet.write_string(0, 1, "full_name").unwrap();
    workbook.save(&file_path).unwrap();

    let import_job_id = Uuid::new_v4();
    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, target_type, status, file_path, total_rows, processed_rows)
         VALUES ($1, $2, $3, 'employee', 'pending', $4, 0, 0)",
        diesel::sql_types::Uuid, import_job_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, user_id,
        diesel::sql_types::Text, &file_path
    );

    let master_job = Job::with_priority(
        Uuid::new_v4(),
        0,
        TaskType::EmployeeImport,
        ws_id,
        JobPayload::EmployeeImport(EmployeeImportPayload {
            import_job_id,
            user_id,
        }),
        JobPriority::Normal,
    );
    let cache: std::sync::Arc<dyn apiserver::repositories::CacheRepository> =
        std::sync::Arc::new(apiserver::cache::RedisCache::new(redis_pool.clone()));
    let queue: std::sync::Arc<dyn apiserver::repositories::QueueRepository> = std::sync::Arc::new(
        apiserver::repositories::queue_repository::RedisQueueRepository::new(redis_pool.clone()),
    );
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let res = EmployeeImportHandler::handle(&master_job, &cache, &queue, &deps).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        "File Excel trống hoặc chỉ chứa dòng tiêu đề."
    );

    let _ = std::fs::remove_file(file_path);
}
