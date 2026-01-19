mod common;

use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::services::pairing_service::PairingService;
use chrono::{DateTime, Utc, TimeZone, NaiveDateTime, Timelike};
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;
use chrono_tz::Asia::Ho_Chi_Minh;

async fn create_log(
    pool: &sqlx::PgPool,
    ws_id: Uuid,
    employee_id: Uuid,
    time_str: &str,
) -> Uuid {
    let dt = DateTime::parse_from_str(&format!("{} +0000", time_str), "%Y-%m-%d %H:%M:%S %z")
        .unwrap()
        .with_timezone(&Utc);
    let id = Uuid::new_v4();
    let log = RawAttendanceLog {
        id,
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
    id
}

async fn create_log_vn(
    pool: &sqlx::PgPool,
    ws_id: Uuid,
    employee_id: Uuid,
    time_str: &str,
) -> Uuid {
    // time_str is in Ho_Chi_Minh time
    let naive = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S").unwrap();
    let local_dt = Ho_Chi_Minh.from_local_datetime(&naive).unwrap();
    let dt = local_dt.with_timezone(&Utc);
    
    let id = Uuid::new_v4();
    let log = RawAttendanceLog {
        id,
        workspace_id: ws_id,
        employee_id,
        punch_time: dt,
        source: LogSource::Import,
        device_id: None,
        raw_data: None,
        created_at: Utc::now(),
    };

    apiserver::repositories::raw_attendance_log_repository::create_raw_log(pool, &log)
        .await
        .unwrap();
    id
}

#[tokio::test]
async fn test_pairing_two_logs_same_day() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().with_timezone("UTC").create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. First log (Punch In)
    let log1_id = create_log(&pool, ws_id, emp_id, "2026-01-14 08:00:00").await;
    let log1 = apiserver::repositories::raw_attendance_log_repository::get_raw_log_by_id(&pool, log1_id).await.unwrap().unwrap();
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
    let log2_id = create_log(&pool, ws_id, emp_id, "2026-01-14 17:00:00").await;
    let log2 = apiserver::repositories::raw_attendance_log_repository::get_raw_log_by_id(&pool, log2_id).await.unwrap().unwrap();
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
        let log_id = create_log(&pool, ws_id, emp_id, t).await;
        let log = apiserver::repositories::raw_attendance_log_repository::get_raw_log_by_id(&pool, log_id).await.unwrap().unwrap();
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

    let log1_id = create_log(&pool, ws_id, emp_id, "2026-01-14 22:05:00").await;
    let log1 = apiserver::repositories::raw_attendance_log_repository::get_raw_log_by_id(&pool, log1_id).await.unwrap().unwrap();
    PairingService::process_new_raw_log(&pool, &log1)
        .await
        .unwrap();

    let log2_id = create_log(&pool, ws_id, emp_id, "2026-01-15 05:55:00").await;
    let log2 = apiserver::repositories::raw_attendance_log_repository::get_raw_log_by_id(&pool, log2_id).await.unwrap().unwrap();
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
async fn test_pairing_no_double_use_and_lenient_dedup() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    
    let ws_id = factory.workspace()
        .with_name("Robustness Workspace")
        .with_timezone("Asia/Ho_Chi_Minh")
        .create().await;
    
    sqlx::query("UPDATE workspaces SET pairing_algorithm = 'sequential' WHERE id = $1")
        .bind(ws_id)
        .execute(&pool)
        .await
        .unwrap();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Reported scenario:
    // 17:31:03, 17:31:04, 07:20:13, 17:37:35
    let logs = vec![
        "2025-12-22 17:31:03",
        "2025-12-22 17:31:04",
        "2025-12-23 07:20:13",
        "2025-12-23 17:37:35",
    ];

    for t in logs {
        let log_id = create_log_vn(&pool, ws_id, emp_id, t).await;
        let log = apiserver::repositories::raw_attendance_log_repository::get_raw_log_by_id(&pool, log_id).await.unwrap().unwrap();
        PairingService::process_new_raw_log(&pool, &log)
            .await
            .unwrap();
    }

    let entries = sqlx::query!(
        "SELECT punch_in_time, punch_out_time, generation_metadata FROM time_entries WHERE employee_id = $1 ORDER BY punch_in_time",
        emp_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // With 0s deduplication (identity only) and double-use fix:
    // Pairs: (17:31:03, 17:31:04) and (07:20:13, 17:37:35)
    assert_eq!(entries.len(), 2);
    
    // Check metadata
    assert!(entries[0].generation_metadata.is_some());
    let meta = entries[0].generation_metadata.as_ref().unwrap();
    assert_eq!(meta["algorithm"], "sequential");
    assert!(meta["log_ids"].as_array().unwrap().len() >= 1);
    
    // Check first entry (17:31 to 17:31)
    assert_eq!(entries[0].punch_in_time.with_timezone(&Ho_Chi_Minh).hour(), 17);
    assert_eq!(entries[0].punch_out_time.unwrap().with_timezone(&Ho_Chi_Minh).hour(), 17);
    
    // Check second entry (07:20 to 17:37)
    assert_eq!(entries[1].punch_in_time.with_timezone(&Ho_Chi_Minh).hour(), 7);
    assert_eq!(entries[1].punch_out_time.unwrap().with_timezone(&Ho_Chi_Minh).hour(), 17);
}
