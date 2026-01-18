mod common;

use apiserver::models::{ImportJobStatus};
use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::models::AttendanceImportJob;
use apiserver::services::attendance_import_service::AttendanceImportService;
use apiserver::repositories::import_job_repository;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_import_csv_with_line_errors() {
    let _test_db = TestDatabase::new().await;
    let pool = _test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    
    // Create employee
    let emp_id_number = "ERR-001";
    factory.employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // CSV with mixed valid and invalid lines
    // Line 1: Header
    // Line 2: Valid
    // Line 3: Invalid Date (Completely wrong format)
    // Line 4: Missing Time
    let csv_content = format!("empNo,empName,empDt\n{},User 1,14/01/2026 08:00\n{},User 1,not-a-date\n{},User 1,", 
        emp_id_number, emp_id_number, emp_id_number
    );

    let file_path = format!("storage/test_imports/{}.csv", uuid::Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports").await.unwrap();
    tokio::fs::write(&file_path, csv_content).await.unwrap();

    let job_id = AttendanceImportService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path,
        "log".to_string()
    ).await.unwrap();

    // Wait for completion
    let mut job: Option<AttendanceImportJob> = None;
    for _ in 0..20 {
        sleep(Duration::from_millis(200)).await;
        if let Ok(Some(j)) = import_job_repository::get_job_by_id(&pool, job_id).await {
            if j.status == ImportJobStatus::Completed || j.status == ImportJobStatus::Failed {
                job = Some(j);
                break;
            }
        }
    }

    let job = job.expect("Job timed out");
    let errors = job.error_log.expect("No error log found");
    println!("ACTUAL ERROR LOG:\n{}", errors);
    
    // Line 1: Header
    // Line 2: OK
    // Line 3: Error
    
    assert!(errors.contains("Không thể nhận diện"), "Should identify recognition error");
    assert!(errors.contains("not-a-date"), "Should show faulty data");
    assert!(errors.contains("Thiếu mã nhân viên hoặc thời gian"), "Should describe error for empty time");
}
