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
    let pool = test_db.diesel_pools.clone();
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
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employee_shift(ws_id, employee_id, shift_id, shift_date, None)
        .await
        .unwrap();

    let roster_repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

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

    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;

    let raw_log_in = apiserver::models::RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id,
        punch_time: punch_in_utc,
        source: LogSource::Device,
        device_id: None,
        raw_data: None,
        punch: -1,
        status: -1,
        created_at: Utc::now(),
    };
    raw_repo.create_raw_log(ws_id, &raw_log_in).await.unwrap();

    let raw_log_out = apiserver::models::RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id,
        punch_time: punch_out_utc,
        source: LogSource::Device,
        device_id: None,
        raw_data: None,
        punch: -1,
        status: -1,
        created_at: Utc::now(),
    };
    raw_repo.create_raw_log(ws_id, &raw_log_out).await.unwrap();

    // 3. Process Pairing
    let repo = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs {
            time_entry_repo: repo.clone(),
            roster_repo: std::sync::Arc::new(roster_repo),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            workspace_repo: std::sync::Arc::new(
                apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
            ),
            employee_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
            ),
            leave_request_repo: std::sync::Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            raw_log_repo: std::sync::Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            shift_repo: std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
        },
    )
    .reprocess_daily_attendance(ws_id, employee_id, shift_date, tz)
    .await
    .unwrap();

    // 4. Verify
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let entries = repo
        .get_employee_time_entries(
            ws_id,
            employee_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            (shift_date + Duration::days(1))
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(entries.len(), 1);
    let entry = &entries[0];

    assert_eq!(entry.punch_in_time, punch_in_utc);
    assert_eq!(entry.punch_out_time, Some(punch_out_utc));
    assert_eq!(entry.duration_minutes, Some(490));
}
