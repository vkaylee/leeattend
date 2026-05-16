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
use std::io::Write;
use uuid::Uuid;

#[tokio::test]
async fn test_employee_import_worker_handlers() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_pool = RedisPool::new(&redis_url).await.unwrap();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Create a dummy CSV file
    let file_path = format!("/tmp/test_emp_import_{}.csv", Uuid::new_v4());
    let csv_content = "employee_code,full_name,hire_date,department_name\nEMP_W1,Worker 1,2026-01-01,\nEMP_W2,Worker 2,2026-01-15,";
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(csv_content.as_bytes()).unwrap();

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

    let res = EmployeeImportHandler::handle(&diesel_pools, &master_job, &redis_pool).await;
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

    let res = EmployeeImportChunkHandler::handle(&diesel_pools, &chunk_job, &redis_pool).await;
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

    let res = FinalizeImportHandler::handle(&diesel_pools, &finalize_job, &redis_pool).await;
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
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_pool = RedisPool::new(&redis_url).await.unwrap();
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let file_path = format!("/tmp/test_emp_import_empty_{}.csv", Uuid::new_v4());
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(b"employee_code,full_name\n").unwrap(); // Only header

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
    let res = EmployeeImportHandler::handle(&diesel_pools, &master_job, &redis_pool).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        "Failed to parse file: File is empty or contains only headers."
    );

    let _ = std::fs::remove_file(file_path);
}
