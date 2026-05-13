//! Overnight shift pairing tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::LogSource;
use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

#[tokio::test]
async fn test_overnight_shift_pairing() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Overnight Shift: 22:00 -> 06:00
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Night Shift")
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign shift
    sqlx::query("INSERT INTO employee_shifts (id, employee_id, shift_id, start_date) VALUES ($1, $2, $3, $4)")
        .bind(Uuid::new_v4()).bind(employee_id).bind(shift_id).bind(shift_date)
        .execute(&pool).await.unwrap();

    // 2. Insert Logs
    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(21, 55, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let punch_out_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 5, 0)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    sqlx::query("INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source) VALUES ($1, $2, $3, $4, $5)")
        .bind(Uuid::new_v4()).bind(ws_id).bind(employee_id).bind(punch_in_utc).bind(LogSource::Device as LogSource)
        .execute(&pool).await.unwrap();

    sqlx::query("INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source) VALUES ($1, $2, $3, $4, $5)")
        .bind(Uuid::new_v4()).bind(ws_id).bind(employee_id).bind(punch_out_utc).bind(LogSource::Device as LogSource)
        .execute(&pool).await.unwrap();

    // 3. Process Pairing
    let repo = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        pool.clone(),
        diesel_pools.clone(),
    ));
    apiserver::services::pairing_service::PairingService::new(
        repo,
        std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        )),
        std::sync::Arc::new(
            apiserver::repositories::DieselAttendancePolicyRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ),
        ),
        pool.clone(),
        diesel_pools.clone(),
    )
    .reprocess_daily_attendance(ws_id, employee_id, shift_date, tz)
    .await
    .unwrap();

    // 4. Verify
    let entry = sqlx::query!("SELECT punch_in_time, punch_out_time, duration_minutes FROM time_entries WHERE employee_id = $1", employee_id)
        .fetch_one(&pool).await.unwrap();

    assert_eq!(entry.punch_in_time, punch_in_utc);
    assert_eq!(entry.punch_out_time, Some(punch_out_utc));
    assert_eq!(entry.duration_minutes, Some(490));
}
