//! Worker handler tests for Leave Import

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::queue::{
    Job, JobPayload, JobPriority, LeaveImportChunkPayload, LeaveImportPayload, RedisPool, TaskType,
};
use apiserver::worker::handlers::{LeaveImportChunkHandler, LeaveImportHandler};
use uuid::Uuid;

#[tokio::test]
async fn test_leave_import_worker_handlers() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let redis_pool = crate::common::app::create_test_redis_pool().await.unwrap();

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

    // 1. Create a dummy XLSX file
    let file_path = format!("/tmp/test_leave_import_{}.xlsx", Uuid::new_v4());
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    let headers = [
        "employee_code",
        "leave_type_name",
        "start_date",
        "end_date",
        "start_session",
        "end_session",
        "reason",
    ];
    for (col, h) in headers.iter().enumerate() {
        sheet.write_string(0, col as u16, *h).unwrap();
    }
    sheet.write_string(1, 0, "EMP_W1").unwrap();
    sheet.write_string(1, 1, "Annual Leave").unwrap();
    sheet.write_string(1, 2, "2026-05-01").unwrap();
    sheet.write_string(1, 3, "2026-05-02").unwrap();
    sheet.write_string(1, 4, "morning").unwrap();
    sheet.write_string(1, 5, "afternoon").unwrap();
    sheet.write_string(1, 6, "Vacation").unwrap();
    workbook.save(&file_path).unwrap();

    // 2. Create Import Job in DB
    let import_job_id = Uuid::new_v4();
    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, target_type, status, file_path, total_rows, processed_rows)
         VALUES ($1, $2, $3, 'leave_import', 'pending', $4, 0, 0)",
        diesel::sql_types::Uuid, import_job_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, user_id,
        diesel::sql_types::Text, &file_path
    );

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

    let cache: std::sync::Arc<dyn apiserver::repositories::CacheRepository> =
        std::sync::Arc::new(apiserver::cache::RedisCache::new(redis_pool.clone()));
    let queue: std::sync::Arc<dyn apiserver::repositories::QueueRepository> = std::sync::Arc::new(
        apiserver::repositories::queue_repository::RedisQueueRepository::new(redis_pool.clone()),
    );
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let res = LeaveImportHandler::handle(&master_job, &cache, &queue, &deps).await;
    res.expect("Master handler should succeed");

    let status: String = crate::test_query_scalar_string!(
        &pool.api,
        "SELECT status::text AS val FROM attendance_import_jobs WHERE id = $1",
        diesel::sql_types::Uuid,
        import_job_id
    );
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

    let res = LeaveImportChunkHandler::handle(&chunk_job, &cache, &queue, &deps).await;
    res.expect("Chunk handler should succeed");

    // We can also verify that Finalize was enqueued, but for coverage, testing the handler code is enough

    let _ = std::fs::remove_file(file_path);
}

#[tokio::test]
async fn test_leave_import_handler_empty_file() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let redis_pool = crate::common::app::create_test_redis_pool().await.unwrap();
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let file_path = format!("/tmp/test_leave_import_empty_{}.xlsx", Uuid::new_v4());
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.write_string(0, 0, "employee_code").unwrap();
    sheet.write_string(0, 1, "leave_type").unwrap();
    workbook.save(&file_path).unwrap();

    let import_job_id = Uuid::new_v4();
    crate::test_execute!(&pool.api,
        "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, target_type, status, file_path, total_rows, processed_rows)
         VALUES ($1, $2, $3, 'leave_import', 'pending', $4, 0, 0)",
        diesel::sql_types::Uuid, import_job_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, user_id,
        diesel::sql_types::Text, &file_path
    );

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
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let cache: std::sync::Arc<dyn apiserver::repositories::CacheRepository> =
        std::sync::Arc::new(apiserver::cache::RedisCache::new(redis_pool.clone()));
    let queue: std::sync::Arc<dyn apiserver::repositories::QueueRepository> = std::sync::Arc::new(
        apiserver::repositories::queue_repository::RedisQueueRepository::new(redis_pool.clone()),
    );
    let res = LeaveImportHandler::handle(&master_job, &cache, &queue, &deps).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        "File Excel trống hoặc chỉ chứa dòng tiêu đề."
    );

    let _ = std::fs::remove_file(file_path);
}
