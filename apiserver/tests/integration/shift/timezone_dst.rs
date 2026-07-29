//! Timezone & DST Transition Pairing Tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::LogSource;
use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

#[tokio::test]
async fn test_dst_spring_forward() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("America/New_York")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift: 22:00 -> 06:00 next day
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift DST Spring")
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    // DST Spring Forward in US Eastern timezone is Sunday, March 8, 2026 (clock jumps 02:00 -> 03:00)
    // The shift starts on Saturday, March 7, 2026.
    let shift_date = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();

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
    let tz: Tz = "America/New_York".parse().unwrap();
    // Punch in at 21:55:00 EST on Saturday
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(21, 55, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);
    // Punch out at 06:05:00 EDT on Sunday
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
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: repo.clone(),
            roster_repo: std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            condition_type_repo: std::sync::Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
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
        },
    ));
    apiserver::services::pairing::PairingService::new(
        apiserver::services::pairing::PairingServiceArgs::new(
            repo.clone(),
            std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
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
    // Physical duration of 21:55 EST (02:55 UTC) to 06:05 EDT (10:05 UTC)
    // is 7 hours and 10 minutes = 430 minutes
    assert_eq!(entry.duration_minutes(), Some(430));
}

#[tokio::test]
async fn test_dst_fall_back() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("America/New_York")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift: 22:00 -> 06:00 next day
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift DST Fall")
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    // DST Fall Back in US Eastern timezone is Sunday, November 1, 2026 (clock falls back 02:00 -> 01:00)
    // The shift starts on Saturday, October 31, 2026.
    let shift_date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();

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
    let tz: Tz = "America/New_York".parse().unwrap();
    // Punch in at 21:55:00 EDT on Saturday
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(21, 55, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);
    // Punch out at 06:05:00 EST on Sunday
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
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: repo.clone(),
            roster_repo: std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            condition_type_repo: std::sync::Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
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
        },
    ));
    apiserver::services::pairing::PairingService::new(
        apiserver::services::pairing::PairingServiceArgs::new(
            repo.clone(),
            std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
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
    // Physical duration of 21:55 EDT (01:55 UTC) to 06:05 EST (11:05 UTC)
    // is 9 hours and 10 minutes = 550 minutes
    assert_eq!(entry.duration_minutes(), Some(550));
}

#[tokio::test]
async fn test_dst_fall_back_repeated_hour() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("America/New_York")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift: 22:00 -> 06:00 next day
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift DST Fall Repeated Hour")
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();

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

    // 2. Insert Logs in the repeated fallback hour
    let tz: Tz = "America/New_York".parse().unwrap();
    // Punch in at 22:00 EDT on Saturday
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(22, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);
    // Punch out at 01:30 EST (the second occurrence of 01:30, after fallback transition)
    let res = tz.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 11, 1)
            .unwrap()
            .and_hms_opt(1, 30, 0)
            .unwrap(),
    );
    let punch_out_utc = match res {
        chrono::LocalResult::Ambiguous(_, t2) => t2.with_timezone(&Utc),
        _ => panic!("Expected ambiguous local time during Fall Back transition"),
    };

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
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: repo.clone(),
            roster_repo: std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            condition_type_repo: std::sync::Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
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
        },
    ));
    apiserver::services::pairing::PairingService::new(
        apiserver::services::pairing::PairingServiceArgs::new(
            repo.clone(),
            std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
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
    // Physical duration between 22:00 EDT (02:00 UTC) and 01:30 EST (06:30 UTC)
    // is exactly 4.5 hours = 270 minutes
    assert_eq!(entry.duration_minutes(), Some(270));
}

#[tokio::test]
async fn test_dst_spring_forward_day_shift() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("America/New_York")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Day Shift on the Sunday of Spring Forward: 08:00 -> 17:00
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Day Shift Spring Forward")
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .create()
        .await;

    // Sunday, March 8, 2026 (clock jumps forward at 02:00)
    let shift_date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();

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
    let tz: Tz = "America/New_York".parse().unwrap();
    // Punch in at 07:55:00 EDT (which is 11:55:00 UTC)
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(7, 55, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);
    // Punch out at 17:05:00 EDT (which is 21:05:00 UTC)
    let punch_out_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(17, 5, 0).unwrap())
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
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: repo.clone(),
            roster_repo: std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            condition_type_repo: std::sync::Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
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
        },
    ));
    apiserver::services::pairing::PairingService::new(
        apiserver::services::pairing::PairingServiceArgs::new(
            repo.clone(),
            std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
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
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(entries.len(), 1);
    let entry = &entries[0];

    assert_eq!(entry.punch_in_time(), punch_in_utc);
    assert_eq!(entry.punch_out_time(), Some(punch_out_utc));
    // Physical duration of 07:55 EDT (11:55 UTC) to 17:05 EDT (21:05 UTC)
    // is 9 hours and 10 minutes = 550 minutes
    assert_eq!(entry.duration_minutes(), Some(550));
}
