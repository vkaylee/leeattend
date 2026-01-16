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

#[tokio::test]
async fn test_import_fallback_logic() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().create().await;
    let user_id = factory.user().create().await;
    
    // Create employee
    let emp_id_number = "EMP-FALLBACK-001";
    let emp_id = factory.employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // Create a file that is actually CSV but could be named anything
    // This will fail Xlsx::new and Xls::new, then fall back to CSV
    let storage_dir = "./storage/test_imports";
    fs::create_dir_all(storage_dir).await.unwrap();
    let file_path = format!("{}/test_fallback.xls", storage_dir); // Named .xls but content is CSV
    let csv_data = format!("empNo,empName,empDt\n{},Fallback User,11/01/2026 09:00", emp_id_number);
    fs::write(&file_path, csv_data).await.unwrap();

    let job_id = AttendanceAdjustmentService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path.clone()
    ).await.expect("Import failed");

    let mut processed = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let job = apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id).await.unwrap().unwrap();
        if job.status == "completed" {
            processed = true;
            break;
        }
    }
    
    assert!(processed, "Job did not complete via fallback");

    let entries = apiserver::repositories::time_entry_repository::get_employee_time_entries(
        &pool, 
        emp_id, 
        chrono::Utc::now() - chrono::Duration::days(30), 
        chrono::Utc::now() + chrono::Duration::days(30)
    ).await.unwrap();
    assert_eq!(entries.len(), 1, "Should have created one time entry via CSV fallback");
}

#[tokio::test]
async fn test_import_strict_validation_failure() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().create().await;
    let user_id = factory.user().create().await;

    // Create a temporary CSV file with UNKNOWN employee
    let storage_dir = "./storage/test_imports";
    fs::create_dir_all(storage_dir).await.unwrap();
    let file_path = format!("{}/test_strict_fail.csv", storage_dir);
    let csv_data = "empNo,empName,empDt\nUNKNOWN-001,Unknown,11/01/2026 08:30";
    fs::write(&file_path, csv_data).await.unwrap();

    // Import from file path
    let job_id = AttendanceAdjustmentService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path.clone()
    ).await.expect("Import job creation failed");

    // Wait for async processing
    let mut processed = false;
    let mut error_msg = String::new();
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let job = apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id).await.unwrap().unwrap();
        if job.status == "failed" {
            processed = true;
            error_msg = job.error_log.unwrap_or_default();
            break;
        }
    }
    
    assert!(processed, "Job did not fail as expected");
    assert!(error_msg.contains("Found 1 unknown employees"), "Error message mismatch: {}", error_msg);
    assert!(error_msg.contains("UNKNOWN-001"), "Error message should contain missing ID: {}", error_msg);

    // Verify file is deleted
    assert!(!fs::try_exists(&file_path).await.unwrap(), "File should be deleted after failure");
}