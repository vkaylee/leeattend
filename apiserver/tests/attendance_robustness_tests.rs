mod common;
use apiserver::models::ImportJobStatus;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

#[tokio::test]
async fn test_import_robustness_duplicates_and_missing_employees() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().create().await;
    let user_id = factory.user().create().await;

    // 1. Create a valid employee
    let emp_id_number = "VALID-001";
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // 2. Create a temporary CSV file with:
    // - One valid entry
    // - One duplicate of the valid entry
    // - One entry for a non-existent employee
    let storage_dir = "./storage/test_imports";
    fs::create_dir_all(storage_dir).await.unwrap();
    let file_path = format!("{}/test_robustness.csv", storage_dir);
    let csv_data = format!(
        "empNo,empName,empDt\n\
        {0},John Doe,11/01/2026 08:30\n\
        {0},John Doe,11/01/2026 08:30\n\
        MISSING-999,Unknown,11/01/2026 09:00",
        emp_id_number
    );
    fs::write(&file_path, csv_data).await.unwrap();

    // 3. Import as 'log' target (this uses process_import_row which calls create_raw_log)
    let job_id = apiserver::services::attendance_import_service::AttendanceImportService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path.clone(),
        "log".to_string(),
    )
    .await
    .expect("Import job creation failed");

    // 4. Wait for completion (it should complete now, not fail)
    let mut job = None;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let j = apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id)
            .await
            .unwrap()
            .unwrap();
        if j.status == ImportJobStatus::Completed || j.status == ImportJobStatus::Failed {
            job = Some(j);
            break;
        }
    }

    let job = job.expect("Job timed out");
    assert_eq!(job.status, ImportJobStatus::Completed, "Job should complete even with missing employees");
    
    let error_log = job.error_log.unwrap_or_default();
    println!("Error Log: {}", error_log);
    
    // Verify error reporting
    assert!(error_log.contains("Bỏ qua 1 mã nhân viên không tồn tại"), "Should report missing employee count");
    assert!(error_log.contains("MISSING-999"), "Should report specific missing ID");
    assert!(error_log.contains("Đã bỏ qua 2 dòng"), "Should report skipped count (1 missing + 1 duplicate)");

    // 5. Verify Database Content
    // Should have exactly 1 log for the valid employee
    let logs = apiserver::repositories::raw_attendance_log_repository::get_paginated_raw_logs(
        &pool,
        ws_id,
        Some(emp_id),
        None,
        None,
        10,
        0
    ).await.unwrap();
    
    assert_eq!(logs.len(), 1, "Should have only 1 unique log inserted");
    assert_eq!(logs[0].punch_time.to_rfc3339(), "2026-01-11T01:30:00+00:00"); // 08:30 VNT is 01:30 UTC

    // Verify file is deleted
    assert!(!fs::try_exists(&file_path).await.unwrap(), "File should be deleted");
}
