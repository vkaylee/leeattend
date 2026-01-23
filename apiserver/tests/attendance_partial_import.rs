use chrono::{TimeZone, Utc};
use std::io::Write;
use tempfile::NamedTempFile;
use uuid::Uuid;

mod common;

use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::models::{ImportJobStatus, LogSource};
use apiserver::repositories::{import_job_repository, raw_attendance_log_repository};
use apiserver::services::attendance_import_service::AttendanceImportService;

#[tokio::test]
async fn test_attendance_import_partial_success() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // 1. Setup: Create Workspace and one valid Employee
    let workspace_id = factory.workspace().create().await;
    let employee_id = factory.employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP001")
        .with_full_name("John Doe")
        .create()
        .await;

    // 2. Prepare CSV data:
    // Row 1: Valid (EMP001)
    // Row 2: Invalid Employee (EMP999)
    // Row 3: Duplicate of Row 1 (Same employee, same time)
    let punch_time = Utc.with_ymd_and_hms(2026, 1, 22, 8, 0, 0).unwrap();
    let punch_time_str = punch_time.to_rfc3339();
    
    let csv_content = format!(
        "Employee ID,Name,Time\n\
        EMP001,John Doe,{time}\n\
        EMP999,Ghost,{time}\n\
        EMP001,John Doe,{time}",
        time = punch_time_str
    );

    let mut tmp_file = NamedTempFile::new().expect("Failed to create temp file");
    write!(tmp_file, "{}", csv_content).expect("Failed to write to temp file");
    let file_path = tmp_file.path().to_str().unwrap().to_string();

    // 3. Create Import Job
    let user_id = Uuid::new_v4(); // Mock user
    let job_id = AttendanceImportService::batch_import_from_file(
        pool.clone(),
        workspace_id,
        user_id,
        file_path,
        "log".to_string(),
    )
    .await
    .expect("Failed to initiate import job");

    // 4. Wait for processing to finish (it's spawned in a task)
    let mut job = None;
    for _ in 0..15 {
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        job = import_job_repository::get_job_by_id(&pool, job_id).await.unwrap();
        if let Some(ref j) = job {
            if j.status == ImportJobStatus::Completed || j.status == ImportJobStatus::Failed {
                break;
            }
        }
    }

    let job = job.expect("Job not found");
    
    // 5. Verifications
    assert_eq!(job.status, ImportJobStatus::Completed);
    assert_eq!(job.processed_rows, 1);
    
    // - Check Raw Logs
    let logs = raw_attendance_log_repository::get_paginated_raw_logs(
        &pool, workspace_id, None, None, None, 10, 0
    )
    .await.unwrap();
    
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].employee_id, employee_id);
    assert_eq!(logs[0].punch_time, punch_time);
    assert_eq!(logs[0].source, LogSource::Import);

    // - Check Error Log
    let error_log = job.error_log.expect("Missing error log for partial success");
    assert!(error_log.contains("EMP999"), "Should mention missing employee EMP999");
    assert!(error_log.contains("bỏ qua") || error_log.contains("skipped"), "Should mention skipping rows");
}
