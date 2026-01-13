mod common;

use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService;
use std::time::Duration;
use tokio::time::sleep;
use tokio::fs;
use chrono::Datelike;

#[tokio::test]
async fn test_workspace_specific_date_parsing() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let user_id = factory.user().create().await;
    
    // 1. Create workspace with US format (MM/DD/YYYY)
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    sqlx::query("UPDATE workspaces SET date_format = 'MM/DD/YYYY' WHERE id = $1")
        .bind(ws_id)
        .execute(&pool).await.unwrap();

    // Create employee
    let emp_id_number = "DATE-TEST-001";
    let emp_id = factory.employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // 2. Upload file with ambiguous date "01/02/2025" 
    // In US format, this should be Jan 2nd.
    let storage_dir = "./storage/test_imports";
    fs::create_dir_all(storage_dir).await.unwrap();
    let file_path = format!("{}/date_test_{}.csv", storage_dir, uuid::Uuid::new_v4());
    let csv_data = format!("empNo,empName,empDt\n{},Test,01/02/2025 08:30", emp_id_number);
    fs::write(&file_path, csv_data.clone()).await.unwrap();

    let job_id = AttendanceAdjustmentService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path.clone()
    ).await.expect("Import failed");

    // Wait for processing
    let mut finished = false;
    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let job = apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id).await.unwrap().unwrap();
        if job.status == "completed" {
            finished = true;
            break;
        }
    }
    assert!(finished, "Job did not complete");

    // 3. Verify it was parsed as January (Month 1), not February (Month 2)
    let entries = apiserver::repositories::time_entry_repository::get_employee_time_entries(
        &pool, 
        emp_id, 
        chrono::Utc::now() - chrono::Duration::days(400), 
        chrono::Utc::now() + chrono::Duration::days(400)
    ).await.unwrap();
    
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].punch_in_time.month(), 1, "Should be January because workspace is MM/DD/YYYY");
    assert_eq!(entries[0].punch_in_time.day(), 2);

    // 4. Repeat with VN format (DD/MM/YYYY)
    sqlx::query("UPDATE workspaces SET date_format = 'DD/MM/YYYY' WHERE id = $1")
        .bind(ws_id)
        .execute(&pool).await.unwrap();

    let file_path_vn = format!("{}/date_test_vn_{}.csv", storage_dir, uuid::Uuid::new_v4());
    fs::write(&file_path_vn, csv_data).await.unwrap();

    let job_id_vn = AttendanceAdjustmentService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path_vn.clone()
    ).await.expect("Import failed");

    for _ in 0..20 {
        sleep(Duration::from_millis(100)).await;
        let job = apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id_vn).await.unwrap().unwrap();
        if job.status == "completed" { break; }
    }

    // This one should be February 1st
    let entries = apiserver::repositories::time_entry_repository::get_employee_time_entries(
        &pool, 
        emp_id, 
        chrono::Utc::now() - chrono::Duration::days(400), 
        chrono::Utc::now() + chrono::Duration::days(400)
    ).await.unwrap();
    
    // We should have 2 entries now
    assert_eq!(entries.len(), 2);
    
    // Find the new entry (the one with day 1 month 2)
    let has_feb_1st = entries.iter().any(|e| e.punch_in_time.month() == 2 && e.punch_in_time.day() == 1);
    assert!(has_feb_1st, "Should be February 1st because workspace is DD/MM/YYYY");
}
