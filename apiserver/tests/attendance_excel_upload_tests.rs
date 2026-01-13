mod common;
use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService;
use std::time::Duration;
use tokio::time::sleep;
use tokio::fs;

#[tokio::test]
async fn test_batch_import_from_file_success() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().create().await;
    let user_id = factory.user().create().await;
    
    // Create employee
    let emp_id_number = "EMP-FILE-001";
    let emp_id = factory.employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // Create a temporary CSV file
    let storage_dir = "./storage/test_imports";
    fs::create_dir_all(storage_dir).await.unwrap();
    let file_path = format!("{}/test_{}.csv", storage_dir, uuid::Uuid::new_v4());
    let csv_data = format!("empNo,empName,empDt\n{},John Doe,11/01/2026 08:30", emp_id_number);
    fs::write(&file_path, csv_data).await.unwrap();

    // Import from file path
    let job_id = AttendanceAdjustmentService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path.clone()
    ).await.expect("Import failed");

    // Wait for async processing
    let mut processed = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let job = apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id).await.unwrap().unwrap();
        if job.status == "completed" {
            processed = true;
            break;
        }
    }
    
    assert!(processed, "Job did not complete");

    // Verify Time Entry
    let entries = apiserver::repositories::time_entry_repository::get_employee_time_entries(
        &pool, 
        emp_id, 
        chrono::Utc::now() - chrono::Duration::days(30), 
        chrono::Utc::now() + chrono::Duration::days(30)
    ).await.unwrap();
    assert_eq!(entries.len(), 1);

    // Verify file is deleted
    assert!(!fs::try_exists(&file_path).await.unwrap(), "File should be deleted after completion");
}

#[tokio::test]
async fn test_import_recovery_on_startup() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().create().await;
    let user_id = factory.user().create().await;
    let emp_id_number = "EMP-RECOVERY-001";
    let emp_id = factory.employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // 1. Manually create a file and a 'processing' job record (simulating interrupted server)
    let storage_dir = "./storage/test_imports";
    fs::create_dir_all(storage_dir).await.unwrap();
    let file_path = format!("{}/test_recovery.csv", storage_dir);
    let csv_data = format!("empNo,empName,empDt\n{},Recovery User,11/01/2026 10:00", emp_id_number);
    fs::write(&file_path, csv_data).await.unwrap();

    let job_id = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO attendance_import_jobs (id, workspace_id, user_id, status, total_rows, processed_rows, file_path, created_at, updated_at) VALUES ($1, $2, $3, 'processing', 1, 0, $4, NOW(), NOW())")
        .bind(job_id)
        .bind(ws_id)
        .bind(user_id)
        .bind(&file_path)
        .execute(&pool).await.unwrap();

    // 2. Trigger recovery
    AttendanceAdjustmentService::resume_all_pending_jobs(pool.clone()).await;

    // 3. Wait and verify
    let mut recovered = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let job = apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id).await.unwrap().unwrap();
        if job.status == "completed" {
            recovered = true;
            break;
        }
    }

    assert!(recovered, "Interrupted job was not recovered");
    let entries = apiserver::repositories::time_entry_repository::get_employee_time_entries(
        &pool, 
        emp_id, 
        chrono::Utc::now() - chrono::Duration::days(30), 
        chrono::Utc::now() + chrono::Duration::days(30)
    ).await.unwrap();
    assert_eq!(entries.len(), 1);
}