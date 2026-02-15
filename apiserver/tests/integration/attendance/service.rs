//! Attendance Log Service integration tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::LogSource;
use apiserver::repositories::raw_attendance_log_repository;
use apiserver::services::attendance_log_service::AttendanceLogService;
use chrono::{TimeZone, Utc};

#[tokio::test]
async fn test_process_log_success_with_timezone() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace_id = factory.workspace().with_timezone("Asia/Ho_Chi_Minh").create().await;
    let _employee_id = factory.employee().with_workspace(workspace_id).with_employee_id_number("EMP_VN_001").create().await;

    // 2. Process a naive timestamp (8 AM VN should be 1 AM UTC)
    let punch_time_raw = "2026-01-23 08:00:00";
    let result = AttendanceLogService::process_log(
        &pool,
        workspace_id,
        "EMP_VN_001",
        punch_time_raw,
        LogSource::Device,
        Some("DEV_01".to_string()),
        None,
        "Asia/Ho_Chi_Minh",
        Some("YYYY-MM-DD"),
        Some("24h")
    ).await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    // Verify in DB
    let logs = raw_attendance_log_repository::get_paginated_raw_logs(&pool, workspace_id, None, None, None, 10, 0).await.unwrap();
    assert_eq!(logs.len(), 1);
    // 08:00:00+07:00 -> 01:00:00 UTC
    assert_eq!(logs[0].punch_time, Utc.with_ymd_and_hms(2026, 1, 23, 1, 0, 0).unwrap());
}
