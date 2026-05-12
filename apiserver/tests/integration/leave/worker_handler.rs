//! Worker handler tests for Leave Import

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::queue::{
    Job, JobPayload, JobPriority, LeaveImportChunkPayload, LeaveImportPayload, RedisPool, TaskType,
};
use apiserver::worker::handlers::{LeaveImportChunkHandler, LeaveImportHandler};
use std::io::Write;
use uuid::Uuid;

#[tokio::test]
async fn test_leave_import_worker_handlers() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(
        &std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/leeattend".to_string()
        }),
    );
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_pool = RedisPool::new(&redis_url).await.unwrap();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create an employee first so leave can map
    let _emp = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EMP_W1")
        .create()
        .await;

    let lt_id = factory
        .leave_policy()
        .with_workspace(ws_id)
        .with_name("Annual Leave")
        .create()
        .await;

    // 1. Create a dummy CSV file
    let file_path = format!("/tmp/test_leave_import_{}.csv", Uuid::new_v4());
    let csv_content = "employee_code,leave_type_name,start_date,end_date,start_session,end_session,reason\nEMP_W1,Annual Leave,2026-05-01,2026-05-02,morning,afternoon,Vacation";
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(csv_content.as_bytes()).unwrap();

    // 2. Create Import Job in DB
    let import_job_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, target_type, status, file_path, total_rows, processed_rows)
         VALUES ($1, $2, $3, 'leave_import', 'pending', $4, 0, 0)"
    )
    .bind(import_job_id)
    .bind(ws_id)
    .bind(user_id)
    .bind(&file_path)
    .execute(&pool)
    .await
    .unwrap();

    // 3. Test Master Handler
    let master_job = Job::with_priority(
        Uuid::new_v4(),
        0,
        TaskType::LeaveImport,
        ws_id,
        JobPayload::LeaveImport(LeaveImportPayload {
            import_job_id,
            user_id,
            file_path: file_path.clone(),
            auto_approve: true,
        }),
        JobPriority::Normal,
    );

    let res = LeaveImportHandler::handle(&pool, &diesel_pools, &master_job, &redis_pool).await;
    res.expect("Master handler should succeed");

    let status: String =
        sqlx::query_scalar("SELECT status::text FROM attendance_import_jobs WHERE id = $1")
            .bind(import_job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "processing");

    // 4. Test Chunk Handler
    let chunk_job = Job::with_priority(
        import_job_id,
        0,
        TaskType::LeaveImportChunk,
        ws_id,
        JobPayload::LeaveImportChunk(LeaveImportChunkPayload {
            import_job_id,
            user_id,
            file_path: file_path.clone(),
            start_index: 1, // Skip header
            end_index: 1,   // Read 1 line
            auto_approve: true,
        }),
        JobPriority::High,
    );

    let res = LeaveImportChunkHandler::handle(&pool, &diesel_pools, &chunk_job, &redis_pool).await;
    res.expect("Chunk handler should succeed");

    // We can also verify that Finalize was enqueued, but for coverage, testing the handler code is enough

    let _ = std::fs::remove_file(file_path);
}

#[tokio::test]
async fn test_leave_import_handler_empty_file() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(
        &std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/leeattend".to_string()
        }),
    );
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_pool = RedisPool::new(&redis_url).await.unwrap();
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let file_path = format!("/tmp/test_leave_import_empty_{}.csv", Uuid::new_v4());
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(b"employee_code,leave_type\n").unwrap(); // Only header

    let import_job_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, target_type, status, file_path, total_rows, processed_rows)
         VALUES ($1, $2, $3, 'leave_import', 'pending', $4, 0, 0)"
    )
    .bind(import_job_id).bind(ws_id).bind(user_id).bind(&file_path).execute(&pool).await.unwrap();

    let master_job = Job::with_priority(
        Uuid::new_v4(),
        0,
        TaskType::LeaveImport,
        ws_id,
        JobPayload::LeaveImport(LeaveImportPayload {
            import_job_id,
            user_id,
            file_path: file_path.clone(),
            auto_approve: false,
        }),
        JobPriority::Normal,
    );
    let res = LeaveImportHandler::handle(&pool, &diesel_pools, &master_job, &redis_pool).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        "Failed to parse file: File is empty or contains only headers."
    );

    let _ = std::fs::remove_file(file_path);
}
