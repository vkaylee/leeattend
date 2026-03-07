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

    let workspace_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create()
        .await;
    let _employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP_VN_001")
        .create()
        .await;

    // 2. Process a naive timestamp (8 AM VN should be 1 AM UTC)
    let punch_time_raw = "2026-01-23 08:00:00";
    let repo = std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone()));
    let pairing = std::sync::Arc::new(apiserver::services::pairing_service::PairingService::new(repo, pool.clone()));
    let (service, _) = AttendanceLogService::new(pool.clone(), pairing, 100, 10, 1000);
    let result = service.process_log(
        workspace_id,
        "EMP_VN_001",
        punch_time_raw,
        LogSource::Device,
        Some("DEV_01".to_string()),
        None,
        "Asia/Ho_Chi_Minh",
        Some("YYYY-MM-DD"),
        Some("24h"),
    )
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    // Verify in DB
    let logs = raw_attendance_log_repository::get_paginated_raw_logs(
        &pool,
        workspace_id,
        None,
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(logs.len(), 1);
    // 08:00:00+07:00 -> 01:00:00 UTC
    assert_eq!(
        logs[0].punch_time,
        Utc.with_ymd_and_hms(2026, 1, 23, 1, 0, 0).unwrap()
    );
}

#[tokio::test]
async fn test_same_second_punch_with_different_status_allowed() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace_id = factory.workspace().create().await;
    let _employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP001")
        .create()
        .await;

    let punch_time_raw = "2026-01-23 08:00:00";

    let repo = std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone()));
    let pairing = std::sync::Arc::new(apiserver::services::pairing_service::PairingService::new(repo, pool.clone()));
    let (service, _) = AttendanceLogService::new(pool.clone(), pairing, 100, 10, 1000);

    // 1. First punch (check-in)
    let res1 = service.process_log(
        workspace_id,
        "EMP001",
        punch_time_raw,
        LogSource::Device,
        Some("DEV_01".to_string()),
        Some(serde_json::json!({"punch": 0, "status": 1})),
        "UTC",
        Some("YYYY-MM-DD"),
        Some("24h"),
    )
    .await;
    assert!(res1.unwrap(), "First log should be saved");

    // 2. Second punch at EXACTLY same time but different status (check-out)
    let res2 = service.process_log(
        workspace_id,
        "EMP001",
        punch_time_raw,
        LogSource::Device,
        Some("DEV_01".to_string()),
        Some(serde_json::json!({"punch": 1, "status": 1})),
        "UTC",
        Some("YYYY-MM-DD"),
        Some("24h"),
    )
    .await;
    assert!(
        res2.unwrap(),
        "Second log at same second with different punch should be saved"
    );

    // 3. Exact duplicate (same time, punch, status) - should be skipped
    let res3 = service.process_log(
        workspace_id,
        "EMP001",
        punch_time_raw,
        LogSource::Device,
        Some("DEV_01".to_string()),
        Some(serde_json::json!({"punch": 0, "status": 1})),
        "UTC",
        Some("YYYY-MM-DD"),
        Some("24h"),
    )
    .await;
    assert!(
        !res3.unwrap(),
        "Exact duplicate should be skipped (Ok(false))"
    );

    // 4. Verify both unique logs are in DB
    let logs = raw_attendance_log_repository::get_paginated_raw_logs(
        &pool,
        workspace_id,
        None,
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(
        logs.len(),
        2,
        "DB should contain exactly 2 unique logs for the same second"
    );
}
