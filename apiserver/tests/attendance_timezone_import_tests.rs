mod common;

use apiserver::models::ImportJobStatus;
use apiserver::repositories::{import_job_repository, raw_attendance_log_repository};
use apiserver::services::attendance_import_service::AttendanceImportService;
use chrono::{Datelike, Timelike};
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_import_csv_with_workspace_timezone() {
    let _test_db = TestDatabase::new().await;
    let pool = _test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;

    // 1. Setup Workspace in Asia/Ho_Chi_Minh (UTC+7)
    let ws_id = factory
        .workspace()
        .with_name("Hanoi Office")
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;

    let emp_id_number = "TZ-001";
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // CSV with Naive time (no offset)
    // 08:00:00 in ICT (UTC+7) should be 01:00:00 UTC
    let csv_content = format!(
        "empNo,empName,empDt\n{},User 1,14/01/2026 08:00:00",
        emp_id_number
    );

    let file_path = format!("storage/test_imports/{}.csv", uuid::Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports")
        .await
        .unwrap();
    tokio::fs::write(&file_path, csv_content).await.unwrap();

    let job_id = AttendanceImportService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path,
        "log".to_string(),
    )
    .await
    .unwrap();

    // Wait for completion
    for _ in 0..20 {
        sleep(Duration::from_millis(200)).await;
        if let Ok(Some(j)) = import_job_repository::get_job_by_id(&pool, job_id).await {
            if j.status == ImportJobStatus::Completed || j.status == ImportJobStatus::Failed {
                break;
            }
        }
    }

    // Verify raw logs
    let logs = raw_attendance_log_repository::get_paginated_raw_logs(
        &pool,
        ws_id,
        Some(employee_id),
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(logs.len(), 1);

    let punch_utc = logs[0].punch_time;
    // 08:00:00 Local (UTC+7) -> 01:00:00 UTC
    assert_eq!(punch_utc.hour(), 1, "Should be 01:00 UTC for 08:00 ICT");
    assert_eq!(punch_utc.day(), 14);
}

#[tokio::test]
async fn test_import_csv_with_utc_workspace() {
    let _test_db = TestDatabase::new().await;
    let pool = _test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;

    // 2. Setup Workspace in UTC
    let ws_id = factory
        .workspace()
        .with_name("Global Office")
        .with_timezone("UTC")
        .create_with_owner(user_id)
        .await;

    let emp_id_number = "TZ-002";
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // CSV with Naive time (no offset)
    // 08:00:00 in UTC should be 08:00:00 UTC
    let csv_content = format!(
        "empNo,empName,empDt\n{},User 2,14/01/2026 08:00:00",
        emp_id_number
    );

    let file_path = format!("storage/test_imports/{}.csv", uuid::Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports")
        .await
        .unwrap();
    tokio::fs::write(&file_path, csv_content).await.unwrap();

    let job_id = AttendanceImportService::batch_import_from_file(
        pool.clone(),
        ws_id,
        user_id,
        file_path,
        "log".to_string(),
    )
    .await
    .unwrap();

    // Wait
    for _ in 0..20 {
        sleep(Duration::from_millis(200)).await;
        if let Ok(Some(j)) = import_job_repository::get_job_by_id(&pool, job_id).await {
            if j.status == ImportJobStatus::Completed || j.status == ImportJobStatus::Failed {
                break;
            }
        }
    }

    let logs = raw_attendance_log_repository::get_paginated_raw_logs(
        &pool,
        ws_id,
        Some(employee_id),
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    let punch_utc = logs[0].punch_time;
    assert_eq!(
        punch_utc.hour(),
        8,
        "Should be 08:00 UTC for 08:00 UTC workspace"
    );
}
