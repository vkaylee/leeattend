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
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

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
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
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

    let raw_log_in = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &raw_log_in).await.unwrap();

    let raw_log_out = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
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
    .reprocess_daily_attendance(ws_id, employment_id, shift_date, tz)
    .await
    .unwrap();

    // 4. Verify
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let entries = repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
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

    assert_eq!(entry.punch_in_time(), punch_in_utc);
    assert_eq!(entry.punch_out_time(), Some(punch_out_utc));
    assert_eq!(entry.duration_minutes(), Some(490));
}

#[tokio::test]
async fn test_overnight_shift_multiple_checkout_punches_subminute_deduplication() {
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
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift: 18:00 -> 02:00 next day
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift 18-02")
        .with_times(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign shift
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
        .await
        .unwrap();

    let roster_repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    // 2. Insert Logs
    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(18, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    // First checkout log at 06:25:01
    let punch_out1_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 1)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    // Second checkout log at 06:25:02 (same minute)
    let punch_out2_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 2)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    // Expected punch out in time entry is at 06:25:00 (rounded to minute)
    let expected_punch_out_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 0)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;

    let log_in = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &log_in).await.unwrap();

    let log_out1 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out1_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &log_out1).await.unwrap();

    let log_out2 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out2_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &log_out2).await.unwrap();

    // 3. Process Pairing for Day 1
    let time_entry_repo = std::sync::Arc::new(
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
    );
    let service = apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
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
    );

    service
        .reprocess_daily_attendance(ws_id, employment_id, shift_date, tz)
        .await
        .unwrap();

    // 4. Verify Day 1 Time Entry contains both raw log IDs in metadata
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let day1_entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day1_entries.len(), 1);
    let entry = &day1_entries[0];
    assert_eq!(entry.punch_in_time(), punch_in_utc);
    // Should choose the latest punch in the same minute as checkout time (06:25:00 after minute precision conversion)
    assert_eq!(entry.punch_out_time(), Some(expected_punch_out_utc));

    let meta = entry.generation_metadata().unwrap();
    let log_ids = meta["log_ids"].as_array().unwrap();
    // Verify that both punch_out1_utc and punch_out2_utc are registered as consumed log IDs
    let log_ids_strs: Vec<&str> = log_ids.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(log_ids_strs.contains(&log_in.id().to_string().as_str()));
    assert!(log_ids_strs.contains(&log_out1.id().to_string().as_str()));
    assert!(log_ids_strs.contains(&log_out2.id().to_string().as_str()));

    // 5. Reprocess Day 2 (Day 2 has no shift / is off day)
    let next_date = shift_date + Duration::days(1);
    service
        .reprocess_daily_attendance(ws_id, employment_id, next_date, tz)
        .await
        .unwrap();

    // Verify Day 2 has ZERO time entries because the 06:25:02 log was consumed by Day 1
    let day2_entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            next_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            next_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day2_entries.len(), 0);
}
