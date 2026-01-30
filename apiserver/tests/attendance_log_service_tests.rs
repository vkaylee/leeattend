use chrono::{TimeZone, Utc};
use apiserver::models::LogSource;
use apiserver::repositories::raw_attendance_log_repository;
use apiserver::services::attendance_log_service::AttendanceLogService;

mod common;

use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_process_log_success_with_timezone() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // 1. Setup: Create Workspace (VN Timezone) and Employee
    let workspace_id = factory.workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create()
        .await;
    
    let _employee_id = factory.employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP_VN_001")
        .create()
        .await;

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

    // 3. Assertions
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Verify in DB
    let logs = raw_attendance_log_repository::get_paginated_raw_logs(
        &pool, workspace_id, None, None, None, 10, 0
    ).await.unwrap();

    assert_eq!(logs.len(), 1);
    // 08:00:00+07:00 -> 01:00:00 UTC
    assert_eq!(logs[0].punch_time, Utc.with_ymd_and_hms(2026, 1, 23, 1, 0, 0).unwrap());
}

#[tokio::test]
async fn test_process_log_duplicate_prevention() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace_id = factory.workspace().create().await;
    let _employee_id = factory.employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP_DUP")
        .create()
        .await;

    let punch_time = "2026-01-23 10:00:00";

    // First call: Success
    let res1 = AttendanceLogService::process_log(
        &pool, workspace_id, "EMP_DUP", punch_time, LogSource::Device, None, None, "UTC", None, None
    ).await;
    assert!(res1.unwrap());

    // Second call: Duplicate (Should return Ok(false) due to ON CONFLICT DO NOTHING)
    let res2 = AttendanceLogService::process_log(
        &pool, workspace_id, "EMP_DUP", punch_time, LogSource::Device, None, None, "UTC", None, None
    ).await;
    assert!(!res2.unwrap());

    // Verify only one log exists
    let count = raw_attendance_log_repository::count_raw_logs(&pool, workspace_id, None, None, None).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_process_log_unknown_employee() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace_id = factory.workspace().create().await;

    let result = AttendanceLogService::process_log(
        &pool,
        workspace_id,
        "NON_EXISTENT",
        "2026-01-23 10:00:00",
        LogSource::Device,
        None,
        None,
        "UTC",
        None,
        None
    ).await;

    // Dual-Layer Protection: unknown employee is staged, not an error
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Ok(false) = staged for later

    // Verify staged in staging table
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attendance_staging_logs WHERE workspace_id = $1 AND raw_user_id = $2")
        .bind(workspace_id)
        .bind("NON_EXISTENT")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "Should have staged the unknown employee log");
}

#[tokio::test]
async fn test_process_log_invalid_timestamp() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace_id = factory.workspace().create().await;
    let _employee_id = factory.employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP_INV")
        .create()
        .await;

    let result = AttendanceLogService::process_log(
        &pool,
        workspace_id,
        "EMP_INV",
        "invalid-date-format",
        LogSource::Device,
        None,
        None,
        "UTC",
        None,
        None
    ).await;

    // Dual-Layer Protection: invalid timestamp is logged to DLQ, not an error
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Ok(false) = logged to error table

    // Verify in DLQ
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attendance_log_errors WHERE workspace_id = $1")
        .bind(workspace_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "Should have error record in DLQ");
}

#[tokio::test]
async fn test_process_log_records_to_dlq_on_error() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace_id = factory.workspace().create().await;

    // 1. Unknown Employee → staged (not DLQ)
    let _ = AttendanceLogService::process_log(
        &pool, workspace_id, "UNKNOWN_EMP", "2026-01-23 10:00:00",
        LogSource::Device, Some("DEV_1".to_string()), None, "UTC", None, None
    ).await;

    // 2. Invalid Timestamp → DLQ
    let _employee_id = factory.employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP_VALID")
        .create()
        .await;
    let _ = AttendanceLogService::process_log(
        &pool, workspace_id, "EMP_VALID", "invalid-time",
        LogSource::Device, Some("DEV_1".to_string()), None, "UTC", None, None
    ).await;

    // 3. Verify DLQ Table (only invalid timestamp goes here)
    let dlq_errors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attendance_log_errors WHERE workspace_id = $1")
        .bind(workspace_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(dlq_errors, 1, "Should have 1 error in DLQ (invalid timestamp only)");

    // 4. Verify Staging Table (unknown employee goes here)
    let staged: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attendance_staging_logs WHERE workspace_id = $1")
        .bind(workspace_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(staged, 1, "Should have 1 staged log (unknown employee)");

    // Verify DLQ error message
    let error_msg: String = sqlx::query_scalar("SELECT error_message FROM attendance_log_errors WHERE workspace_id = $1")
        .bind(workspace_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(error_msg.contains("Failed to parse timestamp"));
}
