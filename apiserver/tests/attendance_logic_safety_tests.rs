use chrono::{Duration, NaiveDate, Utc, TimeZone};
use chrono_tz::Tz;
use uuid::Uuid;

mod common;
use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_daily_isolation_and_human_error_tolerance() {
    let db = TestDatabase::new().await;
    let pool = db.pool();
    let factory = TestFactory::new(pool.clone());
    
    let ws_id = factory.workspace().create().await;
    
    // Set to Anchor First
    sqlx::query!("UPDATE workspaces SET pairing_algorithm = 'anchor_first' WHERE id = $1", ws_id)
        .execute(&pool).await.unwrap();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    
    // 1. Create a Standard Day Shift: 07:30 - 16:30
    let shift_id = factory.shift()
        .with_workspace(ws_id)
        .with_name("Day Shift")
        .with_times(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap().and_hms_opt(7, 30, 0).unwrap().time(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap().and_hms_opt(16, 30, 0).unwrap().time()
        )
        .create().await;

    // Assign shift to employee for specific dates
    let date_30 = NaiveDate::from_ymd_opt(2025, 12, 30).unwrap();
    let date_31 = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    
    for date in [date_30, date_31] {
        sqlx::query!(
            "INSERT INTO employee_shifts (id, employee_id, shift_id, start_date) VALUES ($1, $2, $3, $4)",
            Uuid::new_v4(), emp_id, shift_id, date
        ).execute(&pool).await.unwrap();
    }

    // 2. Insert Raw Logs with "Human Errors"
    let tz: Tz = "Asia/Bangkok".parse().unwrap();
    
    // Day 30 logs:
    // 07:18 (Normal In)
    // 07:18:30 (Double punch - same minute)
    // 17:40 (Late Out - 10h total)
    let logs_30 = [
        tz.from_local_datetime(&date_30.and_hms_opt(7, 18, 0).unwrap()).unwrap().with_timezone(&Utc),
        tz.from_local_datetime(&date_30.and_hms_opt(7, 18, 30).unwrap()).unwrap().with_timezone(&Utc),
        tz.from_local_datetime(&date_30.and_hms_opt(17, 40, 0).unwrap()).unwrap().with_timezone(&Utc),
    ];

    // Day 31 logs:
    // 07:16 (Normal In)
    let logs_31 = [
        tz.from_local_datetime(&date_31.and_hms_opt(7, 16, 0).unwrap()).unwrap().with_timezone(&Utc),
    ];

    for punch_time in logs_30.iter().chain(logs_31.iter()) {
        sqlx::query!(
            "INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source) VALUES ($1, $2, $3, $4, 'device')",
            Uuid::new_v4(), ws_id, emp_id, punch_time
        ).execute(&pool).await.unwrap();
    }

    // 3. Reprocess Day 30
    apiserver::services::pairing_service::PairingService::reprocess_daily_attendance(
        &pool, emp_id, date_30, tz, "anchor_first"
    ).await.unwrap();

    // Verification for Day 30:
    // Should have 1 entry: 07:18 -> 17:40
    // Should NOT have 'stolen' 07:16 from Day 31
    let entries_30 = sqlx::query!(
        r#"SELECT punch_in_time, punch_out_time FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time < $3 ORDER BY punch_in_time"#,
        emp_id, 
        tz.from_local_datetime(&date_30.and_hms_opt(0,0,0).unwrap()).unwrap().with_timezone(&Utc),
        tz.from_local_datetime(&date_31.and_hms_opt(0,0,0).unwrap()).unwrap().with_timezone(&Utc)
    ).fetch_all(&pool).await.unwrap();

    assert_eq!(entries_30.len(), 1, "Day 30 should only have ONE paired session");
    assert_eq!(entries_30[0].punch_in_time.with_timezone(&tz).time(), NaiveDate::from_ymd_opt(2025, 1, 1).unwrap().and_hms_opt(7, 18, 0).unwrap().time());
    assert_eq!(entries_30[0].punch_out_time.unwrap().with_timezone(&tz).time(), NaiveDate::from_ymd_opt(2025, 1, 1).unwrap().and_hms_opt(17, 40, 0).unwrap().time());
    
    // 4. Reprocess Day 31
    apiserver::services::pairing_service::PairingService::reprocess_daily_attendance(
        &pool, emp_id, date_31, tz, "anchor_first"
    ).await.unwrap();

    // Verification for Day 31:
    let entries_31 = sqlx::query!(
        r#"SELECT punch_in_time, punch_out_time FROM time_entries WHERE employee_id = $1 AND punch_in_time >= $2 AND punch_in_time < $3 ORDER BY punch_in_time"#,
        emp_id,
        tz.from_local_datetime(&date_31.and_hms_opt(0,0,0).unwrap()).unwrap().with_timezone(&Utc),
        tz.from_local_datetime(&(date_31 + Duration::days(1)).and_hms_opt(0,0,0).unwrap()).unwrap().with_timezone(&Utc)
    ).fetch_all(&pool).await.unwrap();

    assert_eq!(entries_31.len(), 1, "Day 31 should have its own session");
    assert_eq!(entries_31[0].punch_in_time.with_timezone(&tz).time(), NaiveDate::from_ymd_opt(2025, 1, 1).unwrap().and_hms_opt(7, 16, 0).unwrap().time());
    assert!(entries_31[0].punch_out_time.is_none(), "Day 31 session should be open (Missing Out)");
}

