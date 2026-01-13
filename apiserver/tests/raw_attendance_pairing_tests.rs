mod common;

use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;
use chrono::{Utc, Duration, TimeZone};
use apiserver::models::RawAttendanceLog;
use apiserver::services::pairing_service::PairingService;
use apiserver::repositories::{raw_attendance_log_repository, time_entry_repository};

#[tokio::test]
async fn test_pairing_logic_full_cycle() {
    let _test_db = TestDatabase::new().await;
    let pool = _test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let base_time = Utc.with_ymd_and_hms(2026, 1, 14, 8, 0, 0).unwrap();

    // 1. First Punch In
    let log1 = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: base_time,
        source: "device".to_string(),
        device_id: Some("DEV-01".to_string()),
        raw_data: None,
        created_at: Utc::now(),
    };
    raw_attendance_log_repository::create_raw_log(&pool, &log1).await.unwrap();
    PairingService::process_new_raw_log(&pool, &log1).await.unwrap();

    let entries = time_entry_repository::get_employee_time_entries(&pool, emp_id, base_time - Duration::days(1), base_time + Duration::days(1)).await.unwrap();
    assert_eq!(entries.len(), 1, "Should create one time entry");
    assert_eq!(entries[0].punch_in_time, base_time);
    assert!(entries[0].punch_out_time.is_none());

    // 2. Deduplication (Punch again after 1 minute)
    let log2 = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: base_time + Duration::minutes(1),
        source: "device".to_string(),
        device_id: Some("DEV-01".to_string()),
        raw_data: None,
        created_at: Utc::now(),
    };
    raw_attendance_log_repository::create_raw_log(&pool, &log2).await.unwrap();
    PairingService::process_new_raw_log(&pool, &log2).await.unwrap();

    let entries = time_entry_repository::get_employee_time_entries(&pool, emp_id, base_time - Duration::days(1), base_time + Duration::days(1)).await.unwrap();
    assert_eq!(entries.len(), 1, "Should NOT create new entry (deduplicated)");
    assert!(entries[0].punch_out_time.is_none(), "Should NOT update punch out (too close)");

    // 3. Punch Out (After 9 hours)
    let punch_out_time = base_time + Duration::hours(9);
    let log3 = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: punch_out_time,
        source: "device".to_string(),
        device_id: Some("DEV-01".to_string()),
        raw_data: None,
        created_at: Utc::now(),
    };
    raw_attendance_log_repository::create_raw_log(&pool, &log3).await.unwrap();
    PairingService::process_new_raw_log(&pool, &log3).await.unwrap();

    let entries = time_entry_repository::get_employee_time_entries(&pool, emp_id, base_time - Duration::days(1), base_time + Duration::days(1)).await.unwrap();
    assert_eq!(entries.len(), 1, "Should still have one entry");
    assert_eq!(entries[0].punch_out_time, Some(punch_out_time), "Should update punch out");
    assert_eq!(entries[0].duration_minutes, Some(540), "Duration should be 540 mins");

    // 4. New Session (Next day)
    let next_day_time = base_time + Duration::days(1);
    let log4 = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: next_day_time,
        source: "device".to_string(),
        device_id: Some("DEV-01".to_string()),
        raw_data: None,
        created_at: Utc::now(),
    };
    raw_attendance_log_repository::create_raw_log(&pool, &log4).await.unwrap();
    PairingService::process_new_raw_log(&pool, &log4).await.unwrap();

    let entries = time_entry_repository::get_employee_time_entries(&pool, emp_id, base_time - Duration::days(1), next_day_time + Duration::days(1)).await.unwrap();
    assert_eq!(entries.len(), 2, "Should create a second time entry for the next day");
    // entries[0] is latest (next day), entries[1] is previous
    assert_eq!(entries[0].punch_in_time, next_day_time);
    assert!(entries[0].punch_out_time.is_none());
}

#[tokio::test]
async fn test_pairing_window_expiry() {
    let _test_db = TestDatabase::new().await;
    let pool = _test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let base_time = Utc.with_ymd_and_hms(2026, 1, 14, 8, 0, 0).unwrap();

    // 1. Punch In
    let log1 = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: base_time,
        source: "import".to_string(),
        device_id: None,
        raw_data: None,
        created_at: Utc::now(),
    };
    raw_attendance_log_repository::create_raw_log(&pool, &log1).await.unwrap();
    PairingService::process_new_raw_log(&pool, &log1).await.unwrap();

    // 2. Punch after 15 hours (exceeds 14h window)
    let late_time = base_time + Duration::hours(15);
    let log2 = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: late_time,
        source: "import".to_string(),
        device_id: None,
        raw_data: None,
        created_at: Utc::now(),
    };
    raw_attendance_log_repository::create_raw_log(&pool, &log2).await.unwrap();
    PairingService::process_new_raw_log(&pool, &log2).await.unwrap();

    let entries = time_entry_repository::get_employee_time_entries(&pool, emp_id, base_time - Duration::days(1), late_time + Duration::days(1)).await.unwrap();
    assert_eq!(entries.len(), 2, "Should create a NEW session because 14h window expired");
    assert!(entries[1].punch_out_time.is_none(), "Old session should remain open (orphan)");
    assert_eq!(entries[0].punch_in_time, late_time, "New session starts at late_time");
}