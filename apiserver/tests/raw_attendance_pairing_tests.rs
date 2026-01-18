mod common;

use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::services::pairing_service::PairingService;
use chrono::{DateTime, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;

async fn create_log(
    pool: &sqlx::PgPool,
    ws_id: Uuid,
    employee_id: Uuid,
    time_str: &str,
) -> RawAttendanceLog {
    let dt = DateTime::parse_from_str(&format!("{} +0000", time_str), "%Y-%m-%d %H:%M:%S %z")
        .unwrap()
        .with_timezone(&Utc);
    let log = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id,
        punch_time: dt,
        source: LogSource::Device,
        device_id: None,
        raw_data: None,
        created_at: Utc::now(),
    };

    apiserver::repositories::raw_attendance_log_repository::create_raw_log(pool, &log)
        .await
        .unwrap();
    log
}

#[tokio::test]
async fn test_pairing_two_logs_same_day() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().with_timezone("UTC").create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. First log (Punch In)
    let log1 = create_log(&pool, ws_id, emp_id, "2026-01-14 08:00:00").await;
    PairingService::process_new_raw_log(&pool, &log1)
        .await
        .unwrap();

    // Verify time_entry created (pending)
    let entries = apiserver::repositories::time_entry_repository::get_paginated_logs(
        &pool,
        ws_id,
        Some(emp_id),
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].punch_out_time.is_none());

    // 2. Second log (Punch Out)
    let log2 = create_log(&pool, ws_id, emp_id, "2026-01-14 17:00:00").await;
    PairingService::process_new_raw_log(&pool, &log2)
        .await
        .unwrap();

    // Verify paired
    let entries = apiserver::repositories::time_entry_repository::get_paginated_logs(
        &pool,
        ws_id,
        Some(emp_id),
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].punch_out_time.is_some());
    assert_eq!(entries[0].duration_minutes, Some(9 * 60));
}

#[tokio::test]
async fn test_pairing_multiple_logs_filtering() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().with_timezone("UTC").create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Sequence: 08:00, 12:00, 13:00, 17:00
    // Result expected: 08:00 -> 12:00 (work), 13:00 -> 17:00 (work)

    let times = [
        "2026-01-14 08:00:00",
        "2026-01-14 12:00:00",
        "2026-01-14 13:00:00",
        "2026-01-14 17:00:00",
    ];

    for t in times {
        let log = create_log(&pool, ws_id, emp_id, t).await;
        PairingService::process_new_raw_log(&pool, &log)
            .await
            .unwrap();
    }

    let entries = apiserver::repositories::time_entry_repository::get_paginated_logs(
        &pool,
        ws_id,
        Some(emp_id),
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    // 2 work sessions
    assert_eq!(entries.len(), 2);
}

#[tokio::test]
async fn test_pairing_overnight_logic_basics() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().with_timezone("UTC").create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Shift: 22:00 -> 06:00
    // Log1: 22:05 (Day 1)
    // Log2: 05:55 (Day 2)

    let log1 = create_log(&pool, ws_id, emp_id, "2026-01-14 22:05:00").await;
    PairingService::process_new_raw_log(&pool, &log1)
        .await
        .unwrap();

    let log2 = create_log(&pool, ws_id, emp_id, "2026-01-15 05:55:00").await;
    PairingService::process_new_raw_log(&pool, &log2)
        .await
        .unwrap();

    let entries = apiserver::repositories::time_entry_repository::get_paginated_logs(
        &pool,
        ws_id,
        Some(emp_id),
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].punch_out_time.is_some());
}

#[tokio::test]
async fn test_pairing_manual_source_overrides() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().with_timezone("UTC").create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let dt1 = DateTime::parse_from_str("2026-01-14 08:00:00 +0000", "%Y-%m-%d %H:%M:%S %z")
        .unwrap()
        .with_timezone(&Utc);
    let log1 = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: dt1,
        source: LogSource::Manual,
        device_id: None,
        raw_data: None,
        created_at: Utc::now(),
    };
    apiserver::repositories::raw_attendance_log_repository::create_raw_log(&pool, &log1)
        .await
        .unwrap();
    PairingService::process_new_raw_log(&pool, &log1)
        .await
        .unwrap();

    let dt2 = DateTime::parse_from_str("2026-01-14 17:00:00 +0000", "%Y-%m-%d %H:%M:%S %z")
        .unwrap()
        .with_timezone(&Utc);
    let log2 = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        punch_time: dt2,
        source: LogSource::Import,
        device_id: None,
        raw_data: None,
        created_at: Utc::now(),
    };
    apiserver::repositories::raw_attendance_log_repository::create_raw_log(&pool, &log2)
        .await
        .unwrap();
    PairingService::process_new_raw_log(&pool, &log2)
        .await
        .unwrap();

    let entries = apiserver::repositories::time_entry_repository::get_paginated_logs(
        &pool,
        ws_id,
        Some(emp_id),
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].punch_out_time.is_some());
}

#[tokio::test]
async fn test_bulk_process_logs() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().with_timezone("UTC").create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let base = DateTime::parse_from_str("2026-01-14 08:00:00 +0000", "%Y-%m-%d %H:%M:%S %z")
        .unwrap()
        .with_timezone(&Utc);
    let log_times = vec![
        base,
        base + chrono::Duration::hours(4),
        base + chrono::Duration::hours(5),
        base + chrono::Duration::hours(9),
    ];

    let mut logs = Vec::new();
    for p in log_times {
        let log = RawAttendanceLog {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            employee_id: emp_id,
            punch_time: p,
            source: LogSource::Device,
            device_id: None,
            raw_data: None,
            created_at: Utc::now(),
        };
        apiserver::repositories::raw_attendance_log_repository::create_raw_log(&pool, &log)
            .await
            .unwrap();
        logs.push(log);
    }

    // Process all
    for log in logs {
        PairingService::process_new_raw_log(&pool, &log)
            .await
            .unwrap();
    }

    let entries = apiserver::repositories::time_entry_repository::get_paginated_logs(
        &pool,
        ws_id,
        Some(emp_id),
        None,
        None,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(entries.len(), 2);
}
